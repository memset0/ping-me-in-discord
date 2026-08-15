## RENAMED Requirements

- FROM: `### Requirement: The starter template identifies the emitter before content`
- TO: `### Requirement: The starter template appends subdued emitter context`

## MODIFIED Requirements

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
