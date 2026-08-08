use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result};
use clap::ValueEnum;
use directories::BaseDirs;

static TEMPORARY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SkillScope {
    Project,
    Global,
}

impl SkillScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Global => "global",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct InstallSummary {
    pub scope: SkillScope,
    pub destination: PathBuf,
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub removed_legacy: usize,
}

struct BundledFile {
    relative_path: &'static str,
    contents: &'static [u8],
    executable: bool,
}

const BUNDLED_FILES: &[BundledFile] = &[
    BundledFile {
        relative_path: "ping-me-send-message/SKILL.md",
        contents: include_bytes!("../.codex/skills/ping-me-send-message/SKILL.md"),
        executable: false,
    },
    BundledFile {
        relative_path: "ping-me-send-message/agents/openai.yaml",
        contents: include_bytes!("../.codex/skills/ping-me-send-message/agents/openai.yaml"),
        executable: false,
    },
    BundledFile {
        relative_path: "ping-me-send-message/scripts/run-pingme.sh",
        contents: include_bytes!("../.codex/skills/ping-me-send-message/scripts/run-pingme.sh"),
        executable: true,
    },
    BundledFile {
        relative_path: "ping-me-report-agent-status/SKILL.md",
        contents: include_bytes!("../.codex/skills/ping-me-report-agent-status/SKILL.md"),
        executable: false,
    },
    BundledFile {
        relative_path: "ping-me-report-agent-status/agents/openai.yaml",
        contents: include_bytes!("../.codex/skills/ping-me-report-agent-status/agents/openai.yaml"),
        executable: false,
    },
    BundledFile {
        relative_path: "ping-me-report-agent-status/scripts/run-pingme.sh",
        contents: include_bytes!(
            "../.codex/skills/ping-me-report-agent-status/scripts/run-pingme.sh"
        ),
        executable: true,
    },
];

const LEGACY_OWNED_FILES: &[&str] = &[
    "discord-notify/SKILL.md",
    "discord-notify/agents/openai.yaml",
    "discord-notify/scripts/run-pingme.sh",
    "discord-agent-notify/SKILL.md",
    "discord-agent-notify/agents/openai.yaml",
    "discord-agent-notify/scripts/run-pingme.sh",
];

const LEGACY_DIRECTORIES_DEEPEST_FIRST: &[&str] = &[
    "discord-notify/agents",
    "discord-notify/scripts",
    "discord-notify",
    "discord-agent-notify/agents",
    "discord-agent-notify/scripts",
    "discord-agent-notify",
];

pub fn install(scope: SkillScope) -> Result<InstallSummary> {
    let current_directory =
        env::current_dir().context("could not determine the current project directory")?;
    let codex_home = env::var_os("CODEX_HOME");
    let home_directory = BaseDirs::new().map(|directories| directories.home_dir().to_path_buf());
    let destination = resolve_destination(
        scope,
        &current_directory,
        codex_home.as_deref(),
        home_directory.as_deref(),
    )?;
    install_into(scope, destination)
}

fn resolve_destination(
    scope: SkillScope,
    current_directory: &Path,
    codex_home: Option<&OsStr>,
    home_directory: Option<&Path>,
) -> Result<PathBuf> {
    match scope {
        SkillScope::Project => Ok(current_directory.join(".codex/skills")),
        SkillScope::Global => {
            if let Some(codex_home) = codex_home.filter(|value| !value.is_empty()) {
                return Ok(PathBuf::from(codex_home).join("skills"));
            }
            home_directory
                .map(|home| home.join(".codex/skills"))
                .context(
                    "could not determine the current user's home directory; set CODEX_HOME and retry",
                )
        }
    }
}

