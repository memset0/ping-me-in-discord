# markdown-message-templates Specification

## Purpose

Define a human-editable Markdown template format that can produce structured Discord webhook messages from CLI-provided data.

## Requirements

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

### Requirement: A default Markdown template is always addressable
The send command SHALL use `templates/defaults.md` when the user does not select another template. A simple template name SHALL resolve to a `.md` file inside the configured template directory. An absolute template selector ending in `.md` SHALL resolve to that exact file, including when the file is outside the configured template directory. Other relative paths, parent-directory traversal, and absolute paths that do not end in `.md` SHALL be rejected before reading a file.

#### Scenario: Send without a template name
- **WHEN** the user invokes `send` without selecting a template
- **THEN** the CLI renders `defaults.md`

#### Scenario: Named template
- **WHEN** the user selects `deployment`
- **THEN** the CLI renders `deployment.md` from the configured template directory

#### Scenario: Absolute template path from the CLI
- **WHEN** the user supplies `--template /tmp/pingme/custom.md`
- **THEN** the CLI renders that exact file without requiring it to be inside the configured template directory

#### Scenario: Absolute default template path
- **WHEN** `[defaults].template` is an absolute path ending in `.md` and the user does not supply `--template`
- **THEN** the CLI renders that exact file as the default template

#### Scenario: Traversal attempt
- **WHEN** a template selector is a relative path or contains a parent-directory component
- **THEN** the CLI rejects it before reading the file

#### Scenario: Absolute non-Markdown path
- **WHEN** an absolute template selector does not end in `.md`
- **THEN** the CLI rejects it before reading the file

### Requirement: Templates combine frontmatter and Markdown
Each template SHALL support optional YAML frontmatter delimited by `---`, followed by a Markdown body. The rendered Markdown body SHALL become Discord `content`, while supported frontmatter fields SHALL populate the remaining webhook payload and local delivery options.

#### Scenario: Frontmatter and body render together
- **WHEN** a template contains a `username`, an `avatar` reference, an embed, and a Markdown body
- **THEN** the payload contains the rendered username, embed, and body while the avatar reference is handled locally rather than sent as an unknown Discord field

#### Scenario: Body-only template
- **WHEN** a template has no frontmatter and has a non-empty body
- **THEN** the body is sent as Discord message content

### Requirement: Template variables are strict
The CLI SHALL render variables in both frontmatter and the Markdown body. Variables SHALL come from a JSON object and repeatable `--var key=value` arguments, with explicit `--var` values taking precedence. Referencing an undefined variable SHALL fail before any network request.

#### Scenario: Variables render in all sections
- **WHEN** `project=api` is provided and the template uses `{{ project }}` in its username and body
- **THEN** both locations contain `api`

#### Scenario: Missing variable
- **WHEN** a template references `{{ version }}` and no version is provided
- **THEN** rendering fails with an error that names `version` and no Discord request is made

### Requirement: Discord message structures are supported
Frontmatter SHALL support `channel`, `username`, `avatar`, `avatar_url`, `tts`, `embeds`, `allowed_mentions`, `components`, `poll`, `flags`, `thread_id`, and `thread_name`. The CLI SHALL reject an empty payload and SHALL validate Discord's basic content and embed count limits before delivery.

#### Scenario: Structured embed message
- **WHEN** frontmatter defines one or more embeds and the Markdown body is empty
- **THEN** the CLI produces a valid embed-only webhook payload

#### Scenario: Empty message
- **WHEN** a rendered template has no content, embeds, components, poll, or attachment
- **THEN** the CLI rejects the message before delivery

### Requirement: Send options use one consistent precedence
For every supported scalar send option, the CLI SHALL resolve an explicitly supplied command-line argument before template frontmatter and template frontmatter before the corresponding `[defaults]` setting. An absent value SHALL fall through, and a value absent from all layers SHALL remain unset. The initial layered fields SHALL be `channel`, `username`, avatar selection, `thread_id`, `thread_name`, and `tts`.

#### Scenario: CLI overrides template and settings
- **WHEN** CLI, frontmatter, and settings each specify a different `username`
- **THEN** the outgoing payload uses the CLI username

#### Scenario: Frontmatter overrides settings
- **WHEN** CLI omits `channel`, frontmatter selects `releases`, and settings select `alerts`
- **THEN** the `releases` channel selector is resolved and used

