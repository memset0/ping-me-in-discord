pub mod avatar;
pub mod cli;
pub mod config;
pub mod discord;
pub mod options;
pub mod paths;
mod runtime;
pub mod skills;
pub mod state;
pub mod template;

use std::ffi::OsString;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    avatar::{AvatarRenderer, AvatarSelection, ResolvedAvatar},
    cli::{
        AvatarCommand, ChannelsCommand, Cli, Command, ConfigCommand, SendOptions, SkillsCommand,
        TemplatesCommand, WebhookCommand,
    },
    config::LoadedConfig,
    discord::DiscordClient,
    state::StateStore,
};

pub async fn run() -> Result<()> {
    run_from(std::env::args_os()).await
}

pub async fn run_from<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    cli::execute(cli).await
}

pub async fn binary_main() {
    if let Err(error) = run().await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

async fn send_message(
    config_path: Option<PathBuf>,
    message: Option<String>,
    options: SendOptions,
) -> Result<()> {
    let loaded = load_config(config_path)?;
    let template_name = options
        .template
        .as_deref()
        .unwrap_or(&loaded.config.defaults.template);
    let context = template::build_context(message, options.data.as_deref(), &options.variables)?;
    let rendered = template::render(&loaded.templates_directory, template_name, &context)?;
    let mut rendered = options::resolve(rendered, &options, &loaded.config)?;

    if options.dry_run {
        println!(
            "{}",
            serde_json::to_string_pretty(&rendered)
                .context("could not serialize the dry-run payload")?
        );
        return Ok(());
    }

    let store = StateStore::new(loaded.data_directory.clone());
    let _lock = store.lock()?;
    let mut state = store.load()?;
    let discord = DiscordClient::new()?;
    let (mut webhook, provisioned) = discord
        .resolve_webhook(
            &loaded.config.discord,
            &mut state,
            rendered.channel.as_deref(),
        )
        .await?;
    let mut state_changed = provisioned;
    state_changed |= discord
        .restore_legacy_base_avatar(&mut state, &webhook)
        .await?;

    if let Some(selection) = &rendered.avatar {
        let (profile, base_directory) = match selection {
            AvatarSelection::Profile { name } => (
                avatar::select_profile(&loaded.config.avatars, name)?,
                loaded.directory.as_path(),
            ),
            AvatarSelection::Inline {
                profile,
                base_directory,
            } => (profile, base_directory.as_path()),
        };
        let avatar = AvatarRenderer::new()?
            .resolve(
                profile,
                base_directory,
                &loaded.data_directory,
                &loaded.config.emoji,
            )
            .await?;
        match avatar {
            ResolvedAvatar::RemoteUrl(url) => {
                rendered
                    .payload
                    .as_object_mut()
                    .expect("rendered payload is an object")
                    .insert("avatar_url".to_owned(), Value::String(url));
            }
            ResolvedAvatar::Png(png) => {
                let digest = avatar::digest(&png);
                let channel_id = rendered.channel.as_deref().context(
                    "locally rendered avatars require a routed Discord channel; configure defaults.channel or pass --channel",
                )?;
                let (generated_webhook, provisioned) = discord
                    .resolve_generated_avatar_webhook(
                        &loaded.config.discord,
                        &mut state,
                        channel_id,
                        &digest,
                        &png,
                    )
                    .await?;
                webhook = generated_webhook;
                state_changed |= provisioned;

                preserve_generated_avatar_username(
                    &mut rendered.payload,
                    &loaded.config.discord.webhook_name,
                );
            }
        }
    }

    if state_changed {
        store.save(&state)?;
    }

    let message_id = discord
        .execute(&webhook, &rendered.payload, rendered.thread_id.as_deref())
        .await?;
    println!("Sent Discord message {message_id}");
    Ok(())
}

fn preserve_generated_avatar_username(payload: &mut Value, base_webhook_name: &str) {
    let payload = payload
        .as_object_mut()
        .expect("rendered payload is an object");
    payload
        .entry("username".to_owned())
        .or_insert_with(|| Value::String(base_webhook_name.to_owned()));
}

async fn execute_command(config_path: Option<PathBuf>, command: Command) -> Result<()> {
    match command {
        Command::Init(arguments) => {
            let directory = if let Some(config_path) = config_path {
                config_path
                    .parent()
                    .map(PathBuf::from)
                    .context("--config path has no parent directory")?
            } else {
                paths::init_directory(arguments.portable)?
            };
            let (config, template) = config::initialize(&directory, arguments.force)?;
            println!("Created {}", config.display());
            println!("Created {}", template.display());
            Ok(())
        }
        Command::Config { command } => match command {
            ConfigCommand::Path => {
                println!("{}", paths::discover_config(config_path)?.display());
                Ok(())
            }
            ConfigCommand::Validate => {
                let loaded = load_config(config_path)?;
                template::validate_directory(
                    &loaded.templates_directory,
                    &loaded.config.defaults.template,
                )?;
                println!("Configuration is valid: {}", loaded.path.display());
                Ok(())
            }
        },
        Command::Templates {
            command: TemplatesCommand::List,
        } => {
            let loaded = load_config(config_path)?;
            for name in template::list(&loaded.templates_directory)? {
                println!("{name}");
            }
            Ok(())
        }
        Command::Skills {
            command: SkillsCommand::Install(arguments),
        } => {
            let summary = skills::install(arguments.agent, arguments.scope)?;
            skills::print_summary(&summary);
            Ok(())
        }
        Command::Channels {
            command: ChannelsCommand::List { json },
        } => {
            let loaded = load_config(config_path)?;
            let listing = channel_listing(&loaded.config)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&listing)
                        .context("could not serialize channel listing")?
                );
            } else {
                print_channel_listing(&listing);
            }
            Ok(())
        }
        Command::Avatar {
            command: AvatarCommand::List { json },
        } => {
            let loaded = load_config(config_path)?;
            let listing = avatar_listing(&loaded.config);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&listing)
                        .context("could not serialize avatar listing")?
                );
            } else {
                print_avatar_listing(&listing);
            }
            Ok(())
        }
        Command::Avatar {
            command: AvatarCommand::Preview { name, output },
        } => {
            let loaded = load_config(config_path)?;
            let profile = avatar::select_profile(&loaded.config.avatars, &name)?;
            let store = StateStore::new(loaded.data_directory.clone());
            store.ensure_directories()?;
            let png = AvatarRenderer::new()?
                .preview(
                    profile,
                    &loaded.directory,
                    &loaded.data_directory,
                    &loaded.config.emoji,
                )
                .await?;
            ensure!(
                output.extension().and_then(|value| value.to_str()) == Some("png"),
                "avatar preview output must use a .png extension"
            );
            fs::write(&output, png)
                .with_context(|| format!("could not write avatar preview {}", output.display()))?;
            println!("Wrote {}", output.display());
            Ok(())
        }
        Command::Webhook {
            command: WebhookCommand::Setup { channel },
        } => {
            let loaded = load_config(config_path)?;
            let channel_selector = channel
                .as_deref()
                .or(loaded.config.defaults.channel.as_deref());
            let channel_id = loaded.config.resolve_channel(channel_selector)?;
            let store = StateStore::new(loaded.data_directory);
            let _lock = store.lock()?;
            let mut state = store.load()?;
            let client = DiscordClient::new()?;
            let (webhook, changed) = client
                .resolve_webhook(&loaded.config.discord, &mut state, channel_id.as_deref())
                .await?;
            if changed {
                store.save(&state)?;
            }
            println!("Webhook is ready (id {})", webhook.id());
            Ok(())
        }
        Command::ReportError(arguments) => {
            report_error(config_path, arguments.channel.as_deref()).await
        }
        Command::Send(_) => unreachable!("send commands are handled by cli::execute"),
    }
}