fn install_into(scope: SkillScope, destination: PathBuf) -> Result<InstallSummary> {
    fs::create_dir_all(&destination).with_context(|| {
        format!(
            "could not create Codex skills directory {}",
            destination.display()
        )
    })?;

    let mut summary = InstallSummary {
        scope,
        destination,
        created: 0,
        updated: 0,
        unchanged: 0,
        removed_legacy: 0,
    };

    for bundled in BUNDLED_FILES {
        let target = summary.destination.join(bundled.relative_path);
        let existing = match fs::read(&target) {
            Ok(contents) => Some(contents),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("could not read installed skill file {}", target.display())
                });
            }
        };

        if existing.as_deref() == Some(bundled.contents) {
            if ensure_executable(&target, bundled.executable)? {
                summary.updated += 1;
            } else {
                summary.unchanged += 1;
            }
            continue;
        }

        let existed = existing.is_some();
        write_atomically(&target, bundled.contents, bundled.executable)?;
        if existed {
            summary.updated += 1;
        } else {
            summary.created += 1;
        }
    }

    summary.removed_legacy = remove_legacy_owned_files(&summary.destination)?;

    Ok(summary)
}

fn remove_legacy_owned_files(destination: &Path) -> Result<usize> {
    let mut removed = 0;

    for relative_path in LEGACY_OWNED_FILES {
        let target = destination.join(relative_path);
        match fs::remove_file(&target) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("could not remove legacy skill file {}", target.display())
                });
            }
        }
    }

    for relative_path in LEGACY_DIRECTORIES_DEEPEST_FIRST {
        let target = destination.join(relative_path);
        match fs::remove_dir(&target) {
            Ok(()) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                ) => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "could not remove empty legacy skill directory {}",
                        target.display()
                    )
                });
            }
        }
    }

    Ok(removed)
}

fn write_atomically(target: &Path, contents: &[u8], executable: bool) -> Result<()> {
    let parent = target
        .parent()
        .context("bundled skill file destination has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "could not create bundled skill directory {}",
            parent.display()
        )
    })?;

    let temporary = temporary_path(target)?;
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("could not create temporary file {}", temporary.display()))?;
        file.write_all(contents)
            .with_context(|| format!("could not write temporary file {}", temporary.display()))?;
        file.sync_all()
            .with_context(|| format!("could not flush temporary file {}", temporary.display()))?;
        set_new_file_permissions(&temporary, executable)?;
        replace_file(&temporary, target)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn temporary_path(target: &Path) -> Result<PathBuf> {
    let file_name = target
        .file_name()
        .context("bundled skill file destination has no file name")?;
    let sequence = TEMPORARY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut temporary_name = OsString::from(".");
    temporary_name.push(file_name);
    temporary_name.push(format!(".tmp-{}-{sequence}", process::id()));
    Ok(target.with_file_name(temporary_name))
}

#[cfg(not(windows))]
fn replace_file(temporary: &Path, target: &Path) -> Result<()> {
    fs::rename(temporary, target).with_context(|| {
        format!(
            "could not replace installed skill file {}",
            target.display()
        )
    })
}

#[cfg(windows)]
fn replace_file(temporary: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_file(target).with_context(|| {
            format!(
                "could not remove outdated installed skill file {}",
                target.display()
            )
        })?;
    }
    fs::rename(temporary, target).with_context(|| {
        format!(
            "could not replace installed skill file {}",
            target.display()
        )
    })
}

#[cfg(unix)]
fn set_new_file_permissions(path: &Path, executable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if executable {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).with_context(|| {
            format!("could not make skill runner executable: {}", path.display())
        })?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_new_file_permissions(_path: &Path, _executable: bool) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn ensure_executable(path: &Path, executable: bool) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    if !executable {
        return Ok(false);
    }
    let metadata = fs::metadata(path)
        .with_context(|| format!("could not inspect skill runner {}", path.display()))?;
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    if mode & 0o111 != 0 {
        return Ok(false);
    }
    fs::set_permissions(path, fs::Permissions::from_mode(mode | 0o111))
        .with_context(|| format!("could not make skill runner executable: {}", path.display()))?;
    Ok(true)
}

