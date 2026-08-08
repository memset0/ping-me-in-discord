## MODIFIED Requirements

### Requirement: A simple Codex notification skill is available
The project SHALL provide a repository-local Codex skill named `ping-me-send-message`. The skill SHALL be used for intentional free-form Discord messages requested by the user or explicitly required as free-form notifications, and it SHALL NOT be selected merely to report an agent lifecycle state. The skill SHALL teach the agent to obtain `CODEX_THREAD_ID`, inspect channels with `pingme channels list --json`, inspect configured avatar profiles with `pingme avatar list --json`, and send freely formatted content through `pingme`. An explicitly requested channel or avatar SHALL take precedence; otherwise the skill SHALL use the effective default channel and SHALL omit the avatar when no suitable configured profile exists.

#### Scenario: User selects configured identities
- **WHEN** the user asks the agent to send a free-form message to channel `test` with avatar profile `rocket`
- **THEN** `ping-me-send-message` verifies those names through the JSON inspection commands and sends with `--channel test --avatar rocket`

#### Scenario: No suitable avatar exists
- **WHEN** no avatar is requested and the listed profile descriptions do not match the free-form notification
- **THEN** `ping-me-send-message` omits all avatar arguments so Discord's default avatar is used

#### Scenario: Lifecycle report does not select the free-form skill
- **WHEN** an agent needs to report a `started`, `progress`, `success`, `needs-input`, `warning`, or `error` lifecycle state
- **THEN** it selects `ping-me-report-agent-status` instead of `ping-me-send-message`

### Requirement: A strict Codex agent notification skill is available
The project SHALL provide a repository-local Codex skill named `ping-me-report-agent-status`. It SHALL be used only for structured agent lifecycle reports and SHALL NOT be used for arbitrary free-form messages or custom avatar selection. It SHALL choose exactly one built-in notification type without querying configured avatar profiles and SHALL map `started`, `progress`, `success`, `needs-input`, `warning`, and `error` to the same-named configured avatar profile. It SHALL invoke the CLI with `--avatar <status>` and SHALL NOT duplicate the selected profile's emoji, source, colors, dimensions, or scale as one-off avatar arguments. The matching `[avatars.<status>]` profile SHALL be the authoritative visual definition. A delivered notification SHALL visibly use the mapped avatar rather than Discord's default avatar, including when different states are sent consecutively to the same channel.

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

## ADDED Requirements

### Requirement: Skill instructions render as valid GitHub Flavored Markdown
Each bundled `SKILL.md` SHALL render its headings, ordered workflow, examples, and failure rules as their intended GitHub Flavored Markdown structures. A fenced command example SHALL NOT absorb later workflow steps or headings.

#### Scenario: Strict skill example remains bounded
- **WHEN** GitHub Flavored Markdown renders the multi-line dry-run example in `ping-me-report-agent-status/SKILL.md`
- **THEN** only the command example is rendered as code and workflow steps 7 and 8 plus `Failure rules` remain normal document content
