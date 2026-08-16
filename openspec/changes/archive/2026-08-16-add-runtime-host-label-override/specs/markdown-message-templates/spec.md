## ADDED Requirements

### Requirement: Runtime host labels can be overridden per send
Both the shorthand invocation and the explicit `send` subcommand SHALL accept `--host <LABEL>` as a complete replacement for the automatically discovered runtime host label. The supplied value SHALL represent the entire label, including any optional user portion, and SHALL be normalized for one-line Discord text. An explicitly blank or whitespace-only label SHALL be rejected before rendering or network access. The host override SHALL be independent of the Discord webhook display identity selected by `--username`, and an ordinary top-level template variable named `host` SHALL NOT replace the reserved `runtime.host` field.

#### Scenario: Shorthand replaces the complete host label
- **WHEN** the user invokes `pingme --host "mukai-h20" "message"`
- **THEN** `runtime.host` is `mukai-h20` without an automatically added user prefix

#### Scenario: Explicit send preserves a supplied user prefix
- **WHEN** the user invokes `pingme send --host "root@mukai-h20" "message"`
- **THEN** `runtime.host` is `root@mukai-h20`

#### Scenario: Blank host override is rejected
- **WHEN** the user explicitly supplies a host label containing only whitespace
- **THEN** the CLI reports an invalid host label before rendering or making a network request

#### Scenario: Host and webhook username remain distinct
- **WHEN** the user supplies `--host "worker-1"` and `--username "Deploy Bot"`
- **THEN** the template receives `worker-1` as `runtime.host` while the Discord webhook payload uses `Deploy Bot` as its display username

#### Scenario: Ordinary host variable does not replace runtime metadata
- **WHEN** the user supplies `--var host=worker-1` without `--host`
- **THEN** the template receives `worker-1` as the ordinary `host` variable and retains its automatically discovered `runtime.host`

## MODIFIED Requirements

### Requirement: Common send options are command-line arguments
Both the shorthand invocation and `send` subcommand SHALL accept `--channel`, `--username`, `--host`, `--avatar`, `--avatar-url`, `--avatar-file`, `--avatar-emoji`, `--avatar-text`, `--avatar-icon`, `--avatar-background`, `--avatar-foreground`, `--avatar-font`, `--avatar-size`, `--avatar-font-size`, `--avatar-scale`, `--thread-id`, `--thread-name`, `--tts`, and `--no-tts`. Avatar source arguments SHALL be mutually exclusive, and conflicting options SHALL fail during argument validation.

#### Scenario: Override identity from the shorthand
- **WHEN** the user invokes `pingme --username "Deploy Bot" --avatar release "done"`
- **THEN** the final payload uses `Deploy Bot` and the configured `release` avatar profile

#### Scenario: Override runtime host from the shorthand
- **WHEN** the user invokes `pingme --host "mukai-h20" "done"`
- **THEN** the rendered runtime host label is `mukai-h20`

#### Scenario: Explicitly disable template TTS
- **WHEN** frontmatter sets `tts: true` and the user supplies `--no-tts`
- **THEN** the outgoing payload sets `tts` to false

### Requirement: Templates receive automatic runtime metadata
Every template render SHALL expose a reserved `runtime` object. It SHALL contain non-empty `user` and `hostname` strings; an optional non-empty `host` string representing the complete display label; nested non-empty `agent.name`, `project.name`, and `session.name` strings; an optional `session.title`; an optional `session.id`; a `timestamp` object whose `local`, `unix`, and `iso8601` fields represent one captured invocation instant; and an optional legacy `codex_thread_id` alias equal to `session.id`. Without an explicit host override, `runtime.host` SHALL be generated as `user@hostname` when both identity values are available, as `hostname` when only the hostname is available, and SHALL be unset when the hostname is unavailable. An explicit non-empty `--host` SHALL replace that complete generated label without changing the compatibility fields `runtime.user` or `runtime.hostname`. An explicit non-empty `PINGME_SESSION_NAME` SHALL populate both `session.title` and `session.name`; explicit non-empty `PINGME_AGENT_NAME`, `PINGME_PROJECT_NAME`, and `PINGME_SESSION_ID` values SHALL take precedence for their matching fields. Without an explicit session name, `session.title` SHALL remain unset while the backward-compatible `session.name` SHALL be derived as `session-<first eight session-ID characters>` or `interactive` when no ID exists. Without other explicit values, the CLI SHALL infer the supported agent from its session environment or identify direct use as `CLI`, and infer the project from the current directory with an `unknown-project` fallback. Session ID discovery SHALL prefer `PINGME_SESSION_ID`, then `CLAUDE_CODE_SESSION_ID`, then `CODEX_THREAD_ID`. All string values SHALL be normalized for one-line Discord text. `timestamp.local` SHALL use the system local time in `M/D HH:mm:ss` form, `timestamp.unix` SHALL be whole Unix seconds, and `timestamp.iso8601` SHALL be a UTC ISO 8601 string.