#[cfg(not(unix))]
fn ensure_executable(_path: &Path, _executable: bool) -> Result<bool> {
    Ok(false)
}

pub fn print_summary(summary: &InstallSummary) {
    println!(
        "Installed Codex skills at {} (scope: {})",
        summary.destination.display(),
        summary.scope.as_str()
    );
    println!(
        "Files: {} created, {} updated, {} unchanged",
        summary.created, summary.updated, summary.unchanged
    );
    println!("Legacy files: {} removed", summary.removed_legacy);
    println!(
        "Restart or reopen Codex to load $ping-me-send-message and $ping-me-report-agent-status."
    );
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn resolves_project_and_global_destinations() {
        let root = TempDir::new().unwrap();
        let project = root.path().join("project");
        let codex_home = root.path().join("codex-home");
        let home = root.path().join("home");

        assert_eq!(
            resolve_destination(SkillScope::Project, &project, None, None).unwrap(),
            project.join(".codex/skills")
        );
        assert_eq!(
            resolve_destination(
                SkillScope::Global,
                &project,
                Some(codex_home.as_os_str()),
                Some(&home),
            )
            .unwrap(),
            codex_home.join("skills")
        );
        assert_eq!(
            resolve_destination(
                SkillScope::Global,
                &project,
                Some(OsStr::new("")),
                Some(&home)
            )
            .unwrap(),
            home.join(".codex/skills")
        );
    }

    #[test]
    fn missing_global_home_has_actionable_error() {
        let error =
            resolve_destination(SkillScope::Global, Path::new("/project"), None, None).unwrap_err();
        assert!(format!("{error:#}").contains("set CODEX_HOME"));
    }

    #[test]
    fn installation_tracks_updates_and_preserves_unrelated_files() {
        let root = TempDir::new().unwrap();
        let destination = root.path().join("skills");
        let unrelated = destination.join("custom/SKILL.md");
        fs::create_dir_all(unrelated.parent().unwrap()).unwrap();
        fs::write(&unrelated, "custom").unwrap();

        let first = install_into(SkillScope::Project, destination.clone()).unwrap();
        assert_eq!(
            (
                first.created,
                first.updated,
                first.unchanged,
                first.removed_legacy
            ),
            (6, 0, 0, 0)
        );

        let second = install_into(SkillScope::Project, destination.clone()).unwrap();
        assert_eq!(
            (
                second.created,
                second.updated,
                second.unchanged,
                second.removed_legacy
            ),
            (0, 0, 6, 0)
        );

        fs::write(
            destination.join("ping-me-send-message/SKILL.md"),
            "outdated",
        )
        .unwrap();
        let third = install_into(SkillScope::Project, destination.clone()).unwrap();
        assert_eq!(
            (
                third.created,
                third.updated,
                third.unchanged,
                third.removed_legacy
            ),
            (0, 1, 5, 0)
        );
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "custom");
    }

    #[test]
    fn installation_removes_only_legacy_owned_files() {
        let root = TempDir::new().unwrap();
        let destination = root.path().join("skills");
        let preserved = destination.join("discord-notify/notes.txt");
        fs::create_dir_all(preserved.parent().unwrap()).unwrap();
        fs::write(&preserved, "keep me").unwrap();

        for relative_path in LEGACY_OWNED_FILES {
            let target = destination.join(relative_path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(target, "legacy").unwrap();
        }

        let summary = install_into(SkillScope::Project, destination.clone()).unwrap();
        assert_eq!(summary.removed_legacy, LEGACY_OWNED_FILES.len());
        assert_eq!(fs::read_to_string(preserved).unwrap(), "keep me");
        for relative_path in LEGACY_OWNED_FILES {
            assert!(!destination.join(relative_path).exists());
        }
        assert!(!destination.join("discord-agent-notify").exists());
    }
}
