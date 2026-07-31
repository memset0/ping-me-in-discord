use std::{fmt, time::Duration};

use anyhow::{Context, Result, anyhow, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use crate::{
    config::{DiscordConfig, redact},
    state::AppState,
};

const DISCORD_API_V10: &str = "https://discord.com/api/v10/";

#[derive(Clone)]
pub struct WebhookUrl {
    url: Url,
    id: String,
    token: String,
}

impl WebhookUrl {
    pub fn parse(value: &str) -> Result<Self> {
        Self::parse_with_policy(value, true)
    }

    fn parse_with_policy(value: &str, enforce_discord_host: bool) -> Result<Self> {
        let mut url = Url::parse(value).context("webhook URL is invalid")?;
        if enforce_discord_host {
            ensure!(url.scheme() == "https", "webhook URL must use HTTPS");
        } else {
            ensure!(
                matches!(url.scheme(), "http" | "https"),
                "test webhook URL must use HTTP or HTTPS"
            );
        }
        let host = url.host_str().context("webhook URL must include a host")?;
        if enforce_discord_host {
            ensure!(
                is_discord_host(host),
                "webhook URL host must be discord.com or discordapp.com"
            );
        }
        url.set_fragment(None);
        url.set_query(None);

        let segments: Vec<_> = url
            .path_segments()
            .context("webhook URL path is invalid")?
            .collect();
        let webhook_index = segments
            .iter()
            .position(|segment| *segment == "webhooks")
            .context("webhook URL path must contain /webhooks/<id>/<token>")?;
        let id = segments
            .get(webhook_index + 1)
            .filter(|value| {
                !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
            })
            .context("webhook URL contains an invalid webhook ID")?
            .to_string();
        let token = segments
            .get(webhook_index + 2)
            .filter(|value| !value.is_empty())
            .context("webhook URL does not contain a token")?
            .to_string();
        ensure!(
            !token.chars().any(char::is_whitespace),
            "webhook URL contains an invalid token"
        );

        Ok(Self { url, id, token })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn secret_url(&self) -> &str {
        self.url.as_str()
    }
}

impl fmt::Debug for WebhookUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebhookUrl")
            .field("id", &self.id)
            .field("url", &"<redacted>")
            .finish()
    }
}

pub struct DiscordClient {
    http: reqwest::Client,
    api_base: Url,
    enforce_discord_host: bool,
    max_rate_limit_retries: usize,
}

impl DiscordClient {
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("notify-me-on-discord/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .context("could not initialize the HTTP client")?;
        Ok(Self {
            http,
            api_base: Url::parse(DISCORD_API_V10).expect("Discord API URL is valid"),
            enforce_discord_host: true,
            max_rate_limit_retries: 3,
        })
    }

