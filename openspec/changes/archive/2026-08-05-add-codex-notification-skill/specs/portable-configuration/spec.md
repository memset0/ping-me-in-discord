## ADDED Requirements

### Requirement: Configured channels can be inspected without secrets
The CLI SHALL provide `channels list` with human-readable output and a `--json` form for agents. The output SHALL identify every configured alias and channel ID plus the configured default selector and its resolved ID when present. It SHALL NOT include Discord credentials, webhook URLs, state paths, or unrelated configuration values.

#### Scenario: Agent lists configured channels
- **WHEN** configuration defines aliases `default` and `test` and selects `default` in settings
- **THEN** `pingme channels list --json` returns both aliases and IDs and identifies the effective default destination

#### Scenario: No default channel is configured
- **WHEN** channel aliases exist but `[defaults].channel` is absent
- **THEN** the listing returns the aliases and represents the effective default as unset
