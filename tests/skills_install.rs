use std::{fs, path::Path};

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

const SHARED_FILES: &[(&str, &[u8])] = &[
    (
        "ping-me-send-message/SKILL.md",
        include_bytes!("../.codex/skills/ping-me-send-message/SKILL.md"),
    ),
    (
        "ping-me-send-message/scripts/run-pingme.sh",
        include_bytes!("../.codex/skills/ping-me-send-message/scripts/run-pingme.sh"),
    ),
    (
        "ping-me-report-work-progress/SKILL.md",
        include_bytes!("../.codex/skills/ping-me-report-work-progress/SKILL.md"),
    ),
    (
        "ping-me-report-work-progress/scripts/run-pingme.sh",
        include_bytes!("../.codex/skills/ping-me-report-work-progress/scripts/run-pingme.sh"),
    ),
    (
        "ping-me-report-turn-outcome/SKILL.md",
        include_bytes!("../.codex/skills/ping-me-report-turn-outcome/SKILL.md"),
    ),
    (
        "ping-me-report-turn-outcome/scripts/run-pingme.sh",
        include_bytes!("../.codex/skills/ping-me-report-turn-outcome/scripts/run-pingme.sh"),
    ),
];

const CODEX_ONLY_FILES: &[(&str, &[u8])] = &[
    (
        "ping-me-send-message/agents/openai.yaml",
        include_bytes!("../.codex/skills/ping-me-send-message/agents/openai.yaml"),
    ),
    (
        "ping-me-report-work-progress/agents/openai.yaml",
        include_bytes!("../.codex/skills/ping-me-report-work-progress/agents/openai.yaml"),
    ),
    (
        "ping-me-report-turn-outcome/agents/openai.yaml",
        include_bytes!("../.codex/skills/ping-me-report-turn-outcome/agents/openai.yaml"),
    ),
];

const CODEX_LEGACY_OWNED_FILES: &[&str] = &[
    "discord-notify/SKILL.md",
    "discord-notify/agents/openai.yaml",
    "discord-notify/scripts/run-pingme.sh",
    "discord-agent-notify/SKILL.md",
    "discord-agent-notify/agents/openai.yaml",
    "discord-agent-notify/scripts/run-pingme.sh",
    "ping-me-report-agent-status/SKILL.md",
    "ping-me-report-agent-status/agents/openai.yaml",
    "ping-me-report-agent-status/scripts/run-pingme.sh",
];

fn assert_files(destination: &Path, files: &[(&str, &[u8])]) {
    for (relative, expected) in files {
        assert_eq!(
            fs::read(destination.join(relative)).unwrap(),
            *expected,
            "installed asset differs: {relative}"
        );
    }
}

fn assert_codex_files(destination: &Path) {
    assert_files(destination, SHARED_FILES);
    assert_files(destination, CODEX_ONLY_FILES);
}

fn assert_claude_files(destination: &Path) {
    assert_files(destination, SHARED_FILES);
    for (relative, _) in CODEX_ONLY_FILES {
        assert!(
            !destination.join(relative).exists(),
            "Claude Code install contains Codex-only asset: {relative}"
        );
    }
}

fn assert_regular_owned_files(destination: &Path, files: &[(&str, &[u8])]) {
    for (relative, _) in files {
        let metadata = fs::symlink_metadata(destination.join(relative)).unwrap();
        assert!(
            metadata.file_type().is_file(),
            "installed asset is not a regular file: {relative}"
        );
        assert!(
            !metadata.file_type().is_symlink(),
            "installed asset is a symbolic link: {relative}"
        );
    }
}

