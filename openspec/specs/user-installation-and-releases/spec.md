# user-installation-and-releases Specification

## Purpose

Define reproducible standalone releases and a no-root installation experience centered on common Linux user-directory conventions.

## Requirements

### Requirement: Linux installation does not require root
The Unix installer SHALL place `ping-me-in-discord` in `$DISCORD_NOTIFICATION_INSTALL_DIR` when set and otherwise in `~/.local/bin`, and SHALL install `pingme` as an equivalent entry point. It SHALL create the target directory as the current user and SHALL never invoke `sudo` or write to system directories by default. After both current entry points are installed successfully, an upgrade SHALL remove only the exact legacy `notify-me-on-discord` path in the same installation directory when it exists.

#### Scenario: Default Linux install
- **WHEN** a Linux user runs the installer without an override
- **THEN** `~/.local/bin/ping-me-in-discord` and an executable `~/.local/bin/pingme` entry point are available

#### Scenario: Custom user install directory
- **WHEN** `DISCORD_NOTIFICATION_INSTALL_DIR` points to a writable user directory
- **THEN** the installer places both current entry points there

#### Scenario: Upgrade retires the legacy executable
- **WHEN** installation of `ping-me-in-discord` and `pingme` succeeds in a directory containing the former `notify-me-on-discord` executable
- **THEN** the installer removes that exact legacy path and leaves other files untouched

### Requirement: Installation preserves user configuration
Installing or upgrading the binaries SHALL not overwrite an existing executable-adjacent `config.toml`, `templates/`, or XDG configuration and data directories. Retiring the exact legacy executable SHALL not rename or remove those companion files or directories.

#### Scenario: Upgrade a portable installation
- **WHEN** current entry points replace an older installation and sibling configuration already exists
- **THEN** only managed executable entry points change while `config.toml` and `templates/` remain unchanged

### Requirement: Releases contain prebuilt platform archives
Tagged releases SHALL build versioned archives named `ping-me-in-discord-<tag>-<target>.tar.gz` on Unix and `ping-me-in-discord-<tag>-<target>.zip` on Windows. Each archive SHALL contain the `ping-me-in-discord` and `pingme` executable entry points for Linux x86_64 and ARM64 using musl, and SHALL additionally be produced for supported GNU/Linux, macOS, and Windows targets when their CI builds succeed. Users SHALL not need a Rust toolchain to run a prebuilt artifact.

#### Scenario: Install on x86_64 Linux
- **WHEN** the installer detects an x86_64 Linux host for tag `v0.1.0`
- **THEN** it downloads `ping-me-in-discord-v0.1.0-x86_64-unknown-linux-musl.tar.gz` and installs the contained `ping-me-in-discord` and `pingme` entry points

#### Scenario: Unsupported host
- **WHEN** no release target matches the detected operating system and architecture
- **THEN** the installer exits with a diagnostic that reports the detected values and manual alternatives

### Requirement: Distribution uses the renamed project identity
Project-facing package metadata, documentation links, the installer's default GitHub repository, release asset prefixes, and the canonical long executable SHALL identify the project as `ping-me-in-discord` at `memset0/ping-me-in-discord`. The `pingme` short entry point and the existing `discord-notification` configuration and data directory namespace SHALL remain compatible; the former `notify-me-on-discord` executable SHALL be retired.

#### Scenario: User runs the documented installer
- **WHEN** a user follows the README installation command without repository overrides
- **THEN** the script is fetched from `memset0/ping-me-in-discord` and installs `ping-me-in-discord` plus `pingme` from a `ping-me-in-discord` release archive

#### Scenario: Existing commands and configuration remain valid
- **WHEN** a user upgrades from the former project identity or executable name
- **THEN** `pingme` remains valid, `ping-me-in-discord` replaces the former long entry point, and both commands continue to discover existing configuration below the `discord-notification` application directories

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

### Requirement: README provides one concise setup flow
The English README SHALL introduce `pingme` and `ping-me-in-discord` once as equivalent executable entry points, then use `pingme` for subsequent CLI command examples. It SHALL combine binary installation, configuration initialization and editing, configuration validation, and optional agent-skill installation into one ordered setup section instead of separate introductory installation and initialization sections. The setup SHALL retain the rootless default, identify the generated `config.toml` and `templates/defaults.md`, and link to detailed references instead of repeating exhaustive configuration or release behavior.

#### Scenario: New user follows setup in order
- **WHEN** a reader starts with no installed binary or configuration
- **THEN** one setup section leads them through installing the release, running `pingme init`, editing and validating `config.toml`, and optionally installing the notification skills

#### Scenario: Command examples use the short entry point
- **WHEN** the README demonstrates a CLI command after introducing the two equivalent executables
- **THEN** the command begins with `pingme` rather than `ping-me-in-discord`

#### Scenario: Both executable names remain discoverable
- **WHEN** a reader opens the README introduction
- **THEN** it states that `pingme` and `ping-me-in-discord` are equivalent entry points even though later examples use only `pingme`

#### Scenario: Setup remains concise
- **WHEN** details are already covered by the configuration or release reference documentation
- **THEN** the README summarizes the essential setup behavior and links to the reference instead of duplicating the full explanation