#### Scenario: Runtime values are available without caller data
- **WHEN** a template references the identity, host, agent, project, session, and timestamp runtime fields without matching `--data` or `--var` input
- **THEN** the CLI renders every required field from automatically discovered runtime metadata, derives the optional host label when the hostname is available, and leaves other optional fields unset when their source is unavailable

#### Scenario: Automatic host includes the available user
- **WHEN** runtime discovery produces user `mem` and hostname `vultr` without an explicit host override
- **THEN** `runtime.host` is `mem@vultr`

#### Scenario: Automatic host omits an unavailable user
- **WHEN** runtime discovery produces `unknown-user` and hostname `vultr` without an explicit host override
- **THEN** `runtime.host` is `vultr`

#### Scenario: Explicit host replaces only the complete display label
- **WHEN** runtime discovery produces user `root` and hostname `vultr` and the caller supplies `--host "mukai-h20"`
- **THEN** `runtime.host` is `mukai-h20` while `runtime.user` remains `root` and `runtime.hostname` remains `vultr`

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
- **THEN** rendering continues with `unknown-user`, `unknown-host`, or `unknown-project` for the unavailable compatibility field and leaves `runtime.host` unset when the hostname is unavailable

#### Scenario: Caller attempts to replace runtime metadata
- **WHEN** JSON data or a variable override defines the reserved top-level key `runtime`
- **THEN** the CLI rejects the collision before rendering or making a network request

### Requirement: The starter template appends subdued emitter context
Newly initialized `templates/defaults.md` SHALL render the caller's Markdown content first, followed immediately on the next line by exactly one Discord subtext metadata line beginning with `-# `. The metadata line SHALL use plain text without bold, inline-code, or blockquote styling. It SHALL render available segments in this order: house emoji and the complete `runtime.host` label, package emoji and project name, thread emoji and session title with the full session ID as its fallback, robot emoji and agent name, and calendar emoji and local timestamp. The timestamp SHALL always render last. The host segment SHALL use the complete explicit `--host` value when supplied, otherwise use the automatically generated runtime host label, and SHALL be omitted when `runtime.host` is unavailable. Project and agent segments SHALL be omitted for their `unknown-project` and direct `CLI` fallbacks. The session segment SHALL be omitted when neither a title nor an ID is available. Exactly three spaces SHALL separate rendered segments, and the starter template SHALL insert exactly one newline and no blank line between its message placeholder and the metadata footer.

#### Scenario: New starter template includes a Codex thread
- **WHEN** runtime metadata is agent `Codex`, project `ping-me-in-discord`, session title `notification-skill-design`, runtime host `mem@vultr`, timestamp `8/14 12:00:11`, and session ID `019fb637`, and the caller sends `build complete`
- **THEN** content is `build complete` followed by ``-# 🏠 mem@vultr   📦 ping-me-in-discord   🧵 notification-skill-design   🤖 Codex   📅 8/14 12:00:11`` with exactly one newline between the two lines

#### Scenario: Explicit host replaces the generated starter segment
- **WHEN** the automatically generated runtime host is `root@vultr` and the caller supplies `--host "mukai-h20"`
- **THEN** the starter metadata renders `🏠 mukai-h20` and does not render `root@vultr`

#### Scenario: Session ID is the title fallback
- **WHEN** runtime metadata has session ID `019fb637` but no explicit session title
- **THEN** the thread segment renders `🧵 019fb637` and does not render the derived compatibility name

#### Scenario: New starter template has no Codex thread
- **WHEN** direct CLI runtime metadata has a usable runtime host and project but no session title or session ID and identifies the agent as `CLI`
- **THEN** the trailing subtext line renders host, project, and timestamp while omitting the session and agent segments

#### Scenario: Unavailable optional context is omitted
- **WHEN** runtime metadata has no usable host, project, session title, session ID, or coding-agent identity
- **THEN** the trailing subtext line contains only `-# `, the calendar emoji, and the local timestamp without empty labels or extra separators

#### Scenario: Caller Markdown remains intact
- **WHEN** the caller's message contains Discord Markdown
- **THEN** the starter template renders that content before the metadata footer without escaping or otherwise changing the caller's message content

#### Scenario: Existing templates survive an upgrade
- **WHEN** a user upgrades the binary without explicitly forcing initialization
- **THEN** the user's existing `defaults.md` remains unchanged
