## Why

The project's long CLI entry point still uses the former `notify-me-on-discord` name even though the project and release identity are now `ping-me-in-discord`. Aligning the long command with the project name leaves users with one concise alias and one predictable canonical executable.

## What Changes

- **BREAKING**: Replace the `notify-me-on-discord` executable entry point with `ping-me-in-discord`.
- Keep `pingme` as the equivalent short entry point.
- Package both new entry points in release archives and make the Unix installer install them without root privileges.
- Make installer upgrades remove only the exact legacy `notify-me-on-discord` executable after both replacement entry points are installed successfully.
- Update CLI diagnostics, initialization comments, tests, README examples, configuration documentation, release documentation, and release automation to use the new long command.
- Preserve the existing `discord-notification` configuration/data directory and `DISCORD_NOTIFICATION_*` environment-variable namespaces.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-message-templates`: Replace the old long executable name in shorthand message behavior with `ping-me-in-discord` while preserving `pingme` equivalence.
- `user-installation-and-releases`: Install and release `ping-me-in-discord` plus `pingme`, retire the exact legacy executable on upgrade, and preserve user configuration namespaces.

## Impact

- Affected build surfaces: Cargo binary target and source entry-point filename.
- Affected distribution surfaces: release archives, GitHub Actions packaging, and `install.sh` upgrade behavior.
- Affected user interfaces: shell commands, help and error examples, README, configuration examples, and tests.
- Existing scripts that explicitly invoke `notify-me-on-discord` must switch to `ping-me-in-discord` or `pingme`.