#[derive(Debug, Serialize)]
struct ChannelListing {
    default: Option<DefaultChannel>,
    channels: Vec<ChannelEntry>,
}

#[derive(Debug, Serialize)]
struct DefaultChannel {
    selector: String,
    id: String,
}

#[derive(Debug, Serialize)]
struct ChannelEntry {
    alias: String,
    id: String,
    is_default: bool,
}

fn channel_listing(config: &config::Config) -> Result<ChannelListing> {
    let default = config
        .defaults
        .channel
        .as_deref()
        .map(|selector| {
            config
                .resolve_channel(Some(selector))
                .map(|resolved| DefaultChannel {
                    selector: selector.to_owned(),
                    id: resolved.expect("a non-empty default selector always resolves to an ID"),
                })
        })
        .transpose()?;
    let default_id = default.as_ref().map(|channel| channel.id.as_str());
    let channels = config
        .channels
        .iter()
        .map(|(alias, id)| ChannelEntry {
            alias: alias.clone(),
            id: id.clone(),
            is_default: default_id == Some(id.as_str()),
        })
        .collect();
    Ok(ChannelListing { default, channels })
}

fn print_channel_listing(listing: &ChannelListing) {
    match &listing.default {
        Some(default) => println!("Default: {} -> {}", default.selector, default.id),
        None => println!("Default: (unset)"),
    }
    for channel in &listing.channels {
        let marker = if channel.is_default { " [default]" } else { "" };
        println!("{}\t{}{}", channel.alias, channel.id, marker);
    }
}

