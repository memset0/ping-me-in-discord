## Context

`src/config.rs` owns the starter template written by `init`, while `examples/templates/defaults.md` is kept byte-for-byte equivalent for users and documentation. The template renderer already preserves Discord Markdown and supplies all required runtime fields. Existing initialization behavior refuses to overwrite user-owned files unless forced.

## Goals / Non-Goals

**Goals:**

- Make caller content the primary visual element and runtime context a Discord subtext footer.
- Keep the starter constant, bundled example, documentation, and exact-output tests aligned.
- Preserve all current runtime-field ordering, fallback, omission, and spacing behavior.

**Non-Goals:**

- Changing runtime metadata discovery or adding template variables.
- Rewriting existing user templates during installation or upgrades.
- Trimming or otherwise normalizing caller-provided Markdown content.

## Decisions

### Put the content placeholder before one literal subtext line

The starter source will use `{{ message }}` followed by one newline and a metadata line whose first characters are `-# `. Discord recognizes subtext only when that marker starts the line, so the marker remains unconditional even when every optional metadata segment is omitted. An alternative embed footer would require a structured payload and would change the intentionally simple Markdown-only default.

### Render footer values as plain subtext

The footer will remove the old blockquote, bold, and inline-code markers while retaining emojis and exactly three spaces between present segments. Layering inline code or emphasis onto Discord subtext would compete with the subdued hierarchy the user selected and would make the footer visually busier.

### Preserve caller whitespace

The template will contribute exactly one structural newline between `{{ message }}` and the footer, but it will not trim the value of `message`. If caller content deliberately ends with additional newlines, those remain part of that content. This avoids corrupting Markdown constructs or fenced code blocks.

### Lock the layout with rendered-output tests

Tests will cover full context, session-ID fallback, omitted optional context, caller Markdown preservation, and content length accounting. Exact template-source assertions will keep the initializer and bundled example synchronized.

## Risks / Trade-offs

- **Discord changes or removes `-#` rendering semantics** → The output remains readable plain Markdown text, and users can customize their local template independently.
- **Caller content ends in blank lines** → Preserve those intentional bytes rather than silently trimming user content; the starter itself adds only one newline.
- **Existing users do not see the new layout automatically** → Preserve their configuration by design; document that they may copy the new template or run forced initialization when appropriate.

## Migration Plan

Ship the new starter in the binary and bundled example. Fresh initialization and explicitly forced initialization receive the new layout, while ordinary upgrades leave existing `templates/defaults.md` untouched. Rollback consists of restoring the previous starter source; it requires no configuration or data migration.
