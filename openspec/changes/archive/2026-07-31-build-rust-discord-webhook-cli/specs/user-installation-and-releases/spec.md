## Purpose

Define reproducible standalone releases and a no-root installation experience centered on common Linux user-directory conventions.

## ADDED Requirements

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
Tagged releases SHALL build versioned archives containing `notify-me-on-discord` and `pingme` for Linux x86_64 and ARM64 using musl, and SHALL additionally produce supported GNU/Linux, macOS, and Windows archives when their CI targets succeed. Users SHALL not need a Rust toolchain to run a prebuilt artifact.

#### Scenario: Install on x86_64 Linux
- **WHEN** the installer detects an x86_64 Linux host
- **THEN** it downloads the matching prebuilt Linux archive

#### Scenario: Unsupported host
- **WHEN** no release target matches the detected operating system and architecture
- **THEN** the installer exits with a diagnostic that reports the detected values and manual alternatives

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