    #[cfg(test)]
    fn for_test(api_base: Url) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_base,
            enforce_discord_host: false,
            max_rate_limit_retries: 3,
        }
    }

    pub async fn resolve_webhook(
        &self,
        config: &DiscordConfig,
        state: &mut AppState,
        channel_id: Option<&str>,
    ) -> Result<(WebhookUrl, bool)> {
        if let Some(channel_id) = nonempty(channel_id) {
            ensure!(
                channel_id
                    .chars()
                    .all(|character| character.is_ascii_digit()),
                "resolved Discord channel ID must contain only digits"
            );
            if let Some(value) = state.provisioned_webhooks.get(channel_id) {
                return Ok((WebhookUrl::parse_with_policy(value, self.enforce_discord_host).with_context(|| {
                    format!(
                        "cached provisioned webhook for channel {channel_id} is invalid; remove it from the state file and run `webhook setup --channel {channel_id}`"
                    )
                })?, false));
            }

            let webhook = self.provision(config, channel_id).await.with_context(|| {
                if nonempty(config.bot_token.as_deref()).is_none()
                    && nonempty(config.webhook_url.as_deref()).is_some()
                {
                    "a direct webhook is bound to its existing channel and cannot satisfy --channel; configure discord.bot_token for channel routing".to_owned()
                } else {
                    format!("could not prepare a webhook for channel {channel_id}")
                }
            })?;
            state
                .provisioned_webhooks
                .insert(channel_id.to_owned(), webhook.secret_url().to_owned());
            return Ok((webhook, true));
        }

        if let Some(value) = nonempty(config.webhook_url.as_deref()) {
            return Ok((
                WebhookUrl::parse_with_policy(value, self.enforce_discord_host)?,
                false,
            ));
        }
        if let Some(value) = nonempty(state.provisioned_webhook_url.as_deref()) {
            return Ok((
                WebhookUrl::parse_with_policy(value, self.enforce_discord_host).context(
                    "legacy cached provisioned webhook is invalid; remove provisioned_webhook_url from the state file",
                )?,
                false,
            ));
        }
        bail!(
            "no Discord destination is configured; set discord.webhook_url or select a channel with --channel, template frontmatter, or defaults.channel"
        )
    }

    pub async fn provision(&self, config: &DiscordConfig, channel_id: &str) -> Result<WebhookUrl> {
        let bot_token = nonempty(config.bot_token.as_deref()).context(
            "discord.bot_token is required for channel routing (or set DISCORD_NOTIFICATION_BOT_TOKEN)",
        )?;
        ensure!(
            channel_id
                .chars()
                .all(|character| character.is_ascii_digit()),
            "Discord channel ID must contain only digits"
        );

        let endpoint = self
            .api_base
            .join(&format!("channels/{channel_id}/webhooks"))
            .context("could not construct the Discord webhook provisioning URL")?;
        let response = self
            .send_with_retry(Method::GET, endpoint.clone(), Some(bot_token), None, &[])
            .await?;
        if response.status == StatusCode::FORBIDDEN {
            bail!(
                "Discord denied access to channel webhooks; grant the Bot MANAGE_WEBHOOKS in channel {channel_id}"
            );
        }
        ensure_success("list channel webhooks", &response, &[bot_token])?;
        let webhooks: Vec<WebhookResponse> = serde_json::from_str(&response.body)
            .context("Discord returned invalid webhook data")?;
        if let Some(webhook) = webhooks.into_iter().find(|webhook| {
            webhook.kind == 1 && webhook.name.as_deref() == Some(&config.webhook_name)
        }) {
            return self.webhook_from_response(webhook);
        }

        let response = self
            .send_with_retry(
                Method::POST,
                endpoint,
                Some(bot_token),
                Some(&json!({ "name": config.webhook_name })),
                &[],
            )
            .await?;
        if response.status == StatusCode::FORBIDDEN {
            bail!(
                "Discord denied webhook creation; grant the Bot MANAGE_WEBHOOKS in channel {channel_id}"
            );
        }
        ensure_success("create webhook", &response, &[bot_token])?;
        let webhook: WebhookResponse = serde_json::from_str(&response.body)
            .context("Discord returned invalid webhook data")?;
        self.webhook_from_response(webhook)
    }

    pub async fn modify_avatar(&self, webhook: &WebhookUrl, png: &[u8]) -> Result<()> {
        self.set_avatar(
            webhook,
            Value::String(format!("data:image/png;base64,{}", STANDARD.encode(png))),
        )
        .await
    }

    pub async fn reset_avatar(&self, webhook: &WebhookUrl) -> Result<()> {
        self.set_avatar(webhook, Value::Null).await
    }

    async fn set_avatar(&self, webhook: &WebhookUrl, avatar: Value) -> Result<()> {
        let response = self
            .send_with_retry(
                Method::PATCH,
                webhook.url.clone(),
                None,
                Some(&json!({ "avatar": avatar })),
                &[webhook.secret_url(), &webhook.token],
            )
            .await?;
        ensure_success(
            "update webhook avatar",
            &response,
            &[webhook.secret_url(), &webhook.token],
        )
    }

    pub async fn execute(
        &self,
        webhook: &WebhookUrl,
        payload: &Value,
        thread_id: Option<&str>,
    ) -> Result<String> {
        let mut endpoint = webhook.url.clone();
        {
            let mut query = endpoint.query_pairs_mut();
            query.append_pair("wait", "true");
            if let Some(thread_id) = thread_id {
                query.append_pair("thread_id", thread_id);
            }
        }

        let response = self
            .send_with_retry(
                Method::POST,
                endpoint,
                None,
                Some(payload),
                &[webhook.secret_url(), &webhook.token],
            )
            .await?;
        ensure_success(
            "execute webhook",
            &response,
            &[webhook.secret_url(), &webhook.token],
        )?;
        let message: DiscordMessage = serde_json::from_str(&response.body)
            .context("Discord returned invalid message data")?;
        ensure!(
            !message.id.is_empty(),
            "Discord response did not include a message ID"
        );
        Ok(message.id)
    }

    fn webhook_from_response(&self, response: WebhookResponse) -> Result<WebhookUrl> {
        if let Some(url) = response.url {
            return WebhookUrl::parse_with_policy(&url, self.enforce_discord_host);
        }
        let token = response
            .token
            .context("Discord incoming webhook response did not include a token")?;
        let url = if self.enforce_discord_host {
            format!("https://discord.com/api/webhooks/{}/{}", response.id, token)
        } else {
            self.api_base
                .join(&format!("webhooks/{}/{}", response.id, token))
                .context("could not construct test webhook URL")?
                .to_string()
        };
        WebhookUrl::parse_with_policy(&url, self.enforce_discord_host)
    }

    async fn send_with_retry(
        &self,
        method: Method,
        url: Url,
        bot_token: Option<&str>,
        body: Option<&Value>,
        extra_secrets: &[&str],
    ) -> Result<ApiResponse> {
        let mut attempts = 0;
        loop {
            let mut request = self.http.request(method.clone(), url.clone());
            if let Some(bot_token) = bot_token {
                request = request.header("Authorization", format!("Bot {bot_token}"));
            }
            if let Some(body) = body {
                request = request.json(body);
            }
            let response = request.send().await.map_err(|_| {
                anyhow!("Discord request failed before a response was received (URL redacted)")
            })?;
            let status = response.status();
            let response_body = response
                .text()
                .await
                .map_err(|_| anyhow!("Discord response body could not be read"))?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempts < self.max_rate_limit_retries {
                attempts += 1;
                let retry_after = serde_json::from_str::<RateLimitBody>(&response_body)
                    .ok()
                    .map(|body| body.retry_after)
                    .filter(|delay| delay.is_finite() && *delay >= 0.0)
                    .unwrap_or(1.0)
                    .min(60.0);
                tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;
                continue;
            }

            let mut secrets = extra_secrets.to_vec();
            if let Some(bot_token) = bot_token {
                secrets.push(bot_token);
            }
            return Ok(ApiResponse {
                status,
                body: redact(&response_body, &secrets),
            });
        }
    }
}