#[derive(Debug, Serialize)]
struct AvatarListing {
    profiles: Vec<AvatarEntry>,
}

#[derive(Debug, Serialize)]
struct AvatarEntry {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    description: Option<String>,
    is_default: bool,
}

fn avatar_listing(config: &config::Config) -> AvatarListing {
    let profiles = config
        .avatars
        .iter()
        .map(|(name, profile)| AvatarEntry {
            name: name.clone(),
            kind: profile.avatar.kind().to_owned(),
            description: profile.description.clone(),
            is_default: config.defaults.avatar.as_deref() == Some(name.as_str()),
        })
        .collect();
    AvatarListing { profiles }
}

fn print_avatar_listing(listing: &AvatarListing) {
    if listing.profiles.is_empty() {
        println!("No configured avatar profiles.");
        return;
    }
    for profile in &listing.profiles {
        let marker = if profile.is_default { " [default]" } else { "" };
        match &profile.description {
            Some(description) => println!(
                "{}\t{}{}\t{}",
                profile.name, profile.kind, marker, description
            ),
            None => println!("{}\t{}{}", profile.name, profile.kind, marker),
        }
    }
}

async fn report_error(config_path: Option<PathBuf>, requested_channel: Option<&str>) -> Result<()> {
    let loaded = load_config(config_path)?;
    let payload = error_report_payload(runtime::current_agent_session_id().as_deref());
    let discord = DiscordClient::new()?;
    let result = deliver_error_report(&loaded, requested_channel, &payload, &discord).await?;
    match result.channel {
        Some(channel) => println!(
            "Reported agent notification failure to channel {channel} (message {})",
            result.message_id
        ),
        None => println!(
            "Reported agent notification failure through the default webhook (message {})",
            result.message_id
        ),
    }
    Ok(())
}

fn error_report_payload(codex_thread_id: Option<&str>) -> Value {
    let content = match codex_thread_id {
        Some(thread_id) => {
            format!("⚠️ Agent notification failed for thread `{thread_id}`.")
        }
        None => "⚠️ Agent notification failed.".to_owned(),
    };
    json!({
        "content": content,
        "allowed_mentions": { "parse": [] }
    })
}

fn error_report_candidates(
    config: &config::Config,
    requested_channel: Option<&str>,
) -> Result<Vec<Option<String>>> {
    let default_channel = config.resolve_channel(config.defaults.channel.as_deref())?;
    let mut candidates = Vec::with_capacity(2);

    if let Some(requested_channel) = requested_channel {
        if let Ok(Some(channel)) = config.resolve_channel(Some(requested_channel)) {
            candidates.push(Some(channel));
        }
        if (candidates.is_empty() || config.defaults.channel.is_some())
            && !candidates.contains(&default_channel)
        {
            candidates.push(default_channel);
        }
    } else {
        candidates.push(default_channel);
    }

    if candidates.is_empty() {
        candidates.push(None);
    }
    Ok(candidates)
}

