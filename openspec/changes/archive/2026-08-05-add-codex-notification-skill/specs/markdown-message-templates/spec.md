## MODIFIED Requirements

### Requirement: Templates receive automatic runtime metadata
Every template render SHALL expose a reserved `runtime` object. It SHALL contain non-empty `user` and `hostname` strings, an optional `codex_thread_id`, plus a `timestamp` object whose `local`, `unix`, and `iso8601` fields represent one captured invocation instant. `codex_thread_id` SHALL contain the normalized non-empty value of `CODEX_THREAD_ID` when that environment variable is available and SHALL otherwise be unset. `timestamp.local` SHALL use the system local time in `M/D HH:mm:ss` form, `timestamp.unix` SHALL be whole Unix seconds, and `timestamp.iso8601` SHALL be a UTC ISO 8601 string.

#### Scenario: Runtime values are available without caller data
- **WHEN** a template references `runtime.user`, `runtime.hostname`, and the three `runtime.timestamp` fields without matching `--data` or `--var` input
- **THEN** the CLI renders all fields from automatically discovered runtime metadata

#### Scenario: Codex thread ID is available
- **WHEN** `CODEX_THREAD_ID` contains a non-empty current conversation identifier
- **THEN** `runtime.codex_thread_id` contains that normalized identifier without caller data

#### Scenario: Codex thread ID is unavailable
- **WHEN** `CODEX_THREAD_ID` is unset or contains only whitespace
- **THEN** `runtime.codex_thread_id` is unset and other runtime metadata remains available

#### Scenario: Timestamp fields share one instant
- **WHEN** a template renders local, Unix, and ISO 8601 timestamp representations
- **THEN** all three values describe the same instant captured once for that send

#### Scenario: Identity discovery fails
- **WHEN** the operating system cannot provide the current user or hostname
- **THEN** rendering continues with `unknown-user` or `unknown-host` for the unavailable field

#### Scenario: Caller attempts to replace runtime metadata
- **WHEN** JSON data or a variable override defines the reserved top-level key `runtime`
- **THEN** the CLI rejects the collision before rendering or making a network request

### Requirement: The starter template identifies the emitter before content
Newly initialized `templates/defaults.md` SHALL render exactly one bold blockquote metadata line followed immediately by the caller's Markdown content on the next line. The metadata line SHALL contain the house emoji, inline-code `user@hostname`, three spaces, the calendar emoji, and the inline-code local timestamp. When `runtime.codex_thread_id` is present, it SHALL append three spaces, the thread emoji, and the inline-code Codex thread ID before closing the bold span. It SHALL not insert a blank line between metadata and content, and when the Codex thread ID is absent it SHALL preserve the original two-field layout.

#### Scenario: New starter template includes a Codex thread
- **WHEN** runtime metadata is `mem`, `vultr`, `8/3 12:00:11`, and thread ID `019fb637`, and the caller sends `build complete`
- **THEN** content is ``> **🏠 `mem@vultr`   📅 `8/3 12:00:11`   🧵 `019fb637`**`` followed immediately by a newline and `build complete`

#### Scenario: New starter template has no Codex thread
- **WHEN** runtime metadata is `mem`, `vultr`, and `7/31 12:00:11` with no Codex thread ID
- **THEN** content is ``> **🏠 `mem@vultr`   📅 `7/31 12:00:11`**`` followed immediately by the caller content on the next line

#### Scenario: Caller Markdown remains intact
- **WHEN** the caller's message contains Discord Markdown
- **THEN** the starter template adds the metadata line without escaping or otherwise changing the caller's message content

#### Scenario: Existing templates survive an upgrade
- **WHEN** a user upgrades the binary without explicitly forcing initialization
- **THEN** the user's existing `defaults.md` remains unchanged
