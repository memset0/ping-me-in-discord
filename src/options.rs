use std::env;

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value};
use url::Url;

use crate::{
    avatar::AvatarSelection,
    cli::SendOptions,
    config::{AvatarConfig, Config},
    template::RenderedMessage,
};

const DEFAULT_AVATAR_BACKGROUND: &str = "#5865F2";
const DEFAULT_AVATAR_FOREGROUND: &str = "#FFFFFF";
const DEFAULT_AVATAR_SIZE: u32 = 256;
const DEFAULT_AVATAR_FONT_SIZE: f32 = 150.0;
const DEFAULT_AVATAR_SCALE: f32 = 0.72;

pub fn resolve(
    mut rendered: RenderedMessage,
    arguments: &SendOptions,
    config: &Config,
) -> Result<RenderedMessage> {
    let channel_selector = arguments
        .channel
        .as_deref()
        .or(rendered.channel.as_deref())
        .or(config.defaults.channel.as_deref())
        .map(str::to_owned);
    rendered.channel = config.resolve_channel(channel_selector.as_deref())?;

    resolve_thread(&mut rendered, arguments, config)?;
    let payload = rendered
        .payload
        .as_object_mut()
        .expect("rendered payload is always an object");
    resolve_payload_options(payload, arguments, config)?;
    resolve_avatar(&mut rendered.avatar, payload, arguments, config)?;
    Ok(rendered)
}

fn resolve_payload_options(
    payload: &mut Map<String, Value>,
    arguments: &SendOptions,
    config: &Config,
) -> Result<()> {
    let template_username = payload
        .get("username")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let username = arguments
        .username
        .clone()
        .or(template_username)
        .or_else(|| config.defaults.username.clone());
    if let Some(username) = &username {
        ensure!(
            (1..=80).contains(&username.chars().count()),
            "username must contain between 1 and 80 characters"
        );
    }
    set_optional_string(payload, "username", username);

    let command_tts = if arguments.tts {
        Some(true)
    } else if arguments.no_tts {
        Some(false)
    } else {
        None
    };
    let template_tts = payload.get("tts").and_then(Value::as_bool);
    match command_tts.or(template_tts).or(config.defaults.tts) {
        Some(tts) => {
            payload.insert("tts".to_owned(), Value::Bool(tts));
        }
        None => {
            payload.remove("tts");
        }
    }
    Ok(())
}

fn resolve_thread(
    rendered: &mut RenderedMessage,
    arguments: &SendOptions,
    config: &Config,
) -> Result<()> {
    let payload = rendered
        .payload
        .as_object_mut()
        .expect("rendered payload is always an object");
    let template_thread_id = rendered.thread_id.take();
    let template_thread_name = payload
        .remove("thread_name")
        .and_then(|value| value.as_str().map(str::to_owned));
    ensure!(
        template_thread_id.is_none() || template_thread_name.is_none(),
        "template cannot set both `thread_id` and `thread_name`"
    );

    let (thread_id, thread_name) =
        if arguments.thread_id.is_some() || arguments.thread_name.is_some() {
            (arguments.thread_id.clone(), arguments.thread_name.clone())
        } else if template_thread_id.is_some() || template_thread_name.is_some() {
            (template_thread_id, template_thread_name)
        } else {
            (
                config.defaults.thread_id.clone(),
                config.defaults.thread_name.clone(),
            )
        };
    ensure!(
        thread_id.is_none() || thread_name.is_none(),
        "thread_id and thread_name cannot be used together"
    );

    if let Some(thread_id) = &thread_id {
        ensure_numeric_id(thread_id, "thread ID")?;
    }
    if let Some(thread_name) = &thread_name {
        ensure!(
            (1..=100).contains(&thread_name.chars().count()),
            "thread name must contain between 1 and 100 characters"
        );
        payload.insert("thread_name".to_owned(), Value::String(thread_name.clone()));
    }
    rendered.thread_id = thread_id;
    Ok(())
}

