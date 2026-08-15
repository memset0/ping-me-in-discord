#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

const SEND_RUNNER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-send-message/scripts/run-pingme.sh"
);
const PROGRESS_RUNNER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-report-work-progress/scripts/run-pingme.sh"
);
const OUTCOME_RUNNER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-report-turn-outcome/scripts/run-pingme.sh"
);
const PROGRESS_SKILL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-report-work-progress/SKILL.md"
);
const OUTCOME_SKILL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-report-turn-outcome/SKILL.md"
);

fn fake_pingme(root: &Path) -> (PathBuf, PathBuf) {
    let binary = root.join("pingme");
    let log = root.join("pingme.log");
    fs::write(
        &binary,
        r#"#!/bin/sh
if [ "$1" = "fail" ]; then
    exit 42
fi
if [ "$1" = "record-context" ]; then
    printf 'context:%s|%s|%s|%s|%s\n' \
        "${PINGME_AGENT_NAME-}" \
        "${PINGME_PROJECT_NAME-}" \
        "${PINGME_SESSION_NAME-}" \
        "${PINGME_SESSION_ID-}" \
        "${CODEX_THREAD_ID-}" >> "$PINGME_TEST_LOG"
    exit 0
fi
printf '%s\n' "$*" >> "$PINGME_TEST_LOG"
exit 0
"#,
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    (binary, log)
}

fn run(root: &TempDir, log: &Path, arguments: &[&str]) -> std::process::Output {
    run_with_sessions(root, log, arguments, None, None, None)
}

fn run_with_sessions(
    root: &TempDir,
    log: &Path,
    arguments: &[&str],
    generic_session_id: Option<&str>,
    codex_thread_id: Option<&str>,
    claude_session_id: Option<&str>,
) -> std::process::Output {
    let mut command = Command::new("/bin/sh");
    command
        .arg(SEND_RUNNER)
        .args(arguments)
        .current_dir(root.path())
        .env("PATH", root.path())
        .env("PINGME_TEST_LOG", log)
        .env_remove("PINGME_AGENT_NAME")
        .env_remove("PINGME_PROJECT_NAME")
        .env_remove("PINGME_SESSION_ID")
        .env_remove("PINGME_SESSION_NAME")
        .env_remove("CODEX_THREAD_ID")
        .env_remove("CLAUDE_CODE_SESSION_ID");
    if let Some(generic_session_id) = generic_session_id {
        command.env("PINGME_SESSION_ID", generic_session_id);
    }
    if let Some(codex_thread_id) = codex_thread_id {
        command.env("CODEX_THREAD_ID", codex_thread_id);
    }
    if let Some(claude_session_id) = claude_session_id {
        command.env("CLAUDE_CODE_SESSION_ID", claude_session_id);
    }
    command.output().unwrap()
}

#[test]
fn runner_copies_are_identical() {
    let send = fs::read(SEND_RUNNER).unwrap();
    assert_eq!(send, fs::read(PROGRESS_RUNNER).unwrap());
    assert_eq!(send, fs::read(OUTCOME_RUNNER).unwrap());
}

#[test]
fn automatic_skills_have_distinct_status_boundaries_without_visual_fields() {
    let progress = fs::read_to_string(PROGRESS_SKILL).unwrap();
    let outcome = fs::read_to_string(OUTCOME_SKILL).unwrap();

    for arguments in [
        "--avatar started",
        "--avatar progress",
        "--avatar warning",
        "--avatar error",
    ] {
        assert!(
            progress.contains(arguments),
            "missing progress mapping: {arguments}"
        );
    }
    for arguments in [
        "--avatar success",
        "--avatar needs-input",
        "--avatar warning",
        "--avatar error",
    ] {
        assert!(
            outcome.contains(arguments),
            "missing outcome mapping: {arguments}"
        );
    }
    assert!(!progress.contains("--avatar success"));
    assert!(!progress.contains("--avatar needs-input"));
    assert!(!outcome.contains("--avatar started"));
    assert!(!outcome.contains("--avatar progress"));

    for skill in [&progress, &outcome] {
        for forbidden in [
            "--avatar-emoji",
            "--avatar-file",
            "--avatar-url",
            "--avatar-text",
            "--avatar-icon",
            "--avatar-background",
            "--avatar-foreground",
            "--avatar-size",
            "--avatar-scale",
        ] {
            assert!(
                !skill.contains(forbidden),
                "automatic skill must not contain visual argument `{forbidden}`"
            );
        }
    }
}

#[test]
fn automatic_skills_define_conversation_activation_and_disable_rules() {
    for path in [PROGRESS_SKILL, OUTCOME_SKILL] {
        let skill = fs::read_to_string(path).unwrap();
        assert!(skill.contains("later turns"));
        assert!(skill.contains("explicitly asks to stop"));
        assert!(skill.contains("Do not carry activation into another conversation"));
        assert!(skill.contains("session name"));
    }
}

#[test]
fn automatic_skill_examples_close_before_remaining_workflow() {
    for path in [PROGRESS_SKILL, OUTCOME_SKILL] {
        let skill = fs::read_to_string(path).unwrap();
        let opening = skill.find("   ```bash\n").unwrap();
        let after_opening = &skill[opening + "   ```bash\n".len()..];
        let closing = after_opening.find("\n   ```\n").unwrap();

        for line in after_opening[..closing].lines() {
            assert!(
                line.starts_with("   "),
                "nested GFM example line lost list indentation: {line:?}"
            );
        }
        let after_closing = &after_opening[closing + "\n   ```\n".len()..];
        assert!(after_closing.starts_with('\n'));
        assert!(after_closing.contains("## Failure rules"));
    }
}

#[test]
fn runner_reports_once_and_preserves_the_original_status() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());
    let output = run(&root, &log, &["--error-channel", "test", "--", "fail"]);

    assert_eq!(output.status.code(), Some(42));
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        "report-error --channel test\n"
    );
}

