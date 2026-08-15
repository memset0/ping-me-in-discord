use std::{fs, path::Path};

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn portable_fixture() -> (TempDir, std::path::PathBuf) {
    let root = TempDir::new().unwrap();
    let config = root.path().join("config.toml");
    fs::create_dir_all(root.path().join("templates")).unwrap();
    fs::write(
        &config,
        r#"
[discord]
webhook_url = "https://discord.com/api/webhooks/123/super-secret-token"
webhook_name = "Notify Me"

[channels]
settings = "111"
template = "222"
command = "333"

[templates]
directory = "templates"

[defaults]
template = "defaults"
channel = "settings"
username = "Ping Me"

[avatars.release]
description = "Use for release notifications"
type = "image"
source = "https://example.com/release.png"
"#,
    )
    .unwrap();
    fs::write(root.path().join("templates/defaults.md"), "{{ message }}").unwrap();
    (root, config)
}

fn config_argument(config: &Path) -> String {
    config.display().to_string()
}

#[test]
fn pingme_shorthand_renders_default_template_without_network() {
    let (_root, config) = portable_fixture();
    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "message content",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\": \"message content\""))
        .stdout(predicate::str::contains("\"avatar\"").not())
        .stdout(predicate::str::contains("super-secret-token").not());
}

#[test]
fn send_arguments_override_frontmatter_and_resolve_channel_aliases() {
    let (root, config) = portable_fixture();
    fs::write(
        root.path().join("templates/identity.md"),
        r#"---
channel: template
username: Template User
avatar_url: https://example.com/template.png
tts: true
---
{{ message }}
"#,
    )
    .unwrap();

    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "overridden",
            "--template",
            "identity",
            "--channel",
            "command",
            "--username",
            "CLI User",
            "--avatar",
            "release",
            "--no-tts",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"channel\": \"333\""))
        .stdout(predicate::str::contains("\"username\": \"CLI User\""))
        .stdout(predicate::str::contains("\"tts\": false"))
        .stdout(predicate::str::contains("\"name\": \"release\""))
        .stdout(predicate::str::contains("template.png").not());
}

#[test]
fn conflicting_avatar_sources_are_rejected_by_argument_parser() {
    let (_root, config) = portable_fixture();
    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "hello",
            "--avatar",
            "release",
            "--avatar-emoji",
            "🚀",
            "--dry-run",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn one_off_emoji_foreground_is_accepted_in_dry_run() {
    let (_root, config) = portable_fixture();
    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "failed",
            "--avatar-emoji",
            "❌",
            "--avatar-foreground",
            "#FFFFFF",
            "--avatar-background",
            "#DD2E44",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"emoji\": \"❌\""))
        .stdout(predicate::str::contains("\"foreground\": \"#FFFFFF\""))
        .stdout(predicate::str::contains("\"background\": \"#DD2E44\""));
}

#[test]
fn long_binary_name_supports_the_same_shorthand() {
    let (_root, config) = portable_fixture();
    let mut command = cargo_bin_cmd!("ping-me-in-discord");
    command
        .args(["--config", &config_argument(&config), "hello", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\": \"hello\""));
}

#[test]
fn named_template_renders_frontmatter_and_variables() {
    let (root, config) = portable_fixture();
    fs::write(
        root.path().join("templates/deploy.md"),
        r##"---
username: "{{ project }}"
embeds:
  - title: Deployment
    description: "{{ message }}"
    color: "#5865F2"
---
"##,
    )
    .unwrap();

    let mut command = cargo_bin_cmd!("ping-me-in-discord");
    command
        .args([
            "--config",
            &config_argument(&config),
            "send",
            "complete",
            "--template",
            "deploy",
            "--var",
            "project=API",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"username\": \"API\""))
        .stdout(predicate::str::contains("\"description\": \"complete\""))
        .stdout(predicate::str::contains("\"color\": 5793266"));
}

#[test]
fn absolute_template_path_renders_outside_the_configured_directory() {
    let (root, config) = portable_fixture();
    let external_directory = root.path().join("external");
    let external_template = external_directory.join("custom.md");
    fs::create_dir_all(&external_directory).unwrap();
    fs::write(
        &external_template,
        "---\nusername: External Template\n---\nabsolute: {{ message }}",
    )
    .unwrap();

    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "outside",
            "--template",
            external_template.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"username\": \"External Template\"",
        ))
        .stdout(predicate::str::contains(
            "\"content\": \"absolute: outside\"",
        ));
}

#[test]
fn absolute_configured_default_is_validated_and_rendered() {
    let (root, config) = portable_fixture();
    let external_template = root.path().join("configured-default.md");
    fs::write(&external_template, "configured: {{ message }}").unwrap();

    let escaped_template = external_template
        .display()
        .to_string()
        .replace('\\', "\\\\");
    let source = fs::read_to_string(&config).unwrap().replace(
        "template = \"defaults\"",
        &format!("template = \"{escaped_template}\""),
    );
    fs::write(&config, source).unwrap();
    let config_value = config_argument(&config);

    let mut validate = cargo_bin_cmd!("pingme");
    validate
        .args(["--config", &config_value, "config", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));

    let mut send = cargo_bin_cmd!("pingme");
    send.args(["--config", &config_value, "configured default", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"content\": \"configured: configured default\"",
        ));
}

#[test]
fn configuration_validation_and_template_listing_work_offline() {
    let (_root, config) = portable_fixture();
    let config_value = config_argument(&config);

    let mut validate = cargo_bin_cmd!("ping-me-in-discord");
    validate
        .args(["--config", &config_value, "config", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));

    let mut list = cargo_bin_cmd!("ping-me-in-discord");
    list.args(["--config", &config_value, "templates", "list"])
        .assert()
        .success()
        .stdout(predicate::eq("defaults\n"));
}

#[test]
fn channel_listing_json_exposes_only_routing_metadata() {
    let (_root, config) = portable_fixture();
    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "channels",
            "list",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"selector\": \"settings\""))
        .stdout(predicate::str::contains("\"id\": \"111\""))
        .stdout(predicate::str::contains("\"alias\": \"command\""))
        .stdout(predicate::str::contains("super-secret-token").not())
        .stdout(predicate::str::contains("release.png").not());
}

#[test]
fn avatar_listing_json_exposes_only_safe_profile_metadata() {
    let (_root, config) = portable_fixture();
    let mut command = cargo_bin_cmd!("pingme");
    command
        .args([
            "--config",
            &config_argument(&config),
            "avatar",
            "list",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"release\""))
        .stdout(predicate::str::contains("\"type\": \"image\""))
        .stdout(predicate::str::contains(
            "\"description\": \"Use for release notifications\"",
        ))
        .stdout(predicate::str::contains("\"is_default\": false"))
        .stdout(predicate::str::contains("release.png").not())
        .stdout(predicate::str::contains("super-secret-token").not());
}