fn resolve_avatar(
    template_avatar: &mut Option<AvatarSelection>,
    payload: &mut Map<String, Value>,
    arguments: &SendOptions,
    config: &Config,
) -> Result<()> {
    let source_count = [
        arguments.avatar.is_some(),
        arguments.avatar_url.is_some(),
        arguments.avatar_file.is_some(),
        arguments.avatar_emoji.is_some(),
        arguments.avatar_text.is_some(),
        arguments.avatar_icon.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    ensure!(
        source_count <= 1,
        "avatar source arguments are mutually exclusive"
    );

    if source_count == 0 {
        ensure!(
            !has_avatar_modifiers(arguments),
            "avatar styling arguments require --avatar-file, --avatar-emoji, --avatar-text, or --avatar-icon"
        );
        if template_avatar.is_none() && !payload.contains_key("avatar_url") {
            if let Some(profile) = &config.defaults.avatar {
                *template_avatar = Some(AvatarSelection::Profile {
                    name: profile.clone(),
                });
            } else if let Some(url) = &config.defaults.avatar_url {
                validate_https_url(url, "defaults.avatar_url")?;
                payload.insert("avatar_url".to_owned(), Value::String(url.clone()));
            }
        }
        return validate_avatar_selection(template_avatar, config);
    }

    payload.remove("avatar_url");
    *template_avatar = None;

    if let Some(profile) = &arguments.avatar {
        ensure!(
            !has_avatar_modifiers(arguments),
            "--avatar selects a complete profile and cannot be combined with avatar styling arguments"
        );
        *template_avatar = Some(AvatarSelection::Profile {
            name: profile.clone(),
        });
        return validate_avatar_selection(template_avatar, config);
    }

    if let Some(url) = &arguments.avatar_url {
        ensure!(
            !has_avatar_modifiers(arguments),
            "--avatar-url cannot be combined with avatar styling arguments"
        );
        validate_https_url(url, "--avatar-url")?;
        payload.insert("avatar_url".to_owned(), Value::String(url.clone()));
        return Ok(());
    }

    let base_directory = env::current_dir().context("could not determine the current directory")?;
    let profile = inline_avatar(arguments)?;
    profile.validate(&base_directory)?;
    *template_avatar = Some(AvatarSelection::Inline {
        profile,
        base_directory,
    });
    Ok(())
}

fn inline_avatar(arguments: &SendOptions) -> Result<AvatarConfig> {
    let size = arguments.avatar_size.unwrap_or(DEFAULT_AVATAR_SIZE);
    if let Some(path) = &arguments.avatar_file {
        reject_options(
            arguments,
            &["background", "foreground", "font", "font-size", "scale"],
        )?;
        return Ok(AvatarConfig::Image {
            source: path.to_string_lossy().into_owned(),
            size,
        });
    }
    if let Some(emoji) = &arguments.avatar_emoji {
        reject_options(arguments, &["foreground", "font", "font-size"])?;
        return Ok(AvatarConfig::Emoji {
            emoji: emoji.clone(),
            background: arguments
                .avatar_background
                .clone()
                .unwrap_or_else(|| DEFAULT_AVATAR_BACKGROUND.to_owned()),
            size,
            scale: arguments.avatar_scale.unwrap_or(DEFAULT_AVATAR_SCALE),
        });
    }
    if let Some(text) = &arguments.avatar_text {
        reject_options(arguments, &["scale"])?;
        return Ok(AvatarConfig::Text {
            text: text.clone(),
            foreground: arguments
                .avatar_foreground
                .clone()
                .unwrap_or_else(|| DEFAULT_AVATAR_FOREGROUND.to_owned()),
            background: arguments
                .avatar_background
                .clone()
                .unwrap_or_else(|| DEFAULT_AVATAR_BACKGROUND.to_owned()),
            font: arguments.avatar_font.clone(),
            size,
            font_size: arguments
                .avatar_font_size
                .unwrap_or(DEFAULT_AVATAR_FONT_SIZE),
        });
    }
    if let Some(glyph) = &arguments.avatar_icon {
        reject_options(arguments, &["scale"])?;
        let font = arguments
            .avatar_font
            .clone()
            .context("--avatar-icon requires --avatar-font")?;
        return Ok(AvatarConfig::FontIcon {
            glyph: glyph.clone(),
            font,
            foreground: arguments
                .avatar_foreground
                .clone()
                .unwrap_or_else(|| DEFAULT_AVATAR_FOREGROUND.to_owned()),
            background: arguments
                .avatar_background
                .clone()
                .unwrap_or_else(|| DEFAULT_AVATAR_BACKGROUND.to_owned()),
            size,
            font_size: arguments
                .avatar_font_size
                .unwrap_or(DEFAULT_AVATAR_FONT_SIZE),
        });
    }
    bail!("an inline avatar source is required")
}

fn reject_options(arguments: &SendOptions, names: &[&str]) -> Result<()> {
    for name in names {
        let is_set = match *name {
            "background" => arguments.avatar_background.is_some(),
            "foreground" => arguments.avatar_foreground.is_some(),
            "font" => arguments.avatar_font.is_some(),
            "font-size" => arguments.avatar_font_size.is_some(),
            "scale" => arguments.avatar_scale.is_some(),
            _ => false,
        };
        ensure!(
            !is_set,
            "--avatar-{name} is not valid for the selected avatar source"
        );
    }
    Ok(())
}

fn has_avatar_modifiers(arguments: &SendOptions) -> bool {
    arguments.avatar_background.is_some()
        || arguments.avatar_foreground.is_some()
        || arguments.avatar_font.is_some()
        || arguments.avatar_size.is_some()
        || arguments.avatar_font_size.is_some()
        || arguments.avatar_scale.is_some()
}

fn validate_avatar_selection(selection: &Option<AvatarSelection>, config: &Config) -> Result<()> {
    if let Some(AvatarSelection::Profile { name }) = selection {
        ensure!(
            config.avatars.contains_key(name),
            "unknown avatar profile `{name}`"
        );
    }
    Ok(())
}

fn validate_https_url(value: &str, field: &str) -> Result<()> {
    let url = Url::parse(value).with_context(|| format!("{field} must be a valid URL"))?;
    ensure!(url.scheme() == "https", "{field} must use HTTPS");
    ensure!(url.host_str().is_some(), "{field} must include a host");
    Ok(())
}

fn ensure_numeric_id(value: &str, field: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.chars().all(|character| character.is_ascii_digit()),
        "{field} must contain only digits"
    );
    Ok(())
}

