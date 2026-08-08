#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

const SIMPLE_RUNNER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-send-message/scripts/run-pingme.sh"
);
const STRICT_RUNNER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-report-agent-status/scripts/run-pingme.sh"
);
const STRICT_SKILL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.codex/skills/ping-me-report-agent-status/SKILL.md"
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
printf '%s\n' "$*" >> "$PINGME_TEST_LOG"
exit 0
"#,
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    (binary, log)
}

fn run(root: &TempDir, log: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new("/bin/sh")
        .arg(SIMPLE_RUNNER)
        .args(arguments)
        .env("PATH", root.path())
        .env("PINGME_TEST_LOG", log)
        .output()
        .unwrap()
}

#[test]
fn runner_copies_are_identical() {
    assert_eq!(
        fs::read(SIMPLE_RUNNER).unwrap(),
        fs::read(STRICT_RUNNER).unwrap()
    );
}

#[test]
fn strict_skill_selects_configured_status_profiles_without_visual_fields() {
    let skill = fs::read_to_string(STRICT_SKILL).unwrap();
    for arguments in [
        "--avatar started",
        "--avatar progress",
        "--avatar success",
        "--avatar needs-input",
        "--avatar warning",
        "--avatar error",
    ] {
        assert!(
            skill.contains(arguments),
            "missing strict mapping: {arguments}"
        );
    }
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
            "strict skill must not contain visual argument `{forbidden}`"
        );
    }
}

#[test]
fn strict_skill_keeps_the_nested_example_inside_its_gfm_fence() {
    let skill = fs::read_to_string(STRICT_SKILL).unwrap();
    let opening = skill.find("   ```bash\n").unwrap();
    let example = &skill[opening + "   ```bash\n".len()..];
    let closing = example.find("\n   ```\n").unwrap();

    for line in example[..closing].lines() {
        assert!(
            line.starts_with("   "),
            "nested GFM example line lost list indentation: {line:?}"
        );
    }
    assert!(example[closing..].starts_with("\n   ```\n\n7. Inspect"));
    assert!(skill.find("7. Inspect").unwrap() < skill.find("## Failure rules").unwrap());
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
    let output = run(&root, &log, &["--", "channels", "list", "--json"]);
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&log).unwrap(), "channels list --json\n");

    let output = run(&root, &log, &["--error-channel", "test", "--report-only"]);
    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(log).unwrap(),
        "channels list --json\nreport-error --channel test\n"
    );
}
