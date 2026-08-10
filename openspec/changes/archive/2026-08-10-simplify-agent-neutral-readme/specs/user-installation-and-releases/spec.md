## ADDED Requirements

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
