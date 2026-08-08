## MODIFIED Requirements

### Requirement: Plain message invocation uses the default template
Both `ping-me-in-discord` and its `pingme` alias SHALL accept a message as their top-level positional argument. The CLI SHALL expose that value as the `message` template variable and render `templates/defaults.md`. The former `notify-me-on-discord` executable SHALL no longer be produced.

#### Scenario: Send through the short alias
- **WHEN** the user invokes `pingme 'message content'`
- **THEN** the CLI renders `defaults.md` with `message` equal to `message content` and sends the resulting payload to the configured Discord channel

#### Scenario: Full binary name has the same shorthand
- **WHEN** the user invokes `ping-me-in-discord 'message content'`
- **THEN** it performs the same default-template send behavior as `pingme`

#### Scenario: Former binary name is retired
- **WHEN** the project builds its declared binary targets
- **THEN** it produces `ping-me-in-discord` and `pingme` without producing `notify-me-on-discord`
