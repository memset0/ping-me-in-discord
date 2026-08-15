## Why

The default notification template currently gives runtime metadata the strongest visual treatment and places it before the actual message. Discord's subtext syntax provides a better hierarchy: the caller's content can remain primary while context stays available as a quiet footer.

## What Changes

- Render the caller's Markdown content first in newly initialized default templates.
- Render the runtime metadata immediately afterward as one Discord subtext line using the `-# ` prefix, without bold, inline-code, or blockquote styling.
- Preserve the existing metadata field order, omission rules, session-title fallback, timestamp placement, and exactly three spaces between rendered segments.
- Update the bundled example, documentation, and exact-output tests to make the trailing subtext layout the default for future initialization.
- Preserve existing user-owned `templates/defaults.md` files during upgrades and non-forced initialization.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-message-templates`: Change the newly initialized starter template from a leading bold blockquote metadata header to a trailing Discord subtext metadata footer.

## Impact

- Affects the starter template constant in `src/config.rs`, the bundled `examples/templates/defaults.md`, template rendering tests, and template documentation in `README.md` and `docs/configuration.md`.
- Does not change template variables, configuration precedence, webhook payload fields, or existing user templates.
