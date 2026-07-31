## Why

Default notifications currently contain only the caller-provided message, so recipients cannot see which machine emitted the notification or when the CLI ran. Templates also lack automatic runtime values, forcing every caller to assemble and pass this common metadata manually.

## What Changes

- Expose a reserved `runtime` template object containing the current system user, hostname, and consistent local, Unix, and ISO 8601 timestamps captured once per send.
- Change the starter `templates/defaults.md` to place a bold blockquote metadata line before the caller-provided Markdown content, without an intervening blank line.
- Keep runtime discovery failure-tolerant so notifications still render with explicit fallback labels.
- Preserve existing user templates during installation and initialization, while documenting how users can adopt or customize the new default.
- Add deterministic tests for runtime context, override protection, exact starter rendering, and Markdown preservation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-message-templates`: Add automatic runtime metadata variables and define the generated default template's observable Markdown layout.

## Impact

- Template context construction and runtime metadata discovery in the Rust library.
- The embedded starter template, example template, documentation, and rendering tests.
- Cross-platform system identity and time-formatting dependencies, subject to the existing Rust 1.87 minimum.
- The executable-adjacent local template will be updated after validation without committing local configuration or credentials.
