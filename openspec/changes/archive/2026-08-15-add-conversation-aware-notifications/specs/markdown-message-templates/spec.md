## MODIFIED Requirements

### Requirement: Templates receive automatic runtime metadata
Every template render SHALL expose a reserved `runtime` object. It SHALL contain non-empty `user` and `hostname` strings; nested non-empty `agent.name`, `project.name`, and `session.name` strings; an optional `session.id`; a `timestamp` object whose `local`, `unix`, and `iso8601` fields represent one captured invocation instant; and an optional legacy `codex_thread_id` alias equal to `session.id`. Explicit non-empty `PINGME_AGENT_NAME`, `PINGME_PROJECT_NAME`, `PINGME_SESSION_NAME`, and `PINGME_SESSION_ID` values SHALL take precedence for their matching fields. Without explicit values, the CLI SHALL infer the supported agent from its session environment or identify direct use as `CLI`, infer the project from the current directory with an `unknown-project` fallback, and derive `session.name` as `session-<first eight session-ID characters>` or `interactive` when no ID exists. Session ID discovery SHALL prefer `PINGME_SESSION_ID`, then `CLAUDE_CODE_SESSION_ID`, then `CODEX_THREAD_ID`. All string values SHALL be normalized for one-line Discord inline code. `timestamp.local` SHALL use the system local time in `M/D HH:mm:ss` form, `timestamp.unix` SHALL be whole Unix seconds, and `timestamp.iso8601` SHALL be a UTC ISO 8601 string.

#### Scenario: Runtime values are available without caller data
- **WHEN** a template references the identity, agent, project, session, and timestamp runtime fields without matching `--data` or `--var` input
- **THEN** the CLI renders every required field from automatically discovered runtime metadata

#### Scenario: Explicit agent context is available
- **WHEN** the four non-empty `PINGME_*` context variables identify `Codex`, `ping-me-in-discord`, `notification-skill-design`, and session `019fb637`
- **THEN** the corresponding nested runtime fields contain those normalized values and `runtime.codex_thread_id` also equals `019fb637`

#### Scenario: Codex thread ID is available
- **WHEN** no generic session ID is set and `CODEX_THREAD_ID` contains a non-empty current conversation identifier
- **THEN** `runtime.session.id` and `runtime.codex_thread_id` contain that normalized identifier without caller data

#### Scenario: Codex thread ID is unavailable
- **WHEN** every supported session-ID environment variable is unset or contains only whitespace
- **THEN** `runtime.session.id` and `runtime.codex_thread_id` are unset, `runtime.session.name` is `interactive`, and other runtime metadata remains available

#### Scenario: Generic session ID has priority
- **WHEN** generic, Claude Code, and Codex session identifiers are all present
- **THEN** `runtime.session.id` and its compatibility alias contain the generic identifier

#### Scenario: Timestamp fields share one instant
- **WHEN** a template renders local, Unix, and ISO 8601 timestamp representations
- **THEN** all three values describe the same instant captured once for that send

#### Scenario: Identity discovery fails
- **WHEN** the operating system cannot provide the current user, hostname, or current project directory
- **THEN** rendering continues with `unknown-user`, `unknown-host`, or `unknown-project` for the unavailable field

#### Scenario: Caller attempts to replace runtime metadata
- **WHEN** JSON data or a variable override defines the reserved top-level key `runtime`
- **THEN** the CLI rejects the collision before rendering or making a network request

### Requirement: The starter template identifies the emitter before content
Newly initialized `templates/defaults.md` SHALL render exactly two bold blockquote metadata lines followed immediately by the caller's Markdown content on the next line. The first line SHALL contain the robot emoji and inline-code agent name, three spaces, the package emoji and inline-code project name, three spaces, and the speech-balloon emoji and inline-code session name. The second line SHALL contain the house emoji and inline-code `user@hostname`, three spaces, the calendar emoji and inline-code local timestamp, and, when `runtime.session.id` is present, three spaces, the thread emoji and inline-code session ID. It SHALL not insert a blank line between either metadata line or the content.

#### Scenario: New starter template includes a Codex thread
- **WHEN** runtime metadata is agent `Codex`, project `ping-me-in-discord`, session name `notification-skill-design`, user `mem`, host `vultr`, timestamp `8/14 12:00:11`, and session ID `019fb637`, and the caller sends `build complete`
- **THEN** content is ``> **🤖 `Codex`   📦 `ping-me-in-discord`   💬 `notification-skill-design`**`` then ``> **🏠 `mem@vultr`   📅 `8/14 12:00:11`   🧵 `019fb637`**`` and then `build complete`, with exactly one newline between each line

#### Scenario: New starter template has no Codex thread
- **WHEN** direct CLI runtime metadata identifies agent `CLI`, a project, session name `interactive`, and no session ID
- **THEN** the first metadata line remains complete and the second line omits only the thread emoji and session ID

#### Scenario: Caller Markdown remains intact
- **WHEN** the caller's message contains Discord Markdown
- **THEN** the starter template adds both metadata lines without escaping or otherwise changing the caller's message content

#### Scenario: Existing templates survive an upgrade
- **WHEN** a user upgrades the binary without explicitly forcing initialization
- **THEN** the user's existing `defaults.md` remains unchanged
