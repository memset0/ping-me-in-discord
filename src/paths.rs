use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;

pub const APP_DIR: &str = "discord-notification";
pub const CONFIG_FILE: &str = "config.toml";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserDirs {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl UserDirs {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("", "", APP_DIR)
            .context("could not determine the current user's configuration directories")?;
        Ok(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DiscoveryInput {
    pub explicit: Option<PathBuf>,
    pub environment: Option<PathBuf>,
    pub executable: PathBuf,
    pub user_dirs: UserDirs,
}

pub fn discover_config(explicit: Option<PathBuf>) -> Result<PathBuf> {
    let environment = std::env::var_os("DISCORD_NOTIFICATION_CONFIG").map(PathBuf::from);
    let executable =
        std::env::current_exe().context("could not determine the running executable path")?;
    discover_config_from(DiscoveryInput {
        explicit,
        environment,
        executable,
        user_dirs: UserDirs::discover()?,
    })
}

pub fn discover_config_from(input: DiscoveryInput) -> Result<PathBuf> {
    let candidates = [
        input.explicit,
        input.environment,
        input
            .executable
            .parent()
            .map(|directory| directory.join(CONFIG_FILE)),
        Some(input.user_dirs.config_dir.join(CONFIG_FILE)),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            return candidate
                .canonicalize()
                .with_context(|| format!("could not resolve {}", candidate.display()));
        }
    }

    let portable = input
        .executable
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(CONFIG_FILE);
    let user = input.user_dirs.config_dir.join(CONFIG_FILE);
    bail!(
        "no configuration found; looked for {} and {} (run `notify-me-on-discord init` or `notify-me-on-discord init --portable`)",
        portable.display(),
        user.display()
    )
}

pub fn init_directory(portable: bool) -> Result<PathBuf> {
    if portable {
        let executable =
            std::env::current_exe().context("could not determine the running executable path")?;
        executable
            .parent()
            .map(Path::to_path_buf)
            .context("the running executable has no parent directory")
    } else {
        Ok(UserDirs::discover()?.config_dir)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn input(root: &TempDir) -> DiscoveryInput {
        DiscoveryInput {
            explicit: None,
            environment: None,
            executable: root.path().join("bin/notify-me-on-discord"),
            user_dirs: UserDirs {
                config_dir: root.path().join("xdg-config/discord-notification"),
                data_dir: root.path().join("xdg-data/discord-notification"),
            },
        }
    }

    fn touch(path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }

    #[test]
    fn explicit_path_wins() {
        let root = TempDir::new().unwrap();
        let mut input = input(&root);
        let explicit = root.path().join("custom/config.toml");
        let portable = root.path().join("bin/config.toml");
        touch(&explicit);
        touch(&portable);
        input.explicit = Some(explicit.clone());

        assert_eq!(
            discover_config_from(input).unwrap(),
            explicit.canonicalize().unwrap()
        );
    }

    #[test]
    fn environment_path_wins_over_portable() {
        let root = TempDir::new().unwrap();
        let mut input = input(&root);
        let environment = root.path().join("environment/config.toml");
        let portable = root.path().join("bin/config.toml");
        touch(&environment);
        touch(&portable);
        input.environment = Some(environment.clone());

        assert_eq!(
            discover_config_from(input).unwrap(),
            environment.canonicalize().unwrap()
        );
    }

    #[test]
    fn portable_path_wins_over_user_path() {
        let root = TempDir::new().unwrap();
        let input = input(&root);
        let portable = root.path().join("bin/config.toml");
        let user = input.user_dirs.config_dir.join("config.toml");
        touch(&portable);
        touch(&user);

        assert_eq!(
            discover_config_from(input).unwrap(),
            portable.canonicalize().unwrap()
        );
    }
}
