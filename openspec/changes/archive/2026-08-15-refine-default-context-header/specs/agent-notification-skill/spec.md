## MODIFIED Requirements

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
