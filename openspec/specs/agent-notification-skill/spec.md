# agent-notification-skill Specification

## Purpose

Provide Codex with concise, repeatable workflows for sending Discord notifications while preserving channel intent, conversation identity, and visible failure handling.

## Requirements

### Requirement: A simple Codex notification skill is available
The project SHALL provide a repository-local Codex skill named `discord-notify`. The skill SHALL teach the agent to obtain `CODEX_THREAD_ID`, inspect channels with `pingme channels list --json`, inspect configured avatar profiles with `pingme avatar list --json`, and send freely formatted content through `pingme`. An explicitly requested channel or avatar SHALL take precedence; otherwise the skill SHALL use the effective default channel and SHALL omit the avatar when no suitable configured profile exists.

#### Scenario: User selects configured identities
- **WHEN** the user asks the agent to notify channel `test` with avatar profile `rocket`
- **THEN** the simple skill verifies those names through the JSON inspection commands and sends with `--channel test --avatar rocket`

#### Scenario: No suitable avatar exists
- **WHEN** no avatar is requested and the listed profile descriptions do not match the notification
- **THEN** the simple skill omits all avatar arguments so Discord's default avatar is used

### Requirement: A strict Codex agent notification skill is available
The project SHALL provide a repository-local Codex skill named `discord-agent-notify`. It SHALL choose exactly one built-in notification type without querying configured avatar profiles and SHALL map `started`, `progress`, `success`, `needs-input`, `warning`, and `error` to the same-named configured avatar profile. It SHALL invoke the CLI with `--avatar <status>` and SHALL NOT duplicate the selected profile's emoji, source, colors, dimensions, or scale as one-off avatar arguments. The matching `[avatars.<status>]` profile SHALL be the authoritative visual definition. A delivered notification SHALL visibly use the mapped avatar rather than Discord's default avatar, including when different states are sent consecutively to the same channel.

#### Scenario: Successful completion notification
- **WHEN** the agent reports that requested work completed successfully
- **THEN** the strict skill selects `success` and invokes the CLI with `--avatar success` without avatar styling arguments

#### Scenario: Error notification preserves a white cross
- **WHEN** the agent reports that work or required verification failed
- **THEN** the strict skill invokes the CLI with `--avatar error`, and the configured profile renders its white cross on `#DD2E44` at scale `0.576`

#### Scenario: User input is required
- **WHEN** work cannot continue without a user decision
- **THEN** the strict skill selects `needs-input` and invokes the CLI with `--avatar needs-input` without avatar styling arguments

#### Scenario: Consecutive states retain distinct identities
- **WHEN** the agent sends `started`, `progress`, and `success` notifications consecutively to one channel
- **THEN** each Discord message displays its corresponding rocket, progress, or success avatar instead of sharing or falling back to one default avatar

#### Scenario: Required status profile is absent
- **WHEN** the selected status profile does not exist in the active configuration
- **THEN** the wrapped normal send fails and performs bounded error reporting without synthesizing a one-off fallback avatar

### Requirement: Strict notifications follow one compact message structure
The strict skill SHALL format content as a bold status emoji and short title on the first line, a one- or two-sentence summary on the second line, and an optional bold `Next:` line only when a next action is useful. It SHALL exclude logs, stack traces, credentials, tokens, webhook URLs, and detailed diagnostic reports.

#### Scenario: Progress has a next action
- **WHEN** the agent sends a progress notification with remaining work
- **THEN** the content contains a bold progress title, a concise summary, and a `**Next:**` line describing the next action

#### Scenario: Completion needs no next action
- **WHEN** the agent sends a success notification and no user action is needed
- **THEN** the content ends after the concise summary without an empty or placeholder `Next:` line

### Requirement: Normal agent notifications require a Codex thread ID
Both skills SHALL read the current Codex thread ID from `CODEX_THREAD_ID` before normal delivery and SHALL rely on the selected message template to include that value in its metadata header. The skills SHALL distinguish the Codex thread ID from Discord's `--thread-id` delivery option. If `CODEX_THREAD_ID` is unavailable or empty, the skill SHALL stop the normal send and invoke bounded failure reporting instead of inventing an identifier.

#### Scenario: Current thread is available
- **WHEN** `CODEX_THREAD_ID` contains a non-empty identifier
- **THEN** the skill proceeds and the rendered Discord message includes that identifier in its metadata header

#### Scenario: Current thread is unavailable
- **WHEN** `CODEX_THREAD_ID` is unset or empty
- **THEN** the skill does not send a normal notification and performs one failure-report attempt

### Requirement: Skill CLI failures receive bounded Discord reporting
Every `pingme` invocation prescribed by either skill SHALL run through its bundled safe-execution script. When the wrapped command fails, the script SHALL preserve the original nonzero status and invoke `pingme report-error` exactly once. The error report SHALL contain only a warning emoji, a short failure statement, and the Codex thread ID when available; it SHALL not include the original diagnostic.

#### Scenario: Inspection command fails
- **WHEN** a channel or avatar inspection command exits nonzero before a destination has been selected
- **THEN** the wrapper attempts one error report using the configured default channel and returns the inspection command's original status

#### Scenario: Send command fails
- **WHEN** a send command explicitly selected channel `test` and exits nonzero
- **THEN** the wrapper asks the error reporter to target `test` and then returns the send command's original status

### Requirement: Error reports fall back without recursion
The `report-error` command SHALL send a built-in, template-independent message to the requested channel when that selector resolves and delivery succeeds. If the selector is unknown or delivery to the resolved requested channel fails, it SHALL attempt the configured default channel once when that destination differs. If no channel is supplied, it SHALL use the configured default directly. Failure of the final error-report attempt SHALL be reported locally and SHALL NOT trigger another Discord report.

#### Scenario: Requested alias does not exist
- **WHEN** error reporting receives an unknown nonnumeric channel selector and a valid default channel exists
- **THEN** it sends the short error message to the default channel

#### Scenario: Requested channel is unreachable
- **WHEN** error delivery to a resolved requested channel fails and a different default channel is configured
- **THEN** it makes one additional delivery attempt to the default channel

#### Scenario: Reporting infrastructure is unavailable
- **WHEN** both the requested and default error-report attempts fail because configuration, credentials, or networking is unusable
- **THEN** the command exits nonzero with a local diagnostic and does not recurse
