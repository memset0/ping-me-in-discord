## Why

The default Discord header currently gives a generated session name and the full session ID separate visual roles, so the thread icon often shows an ID instead of the human-readable conversation title users expect. The header also needs a compact, predictable order and must omit unavailable context rather than displaying synthetic placeholders.

## What Changes

- Expose an optional `runtime.session.title` derived only from an explicitly supplied session name, while retaining `runtime.session.name` as a backward-compatible non-empty field.
- Replace the two-line starter header with one bold blockquote line ordered as host, project, session title (falling back to full session ID), agent, and local timestamp.
- Omit unavailable host, project, session, and agent segments independently; always keep the timestamp last and place message content on the immediately following line.
- Stop skill runners from inventing and exporting a session name when the agent did not establish one, allowing the default template to use the session-ID fallback.
- Make bundled skill preflights accept the explicit session title as the rendered session label, while still requiring and validating the full session ID when no title exists.
- Update examples, documentation, and automated coverage for the new runtime field and exact rendered layout.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-message-templates`: Add optional session-title metadata and redefine the starter template's conditional, ordered one-line context header.
- `agent-notification-skill`: Preserve absent session titles in the runner so the template can fall back to the agent session ID.

## Impact

This affects runtime metadata serialization, the bundled starter template, skill runner context export, skill preflight instructions, template and runner tests, and user-facing template documentation. Existing initialized user templates remain untouched, and existing custom references to `runtime.session.name` remain compatible.
