use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::paths::{CONFIG_FILE, UserDirs};

pub const STARTER_CONFIG: &str = r##"# ping-me-in-discord configuration
#
[discord]
# Prefer a direct incoming webhook URL for the smallest possible credential:
# webhook_url = "https://discord.com/api/webhooks/WEBHOOK_ID/WEBHOOK_TOKEN"
#
# Or let a Bot with MANAGE_WEBHOOKS create/reuse webhooks by channel.
# A Bot token is required for local, emoji, text, and font-icon avatars:
# bot_token = "YOUR_BOT_TOKEN"
webhook_name = "Notify Me"

[channels]
# alerts = "123456789012345678"
# releases = "234567890123456789"

[templates]
directory = "templates"

[defaults]
template = "defaults"
# channel = "alerts"
# username = "Ping Me"
# avatar = "letter"

[emoji]
asset_base_url = "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/72x72/"

[avatars.letter]
description = "General notification avatar"
type = "text"
text = "N"
foreground = "#FFFFFF"
background = "#5865F2"
size = 256
font_size = 150.0

[avatars.started]
description = "Agent work started"
type = "emoji"
emoji = "🚀"
background = "#FFFFFF"
size = 256
scale = 0.72

[avatars.progress]
description = "Agent work in progress"
type = "emoji"
emoji = "🔄"
background = "#3B88C3"
size = 256
scale = 0.72

[avatars.success]
description = "Agent work completed successfully"
type = "emoji"
emoji = "✅"
background = "#77B255"
size = 256
scale = 0.72

[avatars.needs-input]
description = "Agent needs user input"
type = "emoji"
emoji = "❓"
background = "#F1C40F"
size = 256
scale = 0.72

[avatars.warning]
description = "Agent warning"
type = "emoji"
emoji = "⚠️"
background = "#E67E22"
size = 256
scale = 0.72

[avatars.error]
description = "Agent work or verification failed"
type = "emoji"
emoji = "❌"
foreground = "#FFFFFF"
background = "#DD2E44"
size = 256
scale = 0.576
"##;