fn set_optional_string(payload: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        payload.insert(key.to_owned(), Value::String(value));
    } else {
        payload.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::{
        config::{DefaultsConfig, DiscordConfig, EmojiConfig, TemplatesConfig},
        template::RenderedMessage,
    };

    fn configuration() -> Config {
        Config {
            discord: DiscordConfig::default(),
            templates: TemplatesConfig::default(),
            defaults: DefaultsConfig {
                template: "defaults".to_owned(),
                channel: Some("settings".to_owned()),
                username: Some("Settings User".to_owned()),
                avatar: Some("settings-avatar".to_owned()),
                avatar_url: None,
                thread_id: Some("400".to_owned()),
                thread_name: None,
                tts: Some(true),
            },
            channels: BTreeMap::from([
                ("settings".to_owned(), "100".to_owned()),
                ("template".to_owned(), "200".to_owned()),
                ("command".to_owned(), "300".to_owned()),
            ]),
            emoji: EmojiConfig::default(),
            avatars: BTreeMap::from([
                (
                    "settings-avatar".to_owned(),
                    AvatarConfig::Text {
                        text: "S".to_owned(),
                        foreground: "#FFFFFF".to_owned(),
                        background: "#5865F2".to_owned(),
                        font: None,
                        size: 256,
                        font_size: 150.0,
                    },
                ),
                (
                    "template-avatar".to_owned(),
                    AvatarConfig::Text {
                        text: "T".to_owned(),
                        foreground: "#FFFFFF".to_owned(),
                        background: "#5865F2".to_owned(),
                        font: None,
                        size: 256,
                        font_size: 150.0,
                    },
                ),
                (
                    "command-avatar".to_owned(),
                    AvatarConfig::Text {
                        text: "C".to_owned(),
                        foreground: "#FFFFFF".to_owned(),
                        background: "#5865F2".to_owned(),
                        font: None,
                        size: 256,
                        font_size: 150.0,
                    },
                ),
            ]),
        }
    }

    fn rendered() -> RenderedMessage {
        RenderedMessage {
            template: "defaults".to_owned(),
            payload: json!({
                "content": "hello",
                "username": "Template User",
                "thread_name": "Template Thread",
                "tts": true
            }),
            channel: Some("template".to_owned()),
            avatar: Some(AvatarSelection::Profile {
                name: "template-avatar".to_owned(),
            }),
            thread_id: None,
        }
    }

    #[test]
    fn command_arguments_override_frontmatter_and_settings() {
        let arguments = SendOptions {
            channel: Some("command".to_owned()),
            username: Some("Command User".to_owned()),
            avatar: Some("command-avatar".to_owned()),
            thread_id: Some("600".to_owned()),
            no_tts: true,
            ..SendOptions::default()
        };

        let resolved = resolve(rendered(), &arguments, &configuration()).unwrap();
        assert_eq!(resolved.channel.as_deref(), Some("300"));
        assert_eq!(resolved.thread_id.as_deref(), Some("600"));
        assert_eq!(resolved.payload["username"], "Command User");
        assert!(resolved.payload.get("thread_name").is_none());
        assert_eq!(resolved.payload["tts"], false);
        assert!(matches!(
            resolved.avatar,
            Some(AvatarSelection::Profile { ref name }) if name == "command-avatar"
        ));
    }

    #[test]
    fn frontmatter_overrides_settings_when_cli_is_absent() {
        let resolved = resolve(rendered(), &SendOptions::default(), &configuration()).unwrap();
        assert_eq!(resolved.channel.as_deref(), Some("200"));
        assert!(resolved.thread_id.is_none());
        assert_eq!(resolved.payload["username"], "Template User");
        assert_eq!(resolved.payload["thread_name"], "Template Thread");
        assert_eq!(resolved.payload["tts"], true);
        assert!(matches!(
            resolved.avatar,
            Some(AvatarSelection::Profile { ref name }) if name == "template-avatar"
        ));
    }

    #[test]
    fn settings_apply_when_cli_and_frontmatter_are_absent() {
        let message = RenderedMessage {
            template: "defaults".to_owned(),
            payload: json!({"content": "hello"}),
            channel: None,
            avatar: None,
            thread_id: None,
        };
        let resolved = resolve(message, &SendOptions::default(), &configuration()).unwrap();
        assert_eq!(resolved.channel.as_deref(), Some("100"));
        assert_eq!(resolved.payload["username"], "Settings User");
        assert!(matches!(
            resolved.avatar,
            Some(AvatarSelection::Profile { ref name }) if name == "settings-avatar"
        ));
    }

    #[test]
    fn no_avatar_remains_unset_when_all_layers_omit_it() {
        let mut config = configuration();
        config.defaults.avatar = None;
        let mut message = rendered();
        message.avatar = None;
        message
            .payload
            .as_object_mut()
            .unwrap()
            .remove("avatar_url");

        let resolved = resolve(message, &SendOptions::default(), &config).unwrap();
        assert!(resolved.avatar.is_none());
        assert!(resolved.payload.get("avatar_url").is_none());
    }

    #[test]
    fn one_off_emoji_is_atomic_and_uses_documented_defaults() {
        let arguments = SendOptions {
            avatar_emoji: Some("🚀".to_owned()),
            ..SendOptions::default()
        };
        let resolved = resolve(rendered(), &arguments, &configuration()).unwrap();
        assert!(matches!(
            resolved.avatar,
            Some(AvatarSelection::Inline {
                profile: AvatarConfig::Emoji {
                    ref emoji,
                    ref background,
                    size: 256,
                    scale,
                },
                ..
            }) if emoji == "🚀" && background == "#5865F2" && scale == 0.72
        ));
        assert!(resolved.payload.get("avatar_url").is_none());
    }

    #[test]
    fn unknown_channel_alias_is_rejected() {
        let arguments = SendOptions {
            channel: Some("missing".to_owned()),
            ..SendOptions::default()
        };
        let error = resolve(rendered(), &arguments, &configuration())
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown channel alias `missing`"));
    }
}