#[derive(Debug)]
struct ErrorReportResult {
    message_id: String,
    channel: Option<String>,
}

async fn deliver_error_report(
    loaded: &LoadedConfig,
    requested_channel: Option<&str>,
    payload: &Value,
    discord: &DiscordClient,
) -> Result<ErrorReportResult> {
    let candidates = error_report_candidates(&loaded.config, requested_channel)?;
    let store = StateStore::new(loaded.data_directory.clone());
    let _lock = store.lock()?;
    let mut state = store.load()?;
    let mut failures = Vec::new();

    for channel in candidates {
        let attempt = async {
            let (webhook, provisioned) = discord
                .resolve_webhook(&loaded.config.discord, &mut state, channel.as_deref())
                .await?;
            if provisioned {
                store.save(&state)?;
            }
            discord.execute(&webhook, payload, None).await
        }
        .await;

        match attempt {
            Ok(message_id) => {
                return Ok(ErrorReportResult {
                    message_id,
                    channel,
                });
            }
            Err(error) => {
                let destination = channel
                    .as_deref()
                    .map(|value| format!("channel {value}"))
                    .unwrap_or_else(|| "the default webhook".to_owned());
                failures.push(format!("{destination}: {error:#}"));
            }
        }
    }

    bail!(
        "could not report the agent notification failure: {}",
        failures.join("; ")
    )
}