#### Scenario: Template normally omits routing
- **WHEN** CLI and frontmatter omit `channel` and `[defaults].channel` selects `alerts`
- **THEN** the configured `alerts` channel is used

### Requirement: Common send options are command-line arguments
Both the shorthand invocation and `send` subcommand SHALL accept `--channel`, `--username`, `--avatar`, `--avatar-url`, `--avatar-file`, `--avatar-emoji`, `--avatar-text`, `--avatar-icon`, `--avatar-background`, `--avatar-foreground`, `--avatar-font`, `--avatar-size`, `--avatar-font-size`, `--avatar-scale`, `--thread-id`, `--thread-name`, `--tts`, and `--no-tts`. Avatar source arguments SHALL be mutually exclusive, and conflicting options SHALL fail during argument validation.

#### Scenario: Override identity from the shorthand
- **WHEN** the user invokes `pingme --username "Deploy Bot" --avatar release "done"`
- **THEN** the final payload uses `Deploy Bot` and the configured `release` avatar profile

#### Scenario: Explicitly disable template TTS
- **WHEN** frontmatter sets `tts: true` and the user supplies `--no-tts`
- **THEN** the outgoing payload sets `tts` to false

### Requirement: Mentions are safe by default
When a template does not define `allowed_mentions`, the generated payload SHALL disable automatic mention parsing.

#### Scenario: Template contains an at-everyone string
- **WHEN** rendered content includes `@everyone` and frontmatter omits `allowed_mentions`
- **THEN** the payload includes an empty allowed-mention parse list

### Requirement: Templates can be inspected without delivery
The CLI SHALL list available templates and SHALL offer a dry-run mode that renders the selected template as redacted, formatted JSON without making network requests.

#### Scenario: Dry run
- **WHEN** the user sends a valid template with `--dry-run`
- **THEN** the CLI prints the final non-secret payload and performs no webhook provisioning, avatar update, or message execution

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

### Requirement: The starter template appends subdued emitter context
Newly initialized `templates/defaults.md` SHALL render the caller's Markdown content first, followed immediately on the next line by exactly one Discord subtext metadata line beginning with `-# `. The metadata line SHALL use plain text without bold, inline-code, or blockquote styling. It SHALL render available segments in this order: house emoji and `user@hostname`, package emoji and project name, thread emoji and session title with the full session ID as its fallback, robot emoji and agent name, and calendar emoji and local timestamp. The timestamp SHALL always render last. The host segment SHALL be omitted when the hostname is unavailable and SHALL omit only the `user@` prefix when the user is unavailable. Project and agent segments SHALL be omitted for their `unknown-project` and direct `CLI` fallbacks. The session segment SHALL be omitted when neither a title nor an ID is available. Exactly three spaces SHALL separate rendered segments, and the starter template SHALL insert exactly one newline and no blank line between its message placeholder and the metadata footer.

#### Scenario: New starter template includes a Codex thread
- **WHEN** runtime metadata is agent `Codex`, project `ping-me-in-discord`, session title `notification-skill-design`, user `mem`, host `vultr`, timestamp `8/14 12:00:11`, and session ID `019fb637`, and the caller sends `build complete`
- **THEN** content is `build complete` followed by ``-# 🏠 mem@vultr   📦 ping-me-in-discord   🧵 notification-skill-design   🤖 Codex   📅 8/14 12:00:11`` with exactly one newline between the two lines

#### Scenario: Session ID is the title fallback
- **WHEN** runtime metadata has session ID `019fb637` but no explicit session title
- **THEN** the thread segment renders `🧵 019fb637` and does not render the derived compatibility name

#### Scenario: New starter template has no Codex thread
- **WHEN** direct CLI runtime metadata has a usable host and project but no session title or session ID and identifies the agent as `CLI`
- **THEN** the trailing subtext line renders host, project, and timestamp while omitting the session and agent segments

#### Scenario: Unavailable optional context is omitted
- **WHEN** runtime metadata has no usable hostname, project, session title, session ID, or coding-agent identity
- **THEN** the trailing subtext line contains only `-# `, the calendar emoji, and the local timestamp without empty labels or extra separators

#### Scenario: Caller Markdown remains intact
- **WHEN** the caller's message contains Discord Markdown
- **THEN** the starter template renders that content before the metadata footer without escaping or otherwise changing the caller's message content

#### Scenario: Existing templates survive an upgrade
- **WHEN** a user upgrades the binary without explicitly forcing initialization
- **THEN** the user's existing `defaults.md` remains unchanged