pub const STARTER_TEMPLATE: &str = r#"> **{% if runtime.hostname != "unknown-host" %}🏠 `{% if runtime.user != "unknown-user" %}{{ runtime.user }}@{% endif %}{{ runtime.hostname }}`   {% endif %}{% if runtime.project.name != "unknown-project" %}📦 `{{ runtime.project.name }}`   {% endif %}{% if runtime.session.title %}🧵 `{{ runtime.session.title }}`   {% elif runtime.session.id %}🧵 `{{ runtime.session.id }}`   {% endif %}{% if runtime.agent.name != "CLI" %}🤖 `{{ runtime.agent.name }}`   {% endif %}📅 `{{ runtime.timestamp.local }}`**
{{ message }}"#;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub templates: TemplatesConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub channels: BTreeMap<String, String>,
    #[serde(default)]
    pub emoji: EmojiConfig,
    #[serde(default)]
    pub avatars: BTreeMap<String, AvatarProfile>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscordConfig {
    pub webhook_url: Option<String>,
    pub bot_token: Option<String>,
    #[serde(default = "default_webhook_name")]
    pub webhook_name: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplatesConfig {
    #[serde(default = "default_templates_directory")]
    pub directory: PathBuf,
}

impl Default for TemplatesConfig {
    fn default() -> Self {
        Self {
            directory: default_templates_directory(),
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultsConfig {
    #[serde(default = "default_template_name")]
    pub template: String,
    pub channel: Option<String>,
    pub username: Option<String>,
    pub avatar: Option<String>,
    pub avatar_url: Option<String>,
    pub thread_id: Option<String>,
    pub thread_name: Option<String>,
    pub tts: Option<bool>,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            template: default_template_name(),
            channel: None,
            username: None,
            avatar: None,
            avatar_url: None,
            thread_id: None,
            thread_name: None,
            tts: None,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmojiConfig {
    #[serde(default = "default_emoji_asset_base_url")]
    pub asset_base_url: String,
}

impl Default for EmojiConfig {
    fn default() -> Self {
        Self {
            asset_base_url: default_emoji_asset_base_url(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum AvatarConfig {
    Image {
        source: String,
        #[serde(default = "default_avatar_size")]
        size: u32,
    },
    Emoji {
        emoji: String,
        background: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        foreground: Option<String>,
        #[serde(default = "default_avatar_size")]
        size: u32,
        #[serde(default = "default_emoji_scale")]
        scale: f32,
    },
    Text {
        text: String,
        foreground: String,
        background: String,
        font: Option<PathBuf>,
        #[serde(default = "default_avatar_size")]
        size: u32,
        #[serde(default = "default_font_size")]
        font_size: f32,
    },
    FontIcon {
        glyph: String,
        font: PathBuf,
        foreground: String,
        background: String,
        #[serde(default = "default_avatar_size")]
        size: u32,
        #[serde(default = "default_font_size")]
        font_size: f32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AvatarProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub avatar: AvatarConfig,
}

#[derive(Clone)]
pub struct LoadedConfig {
    pub config: Config,
    pub path: PathBuf,
    pub directory: PathBuf,
    pub templates_directory: PathBuf,
    pub data_directory: PathBuf,
}

impl LoadedConfig {
    pub fn load(path: PathBuf) -> Result<Self> {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("could not read configuration {}", path.display()))?;
        let mut config: Config = toml::from_str(&source)
            .with_context(|| format!("could not parse TOML configuration {}", path.display()))?;
        apply_environment_overrides(&mut config);
        config.validate(&path)?;

        let directory = path
            .parent()
            .map(Path::to_path_buf)
            .context("configuration path has no parent directory")?;
        let templates_directory = if config.templates.directory.is_absolute() {
            config.templates.directory.clone()
        } else {
            directory.join(&config.templates.directory)
        };

        Ok(Self {
            config,
            path,
            directory,
            templates_directory,
            data_directory: UserDirs::discover()?.data_dir,
        })
    }
}

impl Config {
    pub fn validate(&self, config_path: &Path) -> Result<()> {
        let name = self.discord.webhook_name.trim();
        ensure!(
            (1..=80).contains(&name.chars().count()),
            "discord.webhook_name must contain between 1 and 80 characters"
        );
        let lowercase = name.to_ascii_lowercase();
        ensure!(
            !lowercase.contains("discord") && !lowercase.contains("clyde"),
            "discord.webhook_name cannot contain `discord` or `clyde`"
        );

        if let Some(url) = nonempty(self.discord.webhook_url.as_deref()) {
            validate_https_url(url, "discord.webhook_url")?;
        }
        validate_template_selector(&self.defaults.template)?;
        self.resolve_channel(self.defaults.channel.as_deref())?;
        validate_optional_username(self.defaults.username.as_deref(), "defaults.username")?;
        if let Some(default_avatar) = &self.defaults.avatar {
            ensure!(
                self.avatars.contains_key(default_avatar),
                "defaults.avatar references unknown profile `{default_avatar}`"
            );
        }
        ensure!(
            self.defaults.avatar.is_none() || self.defaults.avatar_url.is_none(),
            "defaults cannot set both `avatar` and `avatar_url`"
        );
        if let Some(url) = nonempty(self.defaults.avatar_url.as_deref()) {
            validate_https_url(url, "defaults.avatar_url")?;
        }
        validate_optional_id(self.defaults.thread_id.as_deref(), "defaults.thread_id")?;
        validate_optional_thread_name(
            self.defaults.thread_name.as_deref(),
            "defaults.thread_name",
        )?;
        ensure!(
            self.defaults.thread_id.is_none() || self.defaults.thread_name.is_none(),
            "defaults cannot set both `thread_id` and `thread_name`"
        );

        for (alias, channel_id) in &self.channels {
            ensure!(
                !alias.is_empty()
                    && alias.chars().all(|character| {
                        character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
                    })
                    && !alias.chars().all(|character| character.is_ascii_digit()),
                "channel alias `{alias}` must contain an ASCII letter and may use letters, digits, `-`, and `_`"
            );
            validate_channel_id(channel_id)
                .with_context(|| format!("invalid channel ID for alias `{alias}`"))?;
        }

        let config_directory = config_path.parent().unwrap_or_else(|| Path::new("."));
        for (name, profile) in &self.avatars {
            ensure!(
                !name.trim().is_empty(),
                "avatar profile names cannot be empty"
            );
            profile
                .validate(config_directory)
                .with_context(|| format!("invalid avatar profile `{name}`"))?;
        }

        let emoji_base = Url::parse(&self.emoji.asset_base_url)
            .context("emoji.asset_base_url must be a valid URL")?;
        ensure!(
            emoji_base.scheme() == "https",
            "emoji.asset_base_url must use HTTPS"
        );
        Ok(())
    }

    pub fn resolve_channel(&self, selector: Option<&str>) -> Result<Option<String>> {
        let Some(selector) = nonempty(selector) else {
            return Ok(None);
        };
        if let Some(channel_id) = self.channels.get(selector) {
            return Ok(Some(channel_id.clone()));
        }
        if validate_channel_id(selector).is_ok() {
            return Ok(Some(selector.to_owned()));
        }
        anyhow::bail!(
            "unknown channel alias `{selector}`; define it under `[channels]` or use a numeric Discord channel ID"
        )
    }
}

impl AvatarConfig {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Image { .. } => "image",
            Self::Emoji { .. } => "emoji",
            Self::Text { .. } => "text",
            Self::FontIcon { .. } => "font-icon",
        }
    }

    pub fn validate(&self, config_directory: &Path) -> Result<()> {
        match self {
            Self::Image { source, size } => {
                validate_size(*size)?;
                if source.starts_with("http://") || source.starts_with("https://") {
                    validate_https_url(source, "image source")?;
                } else {
                    let path = resolve_path(config_directory, Path::new(source));
                    ensure!(
                        path.is_file(),
                        "image file does not exist: {}",
                        path.display()
                    );
                }
            }
            Self::Emoji {
                emoji,
                background,
                foreground,
                size,
                scale,
            } => {
                ensure!(!emoji.trim().is_empty(), "emoji cannot be empty");
                validate_color(background)?;
                if let Some(foreground) = foreground {
                    validate_color(foreground)?;
                }
                validate_size(*size)?;
                ensure!(
                    (0.1..=1.0).contains(scale),
                    "emoji scale must be between 0.1 and 1.0"
                );
            }
            Self::Text {
                text,
                foreground,
                background,
                font,
                size,
                font_size,
            } => {
                ensure!(!text.is_empty(), "text cannot be empty");
                ensure!(
                    text.chars().count() <= 8,
                    "text avatars support at most 8 characters"
                );
                validate_text_settings(
                    config_directory,
                    font.as_deref(),
                    foreground,
                    background,
                    *size,
                    *font_size,
                )?;
            }
            Self::FontIcon {
                glyph,
                font,
                foreground,
                background,
                size,
                font_size,
            } => {
                ensure!(!glyph.trim().is_empty(), "glyph cannot be empty");
                validate_text_settings(
                    config_directory,
                    Some(font),
                    foreground,
                    background,
                    *size,
                    *font_size,
                )?;
            }
        }
        Ok(())
    }
}

impl AvatarProfile {
    pub fn validate(&self, config_directory: &Path) -> Result<()> {
        if let Some(description) = &self.description {
            ensure!(
                !description.trim().is_empty(),
                "description cannot be empty"
            );
            ensure!(
                description.chars().count() <= 200,
                "description cannot exceed 200 characters"
            );
        }
        self.avatar.validate(config_directory)
    }
}

impl From<AvatarConfig> for AvatarProfile {
    fn from(avatar: AvatarConfig) -> Self {
        Self {
            description: None,
            avatar,
        }
    }
}

pub fn initialize(directory: &Path, force: bool) -> Result<(PathBuf, PathBuf)> {
    let config_path = directory.join(CONFIG_FILE);
    let template_path = directory.join("templates/defaults.md");
    if !force {
        ensure!(
            !config_path.exists() && !template_path.exists(),
            "refusing to overwrite existing {} or {} (pass --force to replace them)",
            config_path.display(),
            template_path.display()
        );
    }

    fs::create_dir_all(
        template_path
            .parent()
            .context("starter template path has no parent")?,
    )
    .with_context(|| format!("could not create {}", directory.display()))?;
    write_private(&config_path, STARTER_CONFIG, force)?;
    write_private(&template_path, STARTER_TEMPLATE, force)?;
    Ok((config_path, template_path))
}

pub fn redact(input: &str, secrets: &[&str]) -> String {
    secrets
        .iter()
        .filter(|secret| !secret.is_empty())
        .fold(input.to_owned(), |text, secret| {
            text.replace(secret, "<redacted>")
        })
}

pub fn validate_template_name(name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "template name cannot be empty");
    ensure!(
        name.chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_')),
        "template name may contain only ASCII letters, digits, `-`, and `_`"
    );
    Ok(())
}

pub fn validate_template_selector(selector: &str) -> Result<()> {
    let path = Path::new(selector);
    if path.is_absolute() {
        ensure!(
            !path
                .components()
                .any(|component| matches!(component, Component::ParentDir)),
            "absolute template path cannot contain parent-directory components"
        );
        ensure!(
            path.extension().and_then(|extension| extension.to_str()) == Some("md"),
            "absolute template path must end in `.md`"
        );
        return Ok(());
    }

    validate_template_name(selector)
}

fn apply_environment_overrides(config: &mut Config) {
    if let Some(value) = std::env::var_os("DISCORD_NOTIFICATION_WEBHOOK_URL") {
        config.discord.webhook_url = Some(value.to_string_lossy().into_owned());
    }
    if let Some(value) = std::env::var_os("DISCORD_NOTIFICATION_BOT_TOKEN") {
        config.discord.bot_token = Some(value.to_string_lossy().into_owned());
    }
}

fn validate_text_settings(
    config_directory: &Path,
    font: Option<&Path>,
    foreground: &str,
    background: &str,
    size: u32,
    font_size: f32,
) -> Result<()> {
    validate_color(foreground)?;
    validate_color(background)?;
    validate_size(size)?;
    ensure!(
        font_size.is_finite() && font_size > 0.0 && font_size <= size as f32 * 2.0,
        "font_size must be positive and at most twice the avatar size"
    );
    if let Some(font) = font {
        let path = resolve_path(config_directory, font);
        ensure!(
            path.is_file(),
            "font file does not exist: {}",
            path.display()
        );
    }
    Ok(())
}

fn validate_size(size: u32) -> Result<()> {
    ensure!(
        (64..=1024).contains(&size),
        "avatar size must be between 64 and 1024 pixels"
    );
    Ok(())
}

fn validate_color(color: &str) -> Result<()> {
    let digits = color.strip_prefix('#').unwrap_or(color);
    ensure!(
        matches!(digits.len(), 6 | 8)
            && digits
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
        "color `{color}` must be #RRGGBB or #RRGGBBAA"
    );
    Ok(())
}

fn validate_https_url(value: &str, field: &str) -> Result<()> {
    let url = Url::parse(value).with_context(|| format!("{field} must be a valid URL"))?;
    ensure!(url.scheme() == "https", "{field} must use HTTPS");
    ensure!(url.host_str().is_some(), "{field} must include a host");
    Ok(())
}

fn validate_channel_id(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.chars().all(|character| character.is_ascii_digit()),
        "Discord channel ID must contain only digits"
    );
    Ok(())
}

fn validate_optional_id(value: Option<&str>, field: &str) -> Result<()> {
    if let Some(value) = nonempty(value) {
        ensure!(
            value.chars().all(|character| character.is_ascii_digit()),
            "{field} must contain only digits"
        );
    }
    Ok(())
}

fn validate_optional_username(value: Option<&str>, field: &str) -> Result<()> {
    if let Some(value) = value {
        ensure!(
            (1..=80).contains(&value.chars().count()),
            "{field} must contain between 1 and 80 characters"
        );
    }
    Ok(())
}

fn validate_optional_thread_name(value: Option<&str>, field: &str) -> Result<()> {
    if let Some(value) = value {
        ensure!(
            (1..=100).contains(&value.chars().count()),
            "{field} must contain between 1 and 100 characters"
        );
    }
    Ok(())
}

pub fn resolve_path(directory: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        directory.join(path)
    }
}

fn write_private(path: &Path, contents: &str, force: bool) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .with_context(|| format!("could not create {}", path.display()))?;
    file.write_all(contents.as_bytes())
        .with_context(|| format!("could not write {}", path.display()))
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.trim().is_empty())
}

fn default_webhook_name() -> String {
    "Notify Me".to_owned()
}

fn default_templates_directory() -> PathBuf {
    PathBuf::from("templates")
}

fn default_template_name() -> String {
    "defaults".to_owned()
}

fn default_emoji_asset_base_url() -> String {
    "https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/72x72/".to_owned()
}

fn default_avatar_size() -> u32 {
    256
}

fn default_emoji_scale() -> f32 {
    0.72
}

fn default_font_size() -> f32 {
    150.0
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn starter_configuration_parses() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("templates")).unwrap();
        fs::write(root.path().join("templates/defaults.md"), STARTER_TEMPLATE).unwrap();
        let config: Config = toml::from_str(STARTER_CONFIG).unwrap();
        config.validate(&root.path().join("config.toml")).unwrap();
        let profile = config.avatars.get("letter").unwrap();
        assert_eq!(
            profile.description.as_deref(),
            Some("General notification avatar")
        );
        assert_eq!(profile.avatar.kind(), "text");
    }

    #[test]
    fn starter_configuration_contains_strict_status_profiles() {
        let config: Config = toml::from_str(STARTER_CONFIG).unwrap();
        let expected = [
            ("started", "🚀", "#FFFFFF", None, 0.72),
            ("progress", "🔄", "#3B88C3", None, 0.72),
            ("success", "✅", "#77B255", None, 0.72),
            ("needs-input", "❓", "#F1C40F", None, 0.72),
            ("warning", "⚠️", "#E67E22", None, 0.72),
            ("error", "❌", "#DD2E44", Some("#FFFFFF"), 0.576),
        ];

        for (name, expected_emoji, expected_background, expected_foreground, expected_scale) in
            expected
        {
            let AvatarConfig::Emoji {
                emoji,
                background,
                foreground,
                size,
                scale,
            } = &config.avatars[name].avatar
            else {
                panic!("strict status profile `{name}` must be an emoji");
            };
            assert_eq!(emoji, expected_emoji);
            assert_eq!(background, expected_background);
            assert_eq!(foreground.as_deref(), expected_foreground);
            assert_eq!(*size, 256);
            assert_eq!(*scale, expected_scale);
        }
    }

    #[test]
    fn avatar_descriptions_are_optional_but_validated() {
        let root = TempDir::new().unwrap();
        let legacy_source =
            STARTER_CONFIG.replace("description = \"General notification avatar\"\n", "");
        let legacy: Config = toml::from_str(&legacy_source).unwrap();
        legacy.validate(&root.path().join("config.toml")).unwrap();
        assert!(legacy.avatars["letter"].description.is_none());

        let mut invalid: Config = toml::from_str(STARTER_CONFIG).unwrap();
        invalid.avatars.get_mut("letter").unwrap().description = Some(" \n ".to_owned());
        let error = invalid
            .validate(&root.path().join("config.toml"))
            .unwrap_err();
        assert!(format!("{error:#}").contains("description cannot be empty"));

        invalid.avatars.get_mut("letter").unwrap().description = Some("x".repeat(201));
        let error = invalid
            .validate(&root.path().join("config.toml"))
            .unwrap_err();
        assert!(format!("{error:#}").contains("description cannot exceed 200 characters"));
    }

    #[test]
    fn emoji_foregrounds_are_optional_but_validated() {
        let root = TempDir::new().unwrap();
        let mut config: Config = toml::from_str(STARTER_CONFIG).unwrap();
        config.validate(&root.path().join("config.toml")).unwrap();
        assert!(matches!(
            config.avatars["started"].avatar,
            AvatarConfig::Emoji {
                foreground: None,
                ..
            }
        ));
        assert!(matches!(
            config.avatars["error"].avatar,
            AvatarConfig::Emoji {
                foreground: Some(ref foreground),
                ..
            } if foreground == "#FFFFFF"
        ));

        let AvatarConfig::Emoji { foreground, .. } =
            &mut config.avatars.get_mut("error").unwrap().avatar
        else {
            unreachable!();
        };
        *foreground = Some("not-a-color".to_owned());
        let error = config
            .validate(&root.path().join("config.toml"))
            .unwrap_err();
        assert!(format!("{error:#}").contains("must be #RRGGBB or #RRGGBBAA"));
    }

    #[test]
    fn starter_template_matches_the_approved_context_header_layout() {
        assert_eq!(
            STARTER_TEMPLATE,
            r#"> **{% if runtime.hostname != "unknown-host" %}🏠 `{% if runtime.user != "unknown-user" %}{{ runtime.user }}@{% endif %}{{ runtime.hostname }}`   {% endif %}{% if runtime.project.name != "unknown-project" %}📦 `{{ runtime.project.name }}`   {% endif %}{% if runtime.session.title %}🧵 `{{ runtime.session.title }}`   {% elif runtime.session.id %}🧵 `{{ runtime.session.id }}`   {% endif %}{% if runtime.agent.name != "CLI" %}🤖 `{{ runtime.agent.name }}`   {% endif %}📅 `{{ runtime.timestamp.local }}`**
{{ message }}"#
        );
        assert_eq!(
            include_str!("../examples/templates/defaults.md"),
            format!("{STARTER_TEMPLATE}\n")
        );
    }

    #[test]
    fn unknown_or_misplaced_configuration_fields_are_rejected() {
        let error = toml::from_str::<Config>(
            r#"
webhook_url = "https://discord.com/api/webhooks/id/token"

[discord]
webhook_name = "Notify Me"
"#,
        )
        .err()
        .expect("root-level webhook_url should be rejected")
        .to_string();

        assert!(error.contains("unknown field `webhook_url`"));
    }

    #[test]
    fn resolves_channel_aliases_and_numeric_ids() {
        let mut config: Config = toml::from_str(STARTER_CONFIG).unwrap();
        config
            .channels
            .insert("alerts".to_owned(), "123456789".to_owned());

        assert_eq!(
            config.resolve_channel(Some("alerts")).unwrap().as_deref(),
            Some("123456789")
        );
        assert_eq!(
            config
                .resolve_channel(Some("987654321"))
                .unwrap()
                .as_deref(),
            Some("987654321")
        );
        assert!(
            config
                .resolve_channel(Some("missing"))
                .unwrap_err()
                .to_string()
                .contains("unknown channel alias")
        );
    }

    #[test]
    fn initialization_does_not_overwrite() {
        let root = TempDir::new().unwrap();
        initialize(root.path(), false).unwrap();
        fs::write(
            root.path().join("templates/defaults.md"),
            "custom user template",
        )
        .unwrap();
        let error = initialize(root.path(), false).unwrap_err().to_string();
        assert!(error.contains("refusing to overwrite"));
        assert_eq!(
            fs::read_to_string(root.path().join("templates/defaults.md")).unwrap(),
            "custom user template"
        );
    }

    #[test]
    fn redacts_multiple_secrets() {
        assert_eq!(
            redact(
                "url=secret-url token=secret-token",
                &["secret-url", "secret-token"]
            ),
            "url=<redacted> token=<redacted>"
        );
    }
}
