use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Send templated messages to Discord",
    long_about = None,
    subcommand_precedence_over_arg = true
)]
pub struct Cli {
    /// Use a specific config.toml instead of automatic discovery.
    #[arg(long, global = true, env = "DISCORD_NOTIFICATION_CONFIG")]
    pub config: Option<PathBuf>,

    /// Send this message through the selected or default template.
    #[arg(value_name = "MESSAGE")]
    pub message: Option<String>,

    #[command(flatten)]
    pub quick: SendOptions,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Render and send a message template.
    Send(Box<SendArgs>),
    /// Create starter configuration and templates.
    Init(InitArgs),
    /// Inspect or validate configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Inspect available templates.
    Templates {
        #[command(subcommand)]
        command: TemplatesCommand,
    },
    /// Render configured avatars.
    Avatar {
        #[command(subcommand)]
        command: AvatarCommand,
    },
    /// Manage the Discord incoming webhook.
    Webhook {
        #[command(subcommand)]
        command: WebhookCommand,
    },
}

#[derive(Debug, Args)]
pub struct SendArgs {
    /// Value exposed to the template as `message`.
    #[arg(value_name = "MESSAGE")]
    pub message: Option<String>,

    #[command(flatten)]
    pub options: SendOptions,
}

#[derive(Debug, Args, Default)]
pub struct SendOptions {
    /// Template name without the .md extension.
    #[arg(long)]
    pub template: Option<String>,

    /// Discord channel ID or alias from the [channels] configuration table.
    #[arg(long, value_name = "CHANNEL")]
    pub channel: Option<String>,

    /// Override the display username for this webhook message.
    #[arg(long, value_name = "NAME")]
    pub username: Option<String>,

    /// Select a named profile from [avatars.<name>].
    #[arg(long, value_name = "PROFILE", group = "avatar_source")]
    pub avatar: Option<String>,

    /// Use an HTTPS image URL as a one-off avatar.
    #[arg(long, value_name = "URL", group = "avatar_source")]
    pub avatar_url: Option<String>,

    /// Use a local image as a one-off avatar.
    #[arg(long, value_name = "PATH", group = "avatar_source")]
    pub avatar_file: Option<PathBuf>,

    /// Render one emoji as a one-off avatar.
    #[arg(long, value_name = "EMOJI", group = "avatar_source")]
    pub avatar_emoji: Option<String>,

    /// Render short Unicode text as a one-off avatar.
    #[arg(long, value_name = "TEXT", group = "avatar_source")]
    pub avatar_text: Option<String>,

    /// Render a glyph or Unicode code point from --avatar-font.
    #[arg(long, value_name = "GLYPH", group = "avatar_source")]
    pub avatar_icon: Option<String>,

    /// Set the background color for a one-off emoji, text, or icon avatar.
    #[arg(long, value_name = "COLOR")]
    pub avatar_background: Option<String>,

    /// Set the foreground color for a one-off text or icon avatar.
    #[arg(long, value_name = "COLOR")]
    pub avatar_foreground: Option<String>,

    /// Use this font for a one-off text or icon avatar.
    #[arg(long, value_name = "PATH")]
    pub avatar_font: Option<PathBuf>,

    /// Set the square output size for a one-off rendered avatar.
    #[arg(long, value_name = "PIXELS")]
    pub avatar_size: Option<u32>,

    /// Set the font size for a one-off text or icon avatar.
    #[arg(long, value_name = "PIXELS")]
    pub avatar_font_size: Option<f32>,

    /// Set the emoji scale from 0.1 through 1.0.
    #[arg(long, value_name = "RATIO")]
    pub avatar_scale: Option<f32>,

    /// Send into an existing thread within the selected webhook channel.
    #[arg(long, value_name = "ID", conflicts_with = "thread_name")]
    pub thread_id: Option<String>,

    /// Create a thread with this name in a forum or media channel.
    #[arg(long, value_name = "NAME", conflicts_with = "thread_id")]
    pub thread_name: Option<String>,

    /// Enable text-to-speech for this message.
    #[arg(long, conflicts_with = "no_tts")]
    pub tts: bool,

    /// Explicitly disable text-to-speech from template or settings.
    #[arg(long, conflicts_with = "tts")]
    pub no_tts: bool,

    /// Read template variables from a JSON object.
    #[arg(long, value_name = "FILE")]
    pub data: Option<PathBuf>,

    /// Set or override a template variable.
    #[arg(long = "var", value_name = "KEY=VALUE")]
    pub variables: Vec<String>,

    /// Render the final payload without contacting Discord.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Put config.toml and templates beside this executable.
    #[arg(long)]
    pub portable: bool,

    /// Replace files created by an earlier initialization.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print the configuration path selected by discovery.
    Path,
    /// Validate configuration, templates, and local avatar inputs offline.
    Validate,
}

#[derive(Debug, Subcommand)]
pub enum TemplatesCommand {
    /// List available Markdown templates.
    List,
}

#[derive(Debug, Subcommand)]
pub enum AvatarCommand {
    /// Render an avatar to a local PNG.
    Preview {
        /// Avatar profile name from config.toml.
        name: String,
        /// Destination PNG path.
        #[arg(long, short, default_value = "avatar.png")]
        output: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum WebhookCommand {
    /// Find or create the configured incoming webhook.
    Setup {
        /// Discord channel ID or alias from the [channels] configuration table.
        #[arg(long, value_name = "CHANNEL")]
        channel: Option<String>,
    },
}

pub async fn execute(cli: Cli) -> Result<()> {
    match (cli.message, cli.command) {
        (Some(message), None) => super::send_message(cli.config, Some(message), cli.quick).await,
        (None, Some(Command::Send(args))) => {
            let args = *args;
            super::send_message(cli.config, args.message, args.options).await
        }
        (None, Some(command)) => super::execute_command(cli.config, command).await,
        (None, None) => {
            bail!("provide a message, for example: pingme 'message content' (or run with --help)")
        }
        (Some(_), Some(_)) => unreachable!("clap rejects positional messages with subcommands"),
    }
}
