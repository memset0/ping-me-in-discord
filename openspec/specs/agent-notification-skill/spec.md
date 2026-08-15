# agent-notification-skill Specification

## Purpose

Provide Codex and Claude Code with concise, repeatable workflows from one canonical source for sending Discord notifications while preserving channel intent, session identity, and visible failure handling.

## Requirements

### Requirement: A simple agent notification skill is available
The project SHALL provide one canonical skill source named `ping-me-send-message` that can be copied for Codex or Claude Code. The skill SHALL be used for intentional free-form Discord messages requested by the user or explicitly required as free-form notifications, and it SHALL NOT be selected for continuing-work updates or end-of-turn outcomes. The skill SHALL teach the agent to obtain the active coding-agent session ID through its runner, inspect channels with `pingme channels list --json`, inspect configured avatar profiles with `pingme avatar list --json`, and send freely formatted content through `pingme`. An explicitly requested channel or avatar SHALL take precedence; otherwise the skill SHALL use the effective default channel and SHALL omit the avatar when no suitable configured profile exists.

#### Scenario: User selects configured identities
- **WHEN** the user asks the agent to send a free-form message to channel `test` with avatar profile `rocket`
- **THEN** `ping-me-send-message` verifies those names through the JSON inspection commands and sends with `--channel test --avatar rocket`

#### Scenario: No suitable avatar exists
- **WHEN** no avatar is requested and the listed profile descriptions do not match the free-form notification
- **THEN** `ping-me-send-message` omits all avatar arguments so Discord's default avatar is used

#### Scenario: Lifecycle report does not select the free-form skill
- **WHEN** an agent needs to report either continuing work or the outcome immediately before yielding to the user
- **THEN** it selects `ping-me-report-work-progress` or `ping-me-report-turn-outcome` instead of `ping-me-send-message`

### Requirement: Continuing work has a dedicated notification skill
The project SHALL provide one canonical skill source named `ping-me-report-work-progress` for structured notifications sent while the agent continues working. It SHALL accept only `started`, `progress`, `warning`, and recoverable `error` states, map each state to the same-named configured avatar profile, and invoke the CLI with `--avatar <status>` without querying or reproducing the profile's visual settings. It SHALL NOT emit successful completion, requests for user input, or terminal failure outcomes, and it SHALL direct those cases to `ping-me-report-turn-outcome`.

#### Scenario: Work begins
- **WHEN** enabled progress reporting announces the beginning of requested work
- **THEN** `ping-me-report-work-progress` sends a `started` notification with `--avatar started` and the agent continues working

#### Scenario: Meaningful progress occurs
- **WHEN** the agent completes a meaningful stage but has more work to do
- **THEN** `ping-me-report-work-progress` sends `progress` with `--avatar progress`

#### Scenario: Recoverable problem occurs
- **WHEN** a step fails but the agent has an in-scope recovery path and continues working
- **THEN** `ping-me-report-work-progress` may send `error` with `--avatar error` and states the next recovery action

#### Scenario: Agent is about to yield
- **WHEN** the agent is about to return a final response or wait for user input
- **THEN** it does not use `ping-me-report-work-progress` for that notification and evaluates `ping-me-report-turn-outcome`

### Requirement: End-of-turn outcomes have a dedicated notification skill
The project SHALL provide one canonical skill source named `ping-me-report-turn-outcome` for exactly one structured outcome notification immediately before the agent returns its final response or blocks waiting for user input. It SHALL choose `success` when requested work completed, `needs-input` when a user decision or missing input prevents continuation, `warning` when the turn completes with an important limitation, or `error` when requested work or required verification terminates unsuccessfully. It SHALL map the selected state to the same-named configured avatar profile and invoke the CLI with `--avatar <status>` without querying or reproducing visual settings. Notification failure SHALL NOT suppress the agent's user-facing response.

#### Scenario: Turn completes successfully
- **WHEN** enabled outcome reporting reaches a final response after successful work
- **THEN** the agent sends exactly one `success` notification immediately before yielding and then returns the response

#### Scenario: User input is required
- **WHEN** the agent cannot continue without a user decision or missing value
- **THEN** it sends exactly one `needs-input` notification immediately before asking the user

#### Scenario: Turn ends with a material limitation
- **WHEN** work is usable but the final response must highlight an important limitation or incomplete verification
- **THEN** the agent sends exactly one `warning` outcome rather than a success outcome

#### Scenario: Turn terminates unsuccessfully
- **WHEN** the requested work or required verification cannot be completed and no recovery path remains in the turn
- **THEN** the agent sends exactly one `error` outcome immediately before returning the failure explanation

### Requirement: Automatic notification activation is conversation-scoped
Explicit invocation of `ping-me-report-work-progress` or `ping-me-report-turn-outcome` SHALL enable that skill's policy for the remainder of the current agent conversation. On later turns in that same conversation, the agent SHALL continue applying each enabled policy without requiring the user to invoke the skill again, until the user explicitly disables that notification policy. Activation SHALL NOT carry into a new conversation and SHALL NOT depend on Herdr injection, hooks, or machine-persistent state.

#### Scenario: Outcome policy persists to the next turn
- **WHEN** the user explicitly activates `ping-me-report-turn-outcome` and then sends another request in the same conversation
- **THEN** the agent sends one outcome notification before yielding on the later turn without another activation request

#### Scenario: Both automatic policies are active
- **WHEN** the user activates both automatic notification skills in one conversation
- **THEN** later turns may emit continuing-work notifications and SHALL emit exactly one outcome notification before each yield

