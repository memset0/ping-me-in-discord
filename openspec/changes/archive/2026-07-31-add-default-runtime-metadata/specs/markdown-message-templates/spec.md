## ADDED Requirements

### Requirement: Templates receive automatic runtime metadata
Every template render SHALL expose a reserved `runtime` object. It SHALL contain non-empty `user` and `hostname` strings plus a `timestamp` object whose `local`, `unix`, and `iso8601` fields represent one captured invocation instant. `timestamp.local` SHALL use the system local time in `M/D HH:mm:ss` form, `timestamp.unix` SHALL be whole Unix seconds, and `timestamp.iso8601` SHALL be a UTC ISO 8601 string.

#### Scenario: Runtime values are available without caller data
- **WHEN** a template references `runtime.user`, `runtime.hostname`, and the three `runtime.timestamp` fields without matching `--data` or `--var` input
- **THEN** the CLI renders all fields from automatically discovered runtime metadata

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
Newly initialized `templates/defaults.md` SHALL render exactly one bold blockquote metadata line followed immediately by the caller's Markdown content on the next line. The metadata line SHALL contain the house emoji, inline-code `user@hostname`, three spaces, the calendar emoji, and the inline-code local timestamp. It SHALL not insert a blank line between metadata and content.

#### Scenario: New starter template is rendered
- **WHEN** runtime metadata is `mem`, `vultr`, and `7/31 12:00:11`, and the caller sends `build complete` through a newly initialized default template
- **THEN** content is ``> **🏠 `mem@vultr`   📅 `7/31 12:00:11`**`` followed immediately by a newline and `build complete`

#### Scenario: Caller Markdown remains intact
- **WHEN** the caller's message contains Discord Markdown
- **THEN** the starter template adds the metadata line without escaping or otherwise changing the caller's message content

#### Scenario: Existing templates survive an upgrade
- **WHEN** installation or non-forced initialization encounters an existing `templates/defaults.md`
- **THEN** it preserves that file instead of replacing it with the new starter