#[test]
fn project_install_is_complete_repeatable_and_narrowly_owned() {
    let project = TempDir::new().unwrap();
    let destination = project.path().join(".codex/skills");
    let unrelated = destination.join("third-party/SKILL.md");
    fs::create_dir_all(unrelated.parent().unwrap()).unwrap();
    fs::write(&unrelated, "keep me").unwrap();

    let mut command = cargo_bin_cmd!("pingme");
    command
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args(["skills", "install", "--scope", "project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scope: project"))
        .stdout(predicate::str::contains(
            "9 created, 0 updated, 0 unchanged",
        ))
        .stdout(predicate::str::contains("Legacy files: 0 removed"))
        .stdout(predicate::str::contains("$ping-me-send-message"))
        .stdout(predicate::str::contains("$ping-me-report-work-progress"))
        .stdout(predicate::str::contains("$ping-me-report-turn-outcome"));

    assert_codex_files(&destination);
    assert_regular_owned_files(&destination, SHARED_FILES);
    assert_regular_owned_files(&destination, CODEX_ONLY_FILES);
    assert_eq!(fs::read_to_string(&unrelated).unwrap(), "keep me");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let runner = destination.join("ping-me-send-message/scripts/run-pingme.sh");
        fs::set_permissions(&runner, fs::Permissions::from_mode(0o644)).unwrap();
        let mut repair_permissions = cargo_bin_cmd!("pingme");
        repair_permissions
            .current_dir(project.path())
            .env_remove("DISCORD_NOTIFICATION_CONFIG")
            .args(["skills", "install", "--scope", "project"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "0 created, 1 updated, 8 unchanged",
            ));
        assert_ne!(
            fs::metadata(runner).unwrap().permissions().mode() & 0o111,
            0
        );
    }

    let mut unchanged = cargo_bin_cmd!("pingme");
    unchanged
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args(["skills", "install", "--scope", "project"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0 created, 0 updated, 9 unchanged",
        ));

    fs::write(
        destination.join("ping-me-send-message/SKILL.md"),
        "outdated",
    )
    .unwrap();
    let mut refresh = cargo_bin_cmd!("pingme");
    refresh
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args(["skills", "install", "--scope", "project"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0 created, 1 updated, 8 unchanged",
        ));

    assert_codex_files(&destination);
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "keep me");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for skill in [
            "ping-me-send-message",
            "ping-me-report-work-progress",
            "ping-me-report-turn-outcome",
        ] {
            let mode = fs::metadata(destination.join(skill).join("scripts/run-pingme.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_ne!(mode & 0o111, 0, "runner is not executable: {skill}");
        }
    }
}

#[test]
fn project_install_migrates_legacy_owned_files_without_removing_extras() {
    let project = TempDir::new().unwrap();
    let destination = project.path().join(".codex/skills");
    let legacy_extra = destination.join("discord-notify/references/keep.md");
    let unrelated = destination.join("third-party/SKILL.md");
    fs::create_dir_all(legacy_extra.parent().unwrap()).unwrap();
    fs::create_dir_all(unrelated.parent().unwrap()).unwrap();
    fs::write(&legacy_extra, "legacy extra").unwrap();
    fs::write(&unrelated, "third party").unwrap();

    for relative in CODEX_LEGACY_OWNED_FILES {
        let path = destination.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "legacy owned").unwrap();
    }

    let mut command = cargo_bin_cmd!("pingme");
    command
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args(["skills", "install", "--scope", "project"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "9 created, 0 updated, 0 unchanged",
        ))
        .stdout(predicate::str::contains("Legacy files: 9 removed"));

    assert_codex_files(&destination);
    assert_eq!(fs::read_to_string(legacy_extra).unwrap(), "legacy extra");
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "third party");
    for relative in CODEX_LEGACY_OWNED_FILES {
        assert!(!destination.join(relative).exists());
    }
    assert!(!destination.join("discord-agent-notify").exists());

    let mut repeat = cargo_bin_cmd!("pingme");
    repeat
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args(["skills", "install", "--scope", "project"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0 created, 0 updated, 9 unchanged",
        ))
        .stdout(predicate::str::contains("Legacy files: 0 removed"));
}