fn load_config(explicit: Option<PathBuf>) -> Result<LoadedConfig> {
    LoadedConfig::load(paths::discover_config(explicit)?)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs};

    use tempfile::TempDir;
    use url::Url;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, method, path, query_param},
    };

    use super::*;
    use crate::{
        config::{Config, DefaultsConfig, DiscordConfig, EmojiConfig, TemplatesConfig},
        state::AppState,
    };

    fn report_config(default_channel: Option<&str>) -> Config {
        Config {
            discord: DiscordConfig {
                webhook_url: None,
                bot_token: Some("bot-secret".to_owned()),
                webhook_name: "Notify Me".to_owned(),
            },
            templates: TemplatesConfig::default(),
            defaults: DefaultsConfig {
                template: "defaults".to_owned(),
                channel: default_channel.map(str::to_owned),
                username: None,
                avatar: None,
                avatar_url: None,
                thread_id: None,
                thread_name: None,
                tts: None,
            },
            channels: BTreeMap::from([
                ("requested".to_owned(), "111".to_owned()),
                ("default".to_owned(), "222".to_owned()),
                ("same".to_owned(), "222".to_owned()),
            ]),
            emoji: EmojiConfig::default(),
            avatars: BTreeMap::new(),
        }
    }

    #[test]
    fn error_report_candidates_fall_back_and_deduplicate() {
        let config = report_config(Some("default"));
        assert_eq!(
            error_report_candidates(&config, Some("requested")).unwrap(),
            vec![Some("111".to_owned()), Some("222".to_owned())]
        );
        assert_eq!(
            error_report_candidates(&config, Some("missing")).unwrap(),
            vec![Some("222".to_owned())]
        );
        assert_eq!(
            error_report_candidates(&config, Some("same")).unwrap(),
            vec![Some("222".to_owned())]
        );
        assert_eq!(
            error_report_candidates(&config, None).unwrap(),
            vec![Some("222".to_owned())]
        );
    }

    #[test]
    fn error_report_payload_is_short_and_secret_safe() {
        assert_eq!(
            error_report_payload(Some("thread-123")),
            json!({
                "content": "⚠️ Agent notification failed for thread `thread-123`.",
                "allowed_mentions": { "parse": [] }
            })
        );
        let without_thread = error_report_payload(None).to_string();
        assert!(without_thread.contains("Agent notification failed"));
        assert!(!without_thread.contains("stack"));
        assert!(!without_thread.contains("token"));
    }

    #[test]
    fn generated_avatar_username_preserves_explicit_identity() {
        let mut implicit = json!({ "content": "hello" });
        preserve_generated_avatar_username(&mut implicit, "Notify Me");
        assert_eq!(implicit["username"], "Notify Me");

        let mut explicit = json!({ "content": "hello", "username": "Release Bot" });
        preserve_generated_avatar_username(&mut explicit, "Notify Me");
        assert_eq!(explicit["username"], "Release Bot");
    }

    #[tokio::test]
    async fn failed_requested_delivery_uses_distinct_default() {
        let server = MockServer::start().await;
        let payload = error_report_payload(Some("thread-123"));
        Mock::given(method("POST"))
            .and(path("/api/v10/webhooks/111/requested-secret"))
            .and(query_param("wait", "true"))
            .and(body_json(payload.clone()))
            .respond_with(ResponseTemplate::new(500).set_body_string("requested failed"))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v10/webhooks/222/default-secret"))
            .and(query_param("wait", "true"))
            .and(body_json(payload.clone()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "999" })))
            .expect(1)
            .mount(&server)
            .await;

        let root = TempDir::new().unwrap();
        let data_directory = root.path().join("data");
        let store = StateStore::new(data_directory.clone());
        let mut state = AppState::default();
        state.provisioned_webhooks.insert(
            "111".to_owned(),
            format!("{}/api/v10/webhooks/111/requested-secret", server.uri()),
        );
        state.provisioned_webhooks.insert(
            "222".to_owned(),
            format!("{}/api/v10/webhooks/222/default-secret", server.uri()),
        );
        store.save(&state).unwrap();
        let loaded = LoadedConfig {
            config: report_config(Some("default")),
            path: root.path().join("config.toml"),
            directory: root.path().to_path_buf(),
            templates_directory: root.path().join("templates"),
            data_directory,
        };
        fs::create_dir_all(&loaded.templates_directory).unwrap();
        let client =
            DiscordClient::for_test(Url::parse(&format!("{}/api/v10/", server.uri())).unwrap());

        let result = deliver_error_report(&loaded, Some("requested"), &payload, &client)
            .await
            .unwrap();
        assert_eq!(result.message_id, "999");
        assert_eq!(result.channel.as_deref(), Some("222"));
    }

    #[tokio::test]
    async fn failed_default_report_stops_after_one_fallback() {
        let server = MockServer::start().await;
        let payload = error_report_payload(Some("thread-123"));
        for (webhook, token) in [("111", "requested-secret"), ("222", "default-secret")] {
            Mock::given(method("POST"))
                .and(path(format!("/api/v10/webhooks/{webhook}/{token}")))
                .and(query_param("wait", "true"))
                .and(body_json(payload.clone()))
                .respond_with(ResponseTemplate::new(500).set_body_string("delivery failed"))
                .expect(1)
                .mount(&server)
                .await;
        }

        let root = TempDir::new().unwrap();
        let data_directory = root.path().join("data");
        let store = StateStore::new(data_directory.clone());
        let mut state = AppState::default();
        state.provisioned_webhooks.insert(
            "111".to_owned(),
            format!("{}/api/v10/webhooks/111/requested-secret", server.uri()),
        );
        state.provisioned_webhooks.insert(
            "222".to_owned(),
            format!("{}/api/v10/webhooks/222/default-secret", server.uri()),
        );
        store.save(&state).unwrap();
        let loaded = LoadedConfig {
            config: report_config(Some("default")),
            path: root.path().join("config.toml"),
            directory: root.path().to_path_buf(),
            templates_directory: root.path().join("templates"),
            data_directory,
        };
        let client =
            DiscordClient::for_test(Url::parse(&format!("{}/api/v10/", server.uri())).unwrap());

        let error = deliver_error_report(&loaded, Some("requested"), &payload, &client)
            .await
            .unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("channel 111"));
        assert!(message.contains("channel 222"));
        assert!(message.contains("could not report the agent notification failure"));
    }
}