#[test]
fn runner_does_not_report_success_and_supports_report_only() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());
    let output = run_with_sessions(
        &root,
        &log,
        &["--", "channels", "list", "--json"],
        Some("generic-session"),
        None,
        None,
    );
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&log).unwrap(), "channels list --json\n");

    let output = run(&root, &log, &["--error-channel", "test", "--report-only"]);
    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        "channels list --json\nreport-error --channel test\n"
    );
}

#[test]
fn runner_preflights_generic_codex_and_claude_session_ids_in_order() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());

    let codex = run_with_sessions(
        &root,
        &log,
        &["--print-session-id"],
        None,
        Some("codex-thread"),
        None,
    );
    assert!(codex.status.success());
    assert_eq!(codex.stdout, b"codex-thread\n");

    let claude = run_with_sessions(
        &root,
        &log,
        &["--print-session-id"],
        None,
        Some("stale-codex-thread"),
        Some("claude-session"),
    );
    assert!(claude.status.success());
    assert_eq!(claude.stdout, b"claude-session\n");

    let generic = run_with_sessions(
        &root,
        &log,
        &["--print-session-id"],
        Some("generic-session"),
        Some("stale-codex-thread"),
        Some("stale-claude-session"),
    );
    assert!(generic.status.success());
    assert_eq!(generic.stdout, b"generic-session\n");
}

#[test]
fn runner_exports_explicit_context_and_compatibility_session() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());

    let output = run_with_sessions(
        &root,
        &log,
        &[
            "--agent-name",
            "Custom Agent",
            "--project-name",
            "ping-me-in-discord",
            "--session-name",
            "notification-skill-design",
            "--",
            "record-context",
        ],
        Some("generic-session"),
        Some("stale-codex"),
        Some("stale-claude"),
    );

    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        "context:Custom Agent|ping-me-in-discord|notification-skill-design|generic-session|generic-session\n"
    );
}

#[test]
fn runner_infers_agent_project_and_deterministic_session_name() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());

    let output = run_with_sessions(
        &root,
        &log,
        &["--", "record-context"],
        None,
        None,
        Some("1234567890-claude"),
    );

    assert!(output.status.success());
    let project = root.path().file_name().unwrap().to_string_lossy();
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        format!(
            "context:Claude Code|{project}|session-12345678|1234567890-claude|1234567890-claude\n"
        )
    );
}

#[test]
fn runner_rejects_empty_context_options_without_calling_pingme() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());

    let output = run_with_sessions(
        &root,
        &log,
        &["--session-name", "", "--print-session-id"],
        Some("session"),
        None,
        None,
    );

    assert_eq!(output.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&output.stderr).contains("non-empty value"));
    assert!(!log.exists());
}

#[test]
fn runner_rejects_missing_session_id_without_calling_pingme() {
    let root = TempDir::new().unwrap();
    let (_binary, log) = fake_pingme(root.path());

    let output = run(&root, &log, &["--print-session-id"]);

    assert_eq!(output.status.code(), Some(65));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no coding-agent session ID"));
    assert!(!log.exists());
}
