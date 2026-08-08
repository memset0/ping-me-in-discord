## MODIFIED Requirements

### Requirement: Releases contain prebuilt platform archives
Tagged releases SHALL build versioned archives named `ping-me-in-discord-<tag>-<target>.tar.gz` on Unix and `ping-me-in-discord-<tag>-<target>.zip` on Windows. Each archive SHALL contain the unchanged `notify-me-on-discord` and `pingme` executable entry points for Linux x86_64 and ARM64 using musl, and SHALL additionally be produced for supported GNU/Linux, macOS, and Windows targets when their CI builds succeed. Users SHALL not need a Rust toolchain to run a prebuilt artifact.

#### Scenario: Install on x86_64 Linux
- **WHEN** the installer detects an x86_64 Linux host for tag `v0.1.0`
- **THEN** it downloads `ping-me-in-discord-v0.1.0-x86_64-unknown-linux-musl.tar.gz` and installs the contained `notify-me-on-discord` and `pingme` entry points

#### Scenario: Unsupported host
- **WHEN** no release target matches the detected operating system and architecture
- **THEN** the installer exits with a diagnostic that reports the detected values and manual alternatives

## ADDED Requirements

### Requirement: Distribution uses the renamed project identity
Project-facing package metadata, documentation links, the installer's default GitHub repository, and release asset prefixes SHALL identify the project as `ping-me-in-discord` at `memset0/ping-me-in-discord`. Existing executable names and the `discord-notification` configuration and data directory namespace SHALL remain compatible.

#### Scenario: User runs the documented installer
- **WHEN** a user follows the README installation command without repository overrides
- **THEN** the script is fetched from `memset0/ping-me-in-discord` and downloads a `ping-me-in-discord` release archive from that repository

#### Scenario: Existing commands and configuration remain valid
- **WHEN** a user upgrades from the former project identity
- **THEN** `notify-me-on-discord`, `pingme`, and existing configuration below the `discord-notification` application directories continue to work
