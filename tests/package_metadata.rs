use std::{collections::BTreeSet, process::Command};

#[test]
fn cargo_declares_only_the_supported_binary_entry_points() {
    let output = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
        .output()
        .expect("cargo metadata should start");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata should return JSON");
    let package = metadata["packages"]
        .as_array()
        .expect("metadata should contain packages")
        .iter()
        .find(|package| package["name"] == env!("CARGO_PKG_NAME"))
        .expect("metadata should contain this package");

    let binaries = package["targets"]
        .as_array()
        .expect("package should contain targets")
        .iter()
        .filter(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
        })
        .map(|target| {
            target["name"]
                .as_str()
                .expect("binary target should have a name")
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(binaries, BTreeSet::from(["ping-me-in-discord", "pingme"]));
}
