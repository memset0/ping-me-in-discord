## Context

The runtime object currently exposes a required `session.name` that combines an explicit human-readable name with generated compatibility fallbacks. The starter template therefore cannot distinguish a real conversation title from `session-<prefix>`. The three bundled skill runners also generate and export that fallback before invoking the CLI. Existing custom templates may rely on the required `runtime.session.name`, and initialized user templates must not be overwritten during upgrades.

## Goals / Non-Goals

**Goals:**

- Give templates a reliable optional session-title field without breaking `runtime.session.name`.
- Make the starter header compact, conditional, and deterministic.
- Preserve full session-ID visibility when no title is available.
- Keep all bundled runner copies behaviorally identical.

**Non-Goals:**

- Migrating existing user-owned template files automatically.
- Adding new CLI flags or configuration keys for runtime context.
- Changing Discord thread routing through `--thread-id` or `--thread-name`.

## Decisions

### Add `runtime.session.title` instead of changing `runtime.session.name`

The runtime serializer will normalize the explicit `PINGME_SESSION_NAME` once, store it as optional `session.title`, and use the same value for `session.name` when present. When absent, only `session.name` receives the existing generated fallback. This preserves custom-template compatibility while giving the starter template an unambiguous title signal. Reinterpreting or making `session.name` optional was rejected because it would silently change existing templates.

### Preserve absence through the skill runner

The runner will export `PINGME_SESSION_NAME` only when inherited or supplied through `--session-name`; it will no longer derive `session-<prefix>`. Agent skills already require a stable explicit name for normal notifications, while manual or degraded runner calls can now exercise the intended full-ID fallback. Deriving a separate marker variable was rejected as unnecessary protocol complexity.

### Validate the rendered session label rather than always requiring the ID text

The skills will continue to require a real agent session ID before any normal delivery, but their dry-run check will look for the established session title when present and the exact ID otherwise. This keeps bounded failure behavior intact without rejecting the new title-first default template. Rendering both title and ID was rejected because it would defeat the requested fallback relationship and make the compact header wider.

### Use one conditional Jinja metadata line

The starter template will emit segments directly in final display order and give each optional segment its own condition. Included segments contribute their trailing three-space separator, while the always-present timestamp closes the line. Sentinel fallback values (`unknown-host`, `unknown-project`, and `CLI`) identify context that should not be shown. This avoids post-render whitespace cleanup and keeps the template user-editable.

### Treat the installed test template as user-owned state

Repository initialization remains non-destructive. For the requested live demonstration, the local template will be updated explicitly outside Git after repository tests pass; this is an operator action, not an installer migration.

## Risks / Trade-offs

- [A long title or ID can make the single metadata line visually wide] → Keep values in inline code and let Discord wrap naturally; do not truncate identifiers.
- [Sentinel string comparisons are visible in the starter template] → Keep the compatibility runtime fields stable and document the conditions so users can customize them.
- [Runner copies can drift] → Update all copies together and retain the byte-equality regression test.
- [A custom template can omit both title and ID] → Keep the skill dry-run check and invoke one bounded failure report when the expected label is absent.

## Migration Plan

Ship the new starter template and runtime field for fresh or forced initialization. Existing installations remain unchanged unless the user explicitly replaces their local template. Rollback consists of restoring the previous starter template; custom templates using `runtime.session.name` remain valid throughout.
