## Purpose

Define predictable, rootless configuration and state discovery while preserving a self-contained layout beside the installed executable.

## ADDED Requirements

### Requirement: Configuration discovery is deterministic
The CLI SHALL load the first available configuration in this order: an explicit `--config` path, `DISCORD_NOTIFICATION_CONFIG`, `config.toml` beside the running executable, and the platform user configuration directory. The CLI SHALL report the selected path through a configuration inspection command.

#### Scenario: Portable configuration beside the binary
- **WHEN** `config.toml` exists in the directory containing the running executable and no higher-priority path is supplied
- **THEN** the CLI loads that file without requiring the current working directory to match the executable directory

#### Scenario: Explicit configuration overrides portable configuration
- **WHEN** the user supplies `--config /path/custom.toml` and another configuration exists beside the executable
- **THEN** the CLI loads `/path/custom.toml`

#### Scenario: User configuration fallback
- **WHEN** no explicit, environment, or executable-adjacent configuration exists
- **THEN** the CLI looks under the platform user configuration directory, using `$XDG_CONFIG_HOME/discord-notification/config.toml` or `~/.config/discord-notification/config.toml` on Linux

### Requirement: Template paths are relative to their configuration
The CLI SHALL resolve a relative template directory from the directory containing the selected configuration file, and SHALL default that directory to `templates`.

#### Scenario: Sibling templates directory
- **WHEN** `/opt/notify/config.toml` is selected and no template directory is configured
- **THEN** templates are loaded from `/opt/notify/templates`

### Requirement: Initialization supports portable and user layouts
The CLI SHALL provide an initialization command that can create either an executable-adjacent portable layout or a platform user configuration layout. Initialization SHALL create a starter `config.toml` and `templates/defaults.md`, and SHALL refuse to overwrite existing user files unless explicitly forced.

#### Scenario: Initialize a portable layout
- **WHEN** the user runs initialization in portable mode and the executable directory is writable
- **THEN** the starter configuration and templates are created beside the executable

#### Scenario: Existing configuration is protected
- **WHEN** initialization targets a location containing `config.toml` and force is not requested
- **THEN** the command fails without changing the existing file

### Requirement: Credentials support environment overrides
The CLI SHALL accept webhook URLs and Bot tokens from configuration, while `DISCORD_NOTIFICATION_WEBHOOK_URL` and `DISCORD_NOTIFICATION_BOT_TOKEN` SHALL override file values. Secret values SHALL be redacted from diagnostics, dry-run output, and errors.

#### Scenario: Environment secret wins
- **WHEN** both `discord.webhook_url` and `DISCORD_NOTIFICATION_WEBHOOK_URL` are set
- **THEN** delivery uses the environment value and does not display either secret

### Requirement: Channel aliases are configurable
Configuration SHALL accept a `[channels]` mapping whose keys are human-readable aliases and whose values are Discord channel IDs. Any channel selector from CLI arguments, template frontmatter, or `[defaults].channel` SHALL resolve an exact alias match before accepting a numeric channel ID directly. Unknown nonnumeric selectors and invalid configured IDs SHALL fail before a network request.

#### Scenario: CLI uses a configured alias
- **WHEN** `[channels]` maps `alerts` to `123456789012345678` and the user supplies `--channel alerts`
- **THEN** delivery targets channel `123456789012345678`

#### Scenario: Numeric channel bypasses aliases
- **WHEN** the user supplies a numeric Discord channel ID directly
- **THEN** that ID is used without requiring an alias entry

#### Scenario: Unknown alias
- **WHEN** a selected channel is neither a configured alias nor a numeric ID
- **THEN** the CLI fails with an error naming the unknown alias

### Requirement: Persistent state is user-owned
The CLI SHALL keep provisioned webhook state and downloaded avatar assets in a user-specific data directory, using `$XDG_DATA_HOME/discord-notification` or `~/.local/share/discord-notification` on Linux. Secret-bearing files SHALL be created with owner-only permissions on Unix.

#### Scenario: Provisioned webhook is cached without root
- **WHEN** the CLI provisions a webhook on Linux with no XDG override
- **THEN** it writes reusable state below `~/.local/share/discord-notification` and does not access a system-wide directory

### Requirement: Configuration can be validated offline
The CLI SHALL provide a validation command that parses configuration, resolves templates and local avatar assets, and reports actionable field-level errors without sending a Discord message.

#### Scenario: Missing templates are diagnosed
- **WHEN** configuration points to a template directory that does not contain `defaults.md`
- **THEN** validation fails and names the expected file path
