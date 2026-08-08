## RENAMED Requirements

- FROM: `### Requirement: A simple Codex notification skill is available`
- TO: `### Requirement: A simple agent notification skill is available`
- FROM: `### Requirement: A strict Codex agent notification skill is available`
- TO: `### Requirement: A strict agent notification skill is available`
- FROM: `### Requirement: Normal agent notifications require a Codex thread ID`
- TO: `### Requirement: Normal agent notifications require an agent session ID`

## MODIFIED Requirements

### Requirement: A simple agent notification skill is available
The project SHALL provide one canonical skill source named `ping-me-send-message` that can be copied for Codex or Claude Code. The skill SHALL be used for intentional free-form Discord messages requested by the user or explicitly required as free-form notifications, and it SHALL NOT be selected merely to report an agent lifecycle state. The skill SHALL teach the agent to obtain the active coding-agent session ID through its runner, inspect channels with `pingme channels list --json`, inspect configured avatar profiles with `pingme avatar list --json`, and send freely formatted content through `pingme`. An explicitly requested channel or avatar SHALL take precedence; otherwise the skill SHALL use the effective default channel and SHALL omit the avatar when no suitable configured profile exists.

#### Scenario: User selects configured identities
- **WHEN** the user asks the agent to send a free-form message to channel `test` with avatar profile `rocket`
- **THEN** `ping-me-send-message` verifies those names through the JSON inspection commands and sends with `--channel test --avatar rocket`

#### Scenario: No suitable avatar exists
- **WHEN** no avatar is requested and the listed profile descriptions do not match the free-form notification
- **THEN** `ping-me-send-message` omits all avatar arguments so Discord's default avatar is used

#### Scenario: Lifecycle report does not select the free-form skill
- **WHEN** an agent needs to report a `started`, `progress`, `success`, `needs-input`, `warning`, or `error` lifecycle state
- **THEN** it selects `ping-me-report-agent-status` instead of `ping-me-send-message`

### Requirement: A strict agent notification skill is available
The project SHALL provide one canonical skill source named `ping-me-report-agent-status` that can be copied for Codex or Claude Code. It SHALL be used only for structured agent lifecycle reports and SHALL NOT be used for arbitrary free-form messages or custom avatar selection. It SHALL choose exactly one built-in notification type without querying configured avatar profiles and SHALL map `started`, `progress`, `success`, `needs-input`, `warning`, and `error` to the same-named configured avatar profile. It SHALL invoke the CLI with `--avatar <status>` and SHALL NOT duplicate the selected profile's emoji, source, colors, dimensions, or scale as one-off avatar arguments. The matching `[avatars.<status>]` profile SHALL be the authoritative visual definition. A delivered notification SHALL visibly use the mapped avatar rather than Discord's default avatar, including when different states are sent consecutively to the same channel.

#### Scenario: Successful completion notification
- **WHEN** the agent reports that requested work completed successfully
- **THEN** `ping-me-report-agent-status` selects `success` and invokes the CLI with `--avatar success` without avatar styling arguments

#### Scenario: Error notification preserves a white cross
- **WHEN** the agent reports that work or required verification failed
- **THEN** `ping-me-report-agent-status` invokes the CLI with `--avatar error`, and the configured profile renders its white cross on `#DD2E44` at scale `0.576`

#### Scenario: User input is required
- **WHEN** work cannot continue without a user decision
- **THEN** `ping-me-report-agent-status` selects `needs-input` and invokes the CLI with `--avatar needs-input` without avatar styling arguments

#### Scenario: Consecutive states retain distinct identities
- **WHEN** the agent sends `started`, `progress`, and `success` notifications consecutively to one channel
- **THEN** each Discord message displays its corresponding rocket, progress, or success avatar instead of sharing or falling back to one default avatar

#### Scenario: Required status profile is absent
- **WHEN** the selected status profile does not exist in the active configuration
- **THEN** the wrapped normal send fails and performs bounded error reporting without synthesizing a one-off fallback avatar

#### Scenario: Free-form message does not select the lifecycle skill
- **WHEN** the user asks to send arbitrary Discord Markdown or choose a custom avatar without assigning a lifecycle state
- **THEN** the agent selects `ping-me-send-message` instead of `ping-me-report-agent-status`

### Requirement: Normal agent notifications require an agent session ID
Both skills SHALL obtain the current agent session ID from their runner before normal delivery and SHALL rely on the selected message template to include that value in its metadata header. The runner SHALL prefer a non-empty `CLAUDE_CODE_SESSION_ID` supplied by Claude Code and otherwise use a non-empty `CODEX_THREAD_ID` supplied by Codex. It SHALL expose the selected value to the existing CLI template runtime without confusing it with Discord's `--thread-id` delivery option. If neither identifier is available, the skill SHALL stop the normal send and invoke bounded failure reporting instead of inventing an identifier.

#### Scenario: Current thread is available
- **WHEN** `CODEX_THREAD_ID` contains a non-empty identifier and no Claude Code session identifier is present
- **THEN** the skill proceeds and the rendered Discord message includes the Codex identifier in its metadata header

#### Scenario: Current thread is unavailable
- **WHEN** both supported session-ID environment variables are unset or empty
- **THEN** the skill does not send a normal notification and performs one failure-report attempt

#### Scenario: Current Claude Code session is available
- **WHEN** `CLAUDE_CODE_SESSION_ID` contains a non-empty identifier
- **THEN** the skill proceeds and the rendered Discord message includes the Claude Code identifier in its metadata header

#### Scenario: Claude Code identifier wins over a stale Codex value
- **WHEN** both environment variables are present in a Claude Code Bash subprocess
- **THEN** the runner selects `CLAUDE_CODE_SESSION_ID` for the rendered metadata header

### Requirement: Skill CLI failures receive bounded Discord reporting
Every `pingme` invocation prescribed by either skill SHALL run through its bundled safe-execution script. When the wrapped command fails, the script SHALL preserve the original nonzero status and invoke `pingme report-error` exactly once. The error report SHALL contain only a warning emoji, a short failure statement, and the active agent session ID when available; it SHALL not include the original diagnostic.

#### Scenario: Inspection command fails
- **WHEN** a channel or avatar inspection command exits nonzero before a destination has been selected
- **THEN** the wrapper attempts one error report using the configured default channel and returns the inspection command's original status

#### Scenario: Send command fails
- **WHEN** a send command explicitly selected channel `test` and exits nonzero
- **THEN** the wrapper asks the error reporter to target `test` and then returns the send command's original status
