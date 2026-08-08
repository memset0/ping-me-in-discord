# user-installation-and-releases Specification

## Purpose

Define reproducible standalone releases and a no-root installation experience centered on common Linux user-directory conventions.

## Requirements

### Requirement: Linux installation does not require root
The Unix installer SHALL place `notify-me-on-discord` in `$DISCORD_NOTIFICATION_INSTALL_DIR` when set and otherwise in `~/.local/bin`, and SHALL install `pingme` as an equivalent entry point. It SHALL create the target directory as the current user and SHALL never invoke `sudo` or write to system directories by default.

#### Scenario: Default Linux install
- **WHEN** a Linux user runs the installer without an override
- **THEN** `~/.local/bin/notify-me-on-discord` and an executable `~/.local/bin/pingme` entry point are available

#### Scenario: Custom user install directory
- **WHEN** `DISCORD_NOTIFICATION_INSTALL_DIR` points to a writable user directory
- **THEN** the installer places the binary there

### Requirement: Installation preserves user configuration
Installing or upgrading the binary SHALL not overwrite an existing executable-adjacent `config.toml`, `templates/`, or XDG configuration and data directories.

#### Scenario: Upgrade a portable installation
- **WHEN** an existing binary is replaced and sibling configuration already exists
- **THEN** only the binary is replaced

### Requirement: Releases contain prebuilt platform archives
Tagged releases SHALL build versioned archives named `ping-me-in-discord-<tag>-<target>.tar.gz` on Unix and `ping-me-in-discord-<tag>-<target>.zip` on Windows. Each archive SHALL contain the unchanged `notify-me-on-discord` and `pingme` executable entry points for Linux x86_64 and ARM64 using musl, and SHALL additionally be produced for supported GNU/Linux, macOS, and Windows targets when their CI builds succeed. Users SHALL not need a Rust toolchain to run a prebuilt artifact.

#### Scenario: Install on x86_64 Linux
- **WHEN** the installer detects an x86_64 Linux host for tag `v0.1.0`
- **THEN** it downloads `ping-me-in-discord-v0.1.0-x86_64-unknown-linux-musl.tar.gz` and installs the contained `notify-me-on-discord` and `pingme` entry points

#### Scenario: Unsupported host
- **WHEN** no release target matches the detected operating system and architecture
- **THEN** the installer exits with a diagnostic that reports the detected values and manual alternatives

### Requirement: Distribution uses the renamed project identity
Project-facing package metadata, documentation links, the installer's default GitHub repository, and release asset prefixes SHALL identify the project as `ping-me-in-discord` at `memset0/ping-me-in-discord`. Existing executable names and the `discord-notification` configuration and data directory namespace SHALL remain compatible.

#### Scenario: User runs the documented installer
- **WHEN** a user follows the README installation command without repository overrides
- **THEN** the script is fetched from `memset0/ping-me-in-discord` and downloads a `ping-me-in-discord` release archive from that repository

#### Scenario: Existing commands and configuration remain valid
- **WHEN** a user upgrades from the former project identity
- **THEN** `notify-me-on-discord`, `pingme`, and existing configuration below the `discord-notification` application directories continue to work

### Requirement: Release assets are integrity-checkable
Every release archive SHALL have a SHA-256 checksum, and the installer SHALL verify the matching checksum before replacing an installed binary.

#### Scenario: Checksum mismatch
- **WHEN** a downloaded archive does not match its published checksum
- **THEN** installation fails and the existing executable remains unchanged

### Requirement: Release builds are gated by quality checks
Release automation SHALL run formatting checks, compilation checks, lints, and tests before publishing assets.

#### Scenario: Tests fail on a release tag
- **WHEN** any required quality check fails
- **THEN** the workflow does not publish a completed release

### Requirement: The installed binary can initialize its companion files
After installation, users SHALL be able to create either the executable-adjacent `config.toml` and `templates/defaults.md` layout or an XDG user layout using the binary itself.

#### Scenario: First-time portable setup
- **WHEN** a user invokes the portable initialization command after installation
- **THEN** the binary creates the companion files without requiring root access
