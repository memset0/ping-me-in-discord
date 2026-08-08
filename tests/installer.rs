#![cfg(unix)]

use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

use sha2::{Digest, Sha256};
use tempfile::TempDir;

const VERSION: &str = "0.1.0";
const TAG: &str = "v0.1.0";
const TARGET: &str = "x86_64-unknown-linux-musl";

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[test]
fn installer_replaces_both_entry_points_before_retiring_only_the_legacy_binary() {
    let root = TempDir::new().unwrap();
    let stage = root.path().join("stage");
    let fake_bin = root.path().join("fake-bin");
    let install_dir = root.path().join("install");
    let templates = install_dir.join("templates");
    fs::create_dir_all(&stage).unwrap();
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&templates).unwrap();

    for binary in ["ping-me-in-discord", "pingme"] {
        fs::write(
            stage.join(binary),
            format!("#!/bin/sh\nprintf '%s\\n' '{binary}'\n"),
        )
        .unwrap();
    }

    let archive_name = format!("ping-me-in-discord-{TAG}-{TARGET}.tar.gz");
    let archive = root.path().join(&archive_name);
    let tar_status = Command::new("tar")
        .args(["-czf"])
        .arg(&archive)
        .args(["-C"])
        .arg(&stage)
        .arg(".")
        .status()
        .expect("tar should start");
    assert!(tar_status.success(), "fixture archive creation failed");

    let checksum = root.path().join(format!("{archive_name}.sha256"));
    let digest = Sha256::digest(fs::read(&archive).unwrap());
    fs::write(
        &checksum,
        format!("{}  {archive_name}\n", hex::encode(digest)),
    )
    .unwrap();

    let fake_curl = fake_bin.join("curl");
    write_executable(
        &fake_curl,
        r#"#!/bin/sh
set -eu
pingme_output=
pingme_url=
while [ "$#" -gt 0 ]; do
    case "$1" in
        -o)
            pingme_output=$2
            shift 2
            ;;
        http://* | https://*)
            pingme_url=$1
            shift
            ;;
        *)
            shift
            ;;
    esac
done
case "$pingme_url" in
    *.sha256) cp "$PINGME_TEST_CHECKSUM" "$pingme_output" ;;
    *) cp "$PINGME_TEST_ARCHIVE" "$pingme_output" ;;
esac
"#,
    );

    let legacy = install_dir.join("notify-me-on-discord");
    let config = install_dir.join("config.toml");
    let default_template = templates.join("defaults.md");
    let unrelated = install_dir.join("keep-me");
    let similar_name = install_dir.join("notify-me-on-discord.backup");
    fs::write(&legacy, "legacy executable").unwrap();
    fs::write(&config, "[defaults]\ntemplate = \"defaults\"\n").unwrap();
    fs::write(&default_template, "{{ message }}\n").unwrap();
    fs::write(&unrelated, "unrelated").unwrap();
    fs::write(&similar_name, "backup").unwrap();

    let mut path_entries = vec![fake_bin];
    if let Some(path) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&path));
    }
    let test_path = env::join_paths(path_entries).unwrap();

    let output = Command::new("sh")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh"))
        .env("PATH", test_path)
        .env("DISCORD_NOTIFICATION_VERSION", VERSION)
        .env("DISCORD_NOTIFICATION_TARGET", TARGET)
        .env("DISCORD_NOTIFICATION_INSTALL_DIR", &install_dir)
        .env("PINGME_TEST_ARCHIVE", &archive)
        .env("PINGME_TEST_CHECKSUM", &checksum)
        .output()
        .expect("installer should start");

    assert!(
        output.status.success(),
        "installer failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for binary in ["ping-me-in-discord", "pingme"] {
        let installed = install_dir.join(binary);
        assert!(installed.is_file(), "{binary} was not installed");
        assert_ne!(
            fs::metadata(installed).unwrap().permissions().mode() & 0o111,
            0,
            "{binary} is not executable"
        );
    }
    assert!(!legacy.exists(), "legacy executable was not retired");
    assert_eq!(
        fs::read_to_string(config).unwrap(),
        "[defaults]\ntemplate = \"defaults\"\n"
    );
    assert_eq!(
        fs::read_to_string(default_template).unwrap(),
        "{{ message }}\n"
    );
    assert_eq!(fs::read_to_string(unrelated).unwrap(), "unrelated");
    assert_eq!(fs::read_to_string(similar_name).unwrap(), "backup");
    assert!(String::from_utf8_lossy(&output.stdout).contains("Removed legacy executable"));
}
