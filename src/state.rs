use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct AppState {
    /// Legacy unrouted cache retained for compatibility with early local installs.
    pub provisioned_webhook_url: Option<String>,
    #[serde(default)]
    pub provisioned_webhooks: BTreeMap<String, String>,
    #[serde(default)]
    pub avatar_digests: BTreeMap<String, String>,
}

pub struct StateStore {
    data_directory: PathBuf,
    state_path: PathBuf,
    lock_path: PathBuf,
}

impl StateStore {
    pub fn new(data_directory: PathBuf) -> Self {
        Self {
            state_path: data_directory.join("state.toml"),
            lock_path: data_directory.join("state.lock"),
            data_directory,
        }
    }

    pub fn ensure_directories(&self) -> Result<()> {
        create_private_directory(&self.data_directory)?;
        create_private_directory(&self.emoji_cache_directory())
    }

    pub fn emoji_cache_directory(&self) -> PathBuf {
        self.data_directory.join("emoji")
    }

    pub fn lock(&self) -> Result<StateLock> {
        self.ensure_directories()?;
        let file = open_private(&self.lock_path)?;
        file.lock_exclusive()
            .with_context(|| format!("could not lock {}", self.lock_path.display()))?;
        Ok(StateLock { file })
    }

    pub fn load(&self) -> Result<AppState> {
        if !self.state_path.exists() {
            return Ok(AppState::default());
        }
        let source = fs::read_to_string(&self.state_path)
            .with_context(|| format!("could not read {}", self.state_path.display()))?;
        toml::from_str(&source)
            .with_context(|| format!("could not parse {}", self.state_path.display()))
    }

    pub fn save(&self, state: &AppState) -> Result<()> {
        self.ensure_directories()?;
        let source =
            toml::to_string_pretty(state).context("could not serialize application state")?;
        let temporary = self
            .data_directory
            .join(format!(".state.toml.tmp-{}", std::process::id()));
        {
            let mut file = open_private_truncated(&temporary)?;
            file.write_all(source.as_bytes())
                .with_context(|| format!("could not write {}", temporary.display()))?;
            file.sync_all()
                .with_context(|| format!("could not sync {}", temporary.display()))?;
        }
        fs::rename(&temporary, &self.state_path).with_context(|| {
            format!(
                "could not atomically replace {} with {}",
                self.state_path.display(),
                temporary.display()
            )
        })
    }
}

pub struct StateLock {
    file: File,
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

fn create_private_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("could not create {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("could not secure {}", path.display()))?;
    }
    Ok(())
}

fn open_private(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .with_context(|| format!("could not open {}", path.display()))
}

fn open_private_truncated(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .with_context(|| format!("could not open {}", path.display()))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn state_round_trip() {
        let root = TempDir::new().unwrap();
        let store = StateStore::new(root.path().join("data"));
        let _lock = store.lock().unwrap();
        let mut state = AppState {
            provisioned_webhook_url: Some("https://discord.com/api/webhooks/id/token".to_owned()),
            ..AppState::default()
        };
        state.provisioned_webhooks.insert(
            "123".to_owned(),
            "https://discord.com/api/webhooks/456/token".to_owned(),
        );
        state
            .avatar_digests
            .insert("id".to_owned(), "digest".to_owned());
        store.save(&state).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(
            loaded.provisioned_webhook_url.as_deref(),
            Some("https://discord.com/api/webhooks/id/token")
        );
        assert_eq!(
            loaded.provisioned_webhooks.get("123").map(String::as_str),
            Some("https://discord.com/api/webhooks/456/token")
        );
        assert_eq!(loaded.avatar_digests.get("id").unwrap(), "digest");
    }
}
