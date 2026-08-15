## MODIFIED Requirements

### Requirement: Templates receive automatic runtime metadata
Every template render SHALL expose a reserved `runtime` object. It SHALL contain non-empty `user` and `hostname` strings; nested non-empty `agent.name`, `project.name`, and `session.name` strings; an optional `session.title`; an optional `session.id`; a `timestamp` object whose `local`, `unix`, and `iso8601` fields represent one captured invocation instant; and an optional legacy `codex_thread_id` alias equal to `session.id`. An explicit non-empty `PINGME_SESSION_NAME` SHALL populate both `session.title` and `session.name`; explicit non-empty `PINGME_AGENT_NAME`, `PINGME_PROJECT_NAME`, and `PINGME_SESSION_ID` values SHALL take precedence for their matching fields. Without an explicit session name, `session.title` SHALL remain unset while the backward-compatible `session.name` SHALL be derived as `session-<first eight session-ID characters>` or `interactive` when no ID exists. Without other explicit values, the CLI SHALL infer the supported agent from its session environment or identify direct use as `CLI`, and infer the project from the current directory with an `unknown-project` fallback. Session ID discovery SHALL prefer `PINGME_SESSION_ID`, then `CLAUDE_CODE_SESSION_ID`, then `CODEX_THREAD_ID`. All string values SHALL be normalized for one-line Discord inline code. `timestamp.local` SHALL use the system local time in `M/D HH:mm:ss` form, `timestamp.unix` SHALL be whole Unix seconds, and `timestamp.iso8601` SHALL be a UTC ISO 8601 string.

#### Scenario: Runtime values are available without caller data
- **WHEN** a template references the identity, agent, project, session, and timestamp runtime fields without matching `--data` or `--var` input
- **THEN** the CLI renders every required field from automatically discovered runtime metadata and leaves optional fields unset when their source is unavailable

#### Scenario: Explicit agent context is available
- **WHEN** the four non-empty `PINGME_*` context variables identify `Codex`, `ping-me-in-discord`, `notification-skill-design`, and session `019fb637`
- **THEN** the corresponding nested runtime fields contain those normalized values, `runtime.session.title` and `runtime.session.name` both equal `notification-skill-design`, and `runtime.codex_thread_id` also equals `019fb637`

#### Scenario: Session title is not invented
- **WHEN** a session ID is available but `PINGME_SESSION_NAME` is unset or empty
- **THEN** `runtime.session.title` is unset while `runtime.session.name` retains its backward-compatible `session-<ID prefix>` value

#### Scenario: Codex thread ID is available
- **WHEN** no generic session ID is set and `CODEX_THREAD_ID` contains a non-empty current conversation identifier
- **THEN** `runtime.session.id` and `runtime.codex_thread_id` contain that normalized identifier without caller data

#### Scenario: Codex thread ID is unavailable
- **WHEN** every supported session-ID environment variable is unset or contains only whitespace
- **THEN** `runtime.session.id`, `runtime.session.title`, and `runtime.codex_thread_id` are unset, `runtime.session.name` is `interactive`, and other runtime metadata remains available

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
Newly initialized `templates/defaults.md` SHALL render exactly one bold blockquote metadata line followed immediately by the caller's Markdown content on the next line. The line SHALL render available segments in this order: house emoji and inline-code `user@hostname`, package emoji and inline-code project name, thread emoji and inline-code session title with the full session ID as its fallback, robot emoji and inline-code agent name, and calendar emoji and inline-code local timestamp. The timestamp SHALL always render last. The host segment SHALL be omitted when the hostname is unavailable and SHALL omit only the `user@` prefix when the user is unavailable. Project and agent segments SHALL be omitted for their `unknown-project` and direct `CLI` fallbacks. The session segment SHALL be omitted when neither a title nor an ID is available. Exactly three spaces SHALL separate rendered segments, and the template SHALL insert no blank line before the content.

#### Scenario: New starter template includes a Codex thread
- **WHEN** runtime metadata is agent `Codex`, project `ping-me-in-discord`, session title `notification-skill-design`, user `mem`, host `vultr`, timestamp `8/14 12:00:11`, and session ID `019fb637`, and the caller sends `build complete`
- **THEN** content is ``> **🏠 `mem@vultr`   📦 `ping-me-in-discord`   🧵 `notification-skill-design`   🤖 `Codex`   📅 `8/14 12:00:11`**`` followed by `build complete` with exactly one newline between the two lines

#### Scenario: Session ID is the title fallback
- **WHEN** runtime metadata has session ID `019fb637` but no explicit session title
- **THEN** the thread segment renders ``🧵 `019fb637` `` and does not render the derived compatibility name

#### Scenario: New starter template has no Codex thread
- **WHEN** direct CLI runtime metadata has a usable host and project but no session title or session ID and identifies the agent as `CLI`
- **THEN** the metadata line renders host, project, and timestamp while omitting the session and agent segments

#### Scenario: Unavailable optional context is omitted
- **WHEN** runtime metadata has no usable hostname, project, session title, session ID, or coding-agent identity
- **THEN** the metadata line contains only the calendar emoji and local timestamp without empty labels or extra separators

#### Scenario: Caller Markdown remains intact
- **WHEN** the caller's message contains Discord Markdown
- **THEN** the starter template adds the metadata line without escaping or otherwise changing the caller's message content

#### Scenario: Existing templates survive an upgrade
- **WHEN** a user upgrades the binary without explicitly forcing initialization
- **THEN** the user's existing `defaults.md` remains unchanged
