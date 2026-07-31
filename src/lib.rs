pub mod avatar;
pub mod cli;
pub mod config;
pub mod discord;
pub mod options;
pub mod paths;
pub mod state;
pub mod template;

use std::ffi::OsString;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, ensure};
use clap::Parser;
use serde_json::Value;

use crate::{
    avatar::{AvatarRenderer, AvatarSelection, ResolvedAvatar},
    cli::{
        AvatarCommand, Cli, Command, ConfigCommand, SendOptions, TemplatesCommand, WebhookCommand,
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
    let (webhook, provisioned) = discord
        .resolve_webhook(
            &loaded.config.discord,
            &mut state,
            rendered.channel.as_deref(),
        )
        .await?;
    if provisioned {
        store.save(&state)?;
    }

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
                if avatar::needs_webhook_update(&state, webhook.id(), &digest) {
                    discord.modify_avatar(&webhook, &png).await?;
                    state.avatar_digests.insert(webhook.id().to_owned(), digest);
                    store.save(&state)?;
                }
            }
        }
    } else if !rendered
        .payload
        .get("avatar_url")
        .is_some_and(Value::is_string)
        && avatar::needs_webhook_reset(&state, webhook.id())
    {
        discord.reset_avatar(&webhook).await?;
        state.avatar_digests.remove(webhook.id());
        store.save(&state)?;
    }

    let message_id = discord
        .execute(&webhook, &rendered.payload, rendered.thread_id.as_deref())
        .await?;
    println!("Sent Discord message {message_id}");
    Ok(())
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
        Command::Send(_) => unreachable!("send commands are handled by cli::execute"),
    }
}

fn load_config(explicit: Option<PathBuf>) -> Result<LoadedConfig> {
    LoadedConfig::load(paths::discover_config(explicit)?)
}