#### Scenario: User disables notifications
- **WHEN** the user explicitly asks to stop one or both automatic notification policies
- **THEN** the agent no longer sends notifications governed by the disabled policy on later turns

#### Scenario: A new conversation begins
- **WHEN** the user starts a separate agent conversation without activating either policy there
- **THEN** activation from the previous conversation does not apply

### Requirement: Automatic notifications carry stable agent context
For an enabled automatic notification policy, the agent SHALL establish one concise session title on first activation and reuse it throughout the conversation. Every normal runner invocation SHALL provide or discover a normalized agent name, project name, session ID, and, when established, session title so the selected template can identify the notification source. The runner SHALL infer supported agent and project values when explicit values are absent, but SHALL leave the session title absent rather than inventing one so the default template can fall back to the full session ID. The compatibility `runtime.session.name` MAY still derive a deterministic name for custom templates.

#### Scenario: Agent establishes a conversation name
- **WHEN** an automatic notification skill is activated for work on notification skill design
- **THEN** the agent chooses a concise stable title such as `notification-skill-design` and uses it for every notification in that conversation

#### Scenario: Context options are omitted
- **WHEN** a skill runner is invoked without explicit agent, project, or session-name values but a supported agent session ID and project directory are available
- **THEN** it infers the agent and project, leaves the session title absent, and exposes the full session ID for the template fallback

#### Scenario: Context value contains unsafe formatting
- **WHEN** a discovered or explicit context value contains control characters, backticks, or repeated whitespace
- **THEN** the rendered metadata uses its normalized single-line inline-code-safe form

### Requirement: Skill instructions render as valid GitHub Flavored Markdown
Each bundled `SKILL.md` SHALL render its headings, ordered workflow, examples, and failure rules as their intended GitHub Flavored Markdown structures. A fenced command example SHALL NOT absorb later workflow steps or headings.

#### Scenario: Strict skill example remains bounded
- **WHEN** GitHub Flavored Markdown renders a multi-line dry-run example in either automatic notification skill
- **THEN** only the command example is rendered as code and the remaining workflow steps plus `Failure rules` remain normal document content

### Requirement: Strict notifications follow one compact message structure
Both automatic notification skills SHALL format content as a bold status emoji and short title on the first line, a one- or two-sentence summary on the second line, and an optional bold `Next:` line only when a next action is useful. They SHALL exclude logs, stack traces, credentials, tokens, webhook URLs, and detailed diagnostic reports.

#### Scenario: Progress has a next action
- **WHEN** the agent sends a progress notification with remaining work
- **THEN** the content contains a bold progress title, a concise summary, and a `**Next:**` line describing the next action

#### Scenario: Completion needs no next action
- **WHEN** the agent sends a successful turn outcome and no user action is needed
- **THEN** the content ends after the concise summary without an empty or placeholder `Next:` line

### Requirement: Normal agent notifications require an agent session ID
All three skills SHALL obtain the current agent session ID from their runner before normal delivery. The runner SHALL prefer a non-empty explicit generic session ID, otherwise a non-empty `CLAUDE_CODE_SESSION_ID` supplied by Claude Code, and otherwise a non-empty `CODEX_THREAD_ID` supplied by Codex. It SHALL expose the selected value as generic template runtime metadata and through the legacy Codex compatibility field without confusing it with Discord's `--thread-id` delivery option. Before delivery, each skill SHALL verify that the rendered content includes the exact established session title when one is present, or the exact agent session ID when no title is present. If no supported identifier is available or the expected session label is absent from the rendered content, the skill SHALL stop the normal send and invoke bounded failure reporting instead of inventing an identifier.

#### Scenario: Current thread is available
- **WHEN** `CODEX_THREAD_ID` contains a non-empty identifier, no higher-priority session identifier is present, and no session title was established
- **THEN** the skill proceeds and the rendered Discord message includes the Codex identifier through the session-ID fallback

#### Scenario: Current thread is unavailable
- **WHEN** all supported session-ID inputs are unset or empty
- **THEN** the skill does not send a normal notification and performs one failure-report attempt

#### Scenario: Current Claude Code session is available
- **WHEN** `CLAUDE_CODE_SESSION_ID` contains a non-empty identifier, no explicit generic identifier is present, and no session title was established
- **THEN** the skill proceeds and the rendered Discord message includes the Claude Code identifier through the session-ID fallback

#### Scenario: Claude Code identifier wins over a stale Codex value
- **WHEN** Claude Code and Codex identifiers are both present and no explicit generic identifier is set
- **THEN** the runner selects the Claude Code identifier for runtime context and any required session-ID fallback

#### Scenario: Explicit generic identifier wins
- **WHEN** a non-empty generic session identifier and adapter-specific session identifiers are present
- **THEN** the runner selects the explicit generic identifier for runtime context and any required session-ID fallback

#### Scenario: Established session title is the rendered label
- **WHEN** the agent establishes a session title and a valid session ID is also available
- **THEN** the skill validates the exact title in rendered content instead of requiring the hidden fallback ID to be rendered

### Requirement: Skill CLI failures receive bounded Discord reporting
Every `pingme` invocation prescribed by any bundled notification skill SHALL run through its bundled safe-execution script. When the wrapped command fails, the script SHALL preserve the original nonzero status and invoke `pingme report-error` exactly once. The error report SHALL contain only a warning emoji, a short failure statement, and the active agent session ID when available; it SHALL not include the original diagnostic.

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