struct ApiResponse {
    status: StatusCode,
    body: String,
}

#[derive(Deserialize)]
struct WebhookResponse {
    id: String,
    #[serde(rename = "type")]
    kind: u8,
    name: Option<String>,
    token: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct DiscordMessage {
    id: String,
}

#[derive(Deserialize)]
struct RateLimitBody {
    retry_after: f64,
}

fn ensure_success(operation: &str, response: &ApiResponse, secrets: &[&str]) -> Result<()> {
    if response.status.is_success() {
        return Ok(());
    }
    let body = truncate(&redact(&response.body, secrets), 1_000);
    bail!(
        "Discord could not {operation}: HTTP {}{}",
        response.status.as_u16(),
        if body.trim().is_empty() {
            String::new()
        } else {
            format!(": {}", body.trim())
        }
    )
}

fn truncate(value: &str, max_characters: usize) -> String {
    let mut characters = value.chars();
    let prefix: String = characters.by_ref().take(max_characters).collect();
    if characters.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.trim().is_empty())
}

fn is_discord_host(host: &str) -> bool {
    host == "discord.com"
        || host.ends_with(".discord.com")
        || host == "discordapp.com"
        || host.ends_with(".discordapp.com")
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, header, method, path, query_param},
    };

    use super::*;

    fn test_config() -> DiscordConfig {
        DiscordConfig {
            webhook_url: None,
            bot_token: Some("bot-secret".to_owned()),
            webhook_name: "Notify Me".to_owned(),
        }
    }

    async fn test_client(server: &MockServer) -> DiscordClient {
        DiscordClient::for_test(
            Url::parse(&format!("{}/api/v10/", server.uri())).expect("mock URL is valid"),
        )
    }

    fn test_webhook(server: &MockServer) -> WebhookUrl {
        WebhookUrl::parse_with_policy(
            &format!("{}/api/v10/webhooks/456/webhook-secret", server.uri()),
            false,
        )
        .unwrap()
    }

    #[test]
    fn parses_and_redacts_official_webhook() {
        let webhook =
            WebhookUrl::parse("https://discord.com/api/webhooks/123/a-secret-token").unwrap();
        assert_eq!(webhook.id(), "123");
        let debug = format!("{webhook:?}");
        assert!(!debug.contains("a-secret-token"));
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn rejects_non_discord_webhook_hosts() {
        let error = WebhookUrl::parse("https://example.com/api/webhooks/123/token")
            .unwrap_err()
            .to_string();
        assert!(error.contains("discord.com"));
    }

    #[tokio::test]
    async fn reuses_existing_bot_managed_webhook() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v10/channels/123/webhooks"))
            .and(header("authorization", "Bot bot-secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
                "id": "456",
                "type": 1,
                "name": "Notify Me",
                "token": "webhook-secret"
            }])))
            .expect(1)
            .mount(&server)
            .await;

        let webhook = test_client(&server)
            .await
            .provision(&test_config(), "123")
            .await
            .unwrap();
        assert_eq!(webhook.id(), "456");
    }

    #[tokio::test]
    async fn creates_missing_webhook() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v10/channels/123/webhooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v10/channels/123/webhooks"))
            .and(header("authorization", "Bot bot-secret"))
            .and(body_json(json!({ "name": "Notify Me" })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "789",
                "type": 1,
                "name": "Notify Me",
                "token": "new-secret"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let webhook = test_client(&server)
            .await
            .provision(&test_config(), "123")
            .await
            .unwrap();
        assert_eq!(webhook.id(), "789");
    }

    #[tokio::test]
    async fn modifies_avatar_and_executes_with_confirmation() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v10/webhooks/456/webhook-secret"))
            .and(body_json(json!({
                "avatar": "data:image/png;base64,cG5n"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "456",
                "type": 1,
                "name": "Notify Me",
                "token": "webhook-secret"
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v10/webhooks/456/webhook-secret"))
            .and(query_param("wait", "true"))
            .and(body_json(json!({"content": "hello"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "999"})))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server).await;
        let webhook = test_webhook(&server);
        client.modify_avatar(&webhook, b"png").await.unwrap();
        let message_id = client
            .execute(&webhook, &json!({"content": "hello"}), None)
            .await
            .unwrap();
        assert_eq!(message_id, "999");
    }

    #[tokio::test]
    async fn resets_a_previously_applied_avatar() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v10/webhooks/456/webhook-secret"))
            .and(body_json(json!({"avatar": null})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "456",
                "type": 1,
                "name": "Notify Me",
                "token": "webhook-secret"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server).await;
        client.reset_avatar(&test_webhook(&server)).await.unwrap();
    }

    #[tokio::test]
    async fn redacts_webhook_secrets_from_errors() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v10/webhooks/456/webhook-secret"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_string("bad webhook-secret and full secret URL"),
            )
            .mount(&server)
            .await;
        let client = test_client(&server).await;
        let webhook = test_webhook(&server);

        let error = client
            .execute(&webhook, &json!({"content": "hello"}), None)
            .await
            .unwrap_err()
            .to_string();
        assert!(!error.contains("webhook-secret"));
        assert!(error.contains("<redacted>"));
    }

    #[tokio::test]
    async fn retries_rate_limited_execution() {
        let server = MockServer::start().await;
        let calls = Arc::new(AtomicUsize::new(0));
        let responder_calls = Arc::clone(&calls);
        Mock::given(method("POST"))
            .and(path("/api/v10/webhooks/456/webhook-secret"))
            .respond_with(move |_: &wiremock::Request| {
                if responder_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    ResponseTemplate::new(429).set_body_json(json!({ "retry_after": 0.0 }))
                } else {
                    ResponseTemplate::new(200).set_body_json(json!({ "id": "1000" }))
                }
            })
            .expect(2)
            .mount(&server)
            .await;
        let client = test_client(&server).await;
        let webhook = test_webhook(&server);

        let id = client
            .execute(&webhook, &json!({"content": "hello"}), None)
            .await
            .unwrap();
        assert_eq!(id, "1000");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn direct_webhook_has_priority_over_cached_state() {
        let client = DiscordClient::new().unwrap();
        let config = DiscordConfig {
            webhook_url: Some("https://discord.com/api/webhooks/111/direct-secret".to_owned()),
            ..test_config()
        };
        let mut state = AppState {
            provisioned_webhook_url: Some(
                "https://discord.com/api/webhooks/222/cached-secret".to_owned(),
            ),
            ..AppState::default()
        };

        let (webhook, changed) = client
            .resolve_webhook(&config, &mut state, None)
            .await
            .unwrap();
        assert_eq!(webhook.id(), "111");
        assert!(!changed);
    }

    #[tokio::test]
    async fn selected_channel_uses_its_own_cached_webhook() {
        let client = DiscordClient::new().unwrap();
        let mut state = AppState::default();
        state.provisioned_webhooks.insert(
            "123".to_owned(),
            "https://discord.com/api/webhooks/333/channel-secret".to_owned(),
        );

        let (webhook, changed) = client
            .resolve_webhook(&test_config(), &mut state, Some("123"))
            .await
            .unwrap();
        assert_eq!(webhook.id(), "333");
        assert!(!changed);
    }

    #[tokio::test]
    async fn provisions_and_caches_each_channel_independently() {
        let server = MockServer::start().await;
        for (channel, webhook, token) in [
            ("123", "456", "first-secret"),
            ("124", "457", "second-secret"),
        ] {
            let endpoint = format!("/api/v10/channels/{channel}/webhooks");
            Mock::given(method("GET"))
                .and(path(endpoint.clone()))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .and(path(endpoint))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "id": webhook,
                    "type": 1,
                    "name": "Notify Me",
                    "token": token
                })))
                .expect(1)
                .mount(&server)
                .await;
        }

        let client = test_client(&server).await;
        let mut state = AppState::default();
        let (first, first_changed) = client
            .resolve_webhook(&test_config(), &mut state, Some("123"))
            .await
            .unwrap();
        let (second, second_changed) = client
            .resolve_webhook(&test_config(), &mut state, Some("124"))
            .await
            .unwrap();
        let (first_cached, cached_changed) = client
            .resolve_webhook(&test_config(), &mut state, Some("123"))
            .await
            .unwrap();

        assert_eq!(first.id(), "456");
        assert_eq!(second.id(), "457");
        assert_eq!(first_cached.id(), "456");
        assert!(first_changed && second_changed);
        assert!(!cached_changed);
        assert_eq!(state.provisioned_webhooks.len(), 2);
    }

    #[tokio::test]
    async fn selected_channel_does_not_silently_use_direct_webhook() {
        let client = DiscordClient::new().unwrap();
        let config = DiscordConfig {
            webhook_url: Some("https://discord.com/api/webhooks/111/direct-secret".to_owned()),
            bot_token: None,
            webhook_name: "Notify Me".to_owned(),
        };

        let error = client
            .resolve_webhook(&config, &mut AppState::default(), Some("123"))
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("direct webhook is bound"));
    }
}