#[test]
fn global_install_honors_codex_home_without_configuration_or_source_checkout() {
    let root = TempDir::new().unwrap();
    let empty_working_directory = root.path().join("empty");
    let codex_home = root.path().join("custom-codex-home");
    fs::create_dir(&empty_working_directory).unwrap();

    let mut command = cargo_bin_cmd!("ping-me-in-discord");
    command
        .current_dir(&empty_working_directory)
        .env("CODEX_HOME", &codex_home)
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args(["skills", "install", "--scope", "global"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scope: global"))
        .stdout(predicate::str::contains(
            codex_home.join("skills").display().to_string(),
        ));

    assert_codex_files(&codex_home.join("skills"));
    assert!(!empty_working_directory.join(".codex").exists());
}

#[test]
fn claude_project_install_copies_only_shared_regular_files() {
    let project = TempDir::new().unwrap();
    let destination = project.path().join(".claude/skills");
    let legacy = destination.join("discord-notify/SKILL.md");
    let retired_skill = destination.join("ping-me-report-agent-status/SKILL.md");
    let retired_runner = destination.join("ping-me-report-agent-status/scripts/run-pingme.sh");
    let retired_metadata = destination.join("ping-me-report-agent-status/agents/openai.yaml");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::create_dir_all(retired_runner.parent().unwrap()).unwrap();
    fs::create_dir_all(retired_metadata.parent().unwrap()).unwrap();
    fs::write(&legacy, "Claude-owned content").unwrap();
    fs::write(&retired_skill, "retired shared content").unwrap();
    fs::write(&retired_runner, "retired shared runner").unwrap();
    fs::write(&retired_metadata, "preserve Codex-only metadata").unwrap();

    let mut command = cargo_bin_cmd!("pingme");
    command
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args([
            "skills",
            "install",
            "--scope",
            "project",
            "--agent",
            "claude-code",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent: claude-code"))
        .stdout(predicate::str::contains("scope: project"))
        .stdout(predicate::str::contains(
            "6 created, 0 updated, 0 unchanged",
        ))
        .stdout(predicate::str::contains("Legacy files: 2 removed"))
        .stdout(predicate::str::contains("/ping-me-send-message"))
        .stdout(predicate::str::contains("/ping-me-report-work-progress"))
        .stdout(predicate::str::contains("/ping-me-report-turn-outcome"));

    assert_claude_files(&destination);
    assert_regular_owned_files(&destination, SHARED_FILES);
    assert_eq!(fs::read_to_string(legacy).unwrap(), "Claude-owned content");
    assert!(!retired_skill.exists());
    assert!(!retired_runner.exists());
    assert_eq!(
        fs::read_to_string(retired_metadata).unwrap(),
        "preserve Codex-only metadata"
    );
    assert!(!project.path().join(".codex").exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for skill in [
            "ping-me-send-message",
            "ping-me-report-work-progress",
            "ping-me-report-turn-outcome",
        ] {
            let mode = fs::metadata(destination.join(skill).join("scripts/run-pingme.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_ne!(mode & 0o111, 0, "runner is not executable: {skill}");
        }
    }

    let mut repeat = cargo_bin_cmd!("pingme");
    repeat
        .current_dir(project.path())
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args([
            "skills", "install", "--scope", "project", "--agent", "claude",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0 created, 0 updated, 6 unchanged",
        ));
}

#[test]
fn claude_global_install_honors_config_directory_without_checkout() {
    let root = TempDir::new().unwrap();
    let empty_working_directory = root.path().join("empty");
    let claude_config = root.path().join("custom-claude-config");
    fs::create_dir(&empty_working_directory).unwrap();

    let mut command = cargo_bin_cmd!("ping-me-in-discord");
    command
        .current_dir(&empty_working_directory)
        .env("CLAUDE_CONFIG_DIR", &claude_config)
        .env_remove("DISCORD_NOTIFICATION_CONFIG")
        .args([
            "skills",
            "install",
            "--scope",
            "global",
            "--agent",
            "claude-code",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent: claude-code"))
        .stdout(predicate::str::contains(
            claude_config.join("skills").display().to_string(),
        ));

    let destination = claude_config.join("skills");
    assert_claude_files(&destination);
    assert_regular_owned_files(&destination, SHARED_FILES);
    assert!(!empty_working_directory.join(".claude").exists());
}

#[cfg(unix)]
#[test]
fn claude_install_replaces_owned_symlink_with_a_regular_copy() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    let destination = project.path().join(".claude/skills");
    let target = destination.join("ping-me-send-message/SKILL.md");
    let external = project.path().join("external-skill.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&external, SHARED_FILES[0].1).unwrap();
    symlink(&external, &target).unwrap();

    let mut command = cargo_bin_cmd!("pingme");
    command
        .current_dir(project.path())
        .args([
            "skills",
            "install",
            "--scope",
            "project",
            "--agent",
            "claude-code",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "5 created, 1 updated, 0 unchanged",
        ));

    let metadata = fs::symlink_metadata(&target).unwrap();
    assert!(metadata.file_type().is_file());
    assert!(!metadata.file_type().is_symlink());
    assert_eq!(fs::read(&target).unwrap(), SHARED_FILES[0].1);
    assert_eq!(fs::read(&external).unwrap(), SHARED_FILES[0].1);
}

#[test]
fn missing_scope_fails_before_writing() {
    let project = TempDir::new().unwrap();
    let mut command = cargo_bin_cmd!("pingme");
    command
        .current_dir(project.path())
        .args(["skills", "install"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--scope <SCOPE>"));

    assert!(!project.path().join(".codex").exists());
    assert!(!project.path().join(".claude").exists());
}
