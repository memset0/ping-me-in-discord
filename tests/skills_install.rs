use std::{fs, path::Path};

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

const BUNDLED_FILES: &[(&str, &[u8])] = &[
    (
        "ping-me-send-message/SKILL.md",
        include_bytes!("../.codex/skills/ping-me-send-message/SKILL.md"),
    ),
    (
        "ping-me-send-message/agents/openai.yaml",
        include_bytes!("../.codex/skills/ping-me-send-message/agents/openai.yaml"),
    ),
    (
        "ping-me-send-message/scripts/run-pingme.sh",
        include_bytes!("../.codex/skills/ping-me-send-message/scripts/run-pingme.sh"),
    ),
    (
        "ping-me-report-agent-status/SKILL.md",
        include_bytes!("../.codex/skills/ping-me-report-agent-status/SKILL.md"),
    ),
    (
        "ping-me-report-agent-status/agents/openai.yaml",
        include_bytes!("../.codex/skills/ping-me-report-agent-status/agents/openai.yaml"),
    ),
    (
        "ping-me-report-agent-status/scripts/run-pingme.sh",
        include_bytes!("../.codex/skills/ping-me-report-agent-status/scripts/run-pingme.sh"),
    ),
];

const LEGACY_OWNED_FILES: &[&str] = &[
    "discord-notify/SKILL.md",
    "discord-notify/agents/openai.yaml",
    "discord-notify/scripts/run-pingme.sh",
    "discord-agent-notify/SKILL.md",
    "discord-agent-notify/agents/openai.yaml",
    "discord-agent-notify/scripts/run-pingme.sh",
];

fn assert_bundled_files(destination: &Path) {
    for (relative, expected) in BUNDLED_FILES {
        assert_eq!(
            fs::read(destination.join(relative)).unwrap(),
            *expected,
            "installed asset differs: {relative}"
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
            "6 created, 0 updated, 0 unchanged",
        ))
        .stdout(predicate::str::contains("Legacy files: 0 removed"))
        .stdout(predicate::str::contains("$ping-me-send-message"))
        .stdout(predicate::str::contains("$ping-me-report-agent-status"));

    assert_bundled_files(&destination);
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
                "0 created, 1 updated, 5 unchanged",
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
            "0 created, 0 updated, 6 unchanged",
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
            "0 created, 1 updated, 5 unchanged",
        ));

    assert_bundled_files(&destination);
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "keep me");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for skill in ["ping-me-send-message", "ping-me-report-agent-status"] {
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

    for relative in LEGACY_OWNED_FILES {
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
            "6 created, 0 updated, 0 unchanged",
        ))
        .stdout(predicate::str::contains("Legacy files: 6 removed"));

    assert_bundled_files(&destination);
    assert_eq!(fs::read_to_string(legacy_extra).unwrap(), "legacy extra");
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "third party");
    for relative in LEGACY_OWNED_FILES {
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
            "0 created, 0 updated, 6 unchanged",
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

    assert_bundled_files(&codex_home.join("skills"));
    assert!(!empty_working_directory.join(".codex").exists());
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
}
