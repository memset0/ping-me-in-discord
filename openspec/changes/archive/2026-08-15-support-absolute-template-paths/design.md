## Context

Template selection currently flows through one string field: an explicit `--template` overrides `[defaults].template`, configuration validation accepts only a restricted template name, and rendering appends `.md` inside the configured template directory. The new path form crosses configuration validation, rendering, offline validation, and CLI documentation, and it must not weaken the existing traversal protection for relative selectors.

## Goals / Non-Goals

**Goals:**

- Give explicit absolute `.md` paths a direct, predictable meaning on every supported platform.
- Keep named-template behavior and its path-traversal boundary unchanged.
- Apply the same selector grammar to CLI and configured default values.
- Ensure offline validation checks an absolute default template's syntax and frontmatter.

**Non-Goals:**

- Supporting relative paths such as `./custom.md` or `../custom.md`.
- Discovering or listing templates outside the configured template directory.
- Copying, canonicalizing, or managing externally selected template files.

## Decisions

### Treat selectors as either safe names or exact absolute Markdown paths

The shared selector validator will accept the existing ASCII name grammar or a platform-absolute path whose final extension is exactly `.md`. Rendering will append `.md` only for a safe name; an absolute selector is used as-is. This avoids ambiguous behavior such as appending an extension to an already complete path.

Allowing arbitrary relative paths was considered, but it would reintroduce traversal and make resolution depend on the process working directory. Requiring all external selections to be absolute keeps caller intent explicit.

### Keep resolution separate from file existence checks

Selector validation establishes the accepted shape without accessing the filesystem. Send reads the resolved file and retains its existing contextual errors. `config validate` additionally requires and compiles the selected default file, while named-template listing and validation remain scoped to the configured directory.

Canonicalizing paths was considered, but it would make missing-file diagnostics less direct and introduce different behavior for symlinks. An explicitly selected absolute path, including a symlink, follows normal operating-system file semantics.

### Preserve the selected value in local dry-run metadata

`RenderedMessage.template` will continue to contain the selector supplied by the caller. For an absolute path this means dry-run JSON can show the local path, but the field remains local diagnostic metadata and is not included in the Discord webhook payload.

## Risks / Trade-offs

- **[An absolute selector can read outside the configuration tree]** → Require an explicit absolute `.md` path and keep all implicit/named resolution confined to the configured directory.
- **[Dry-run output can reveal a local path]** → Document absolute selection as explicit caller authority; Discord receives only the rendered payload, not the template selector.
- **[External files are not shown by `templates list`]** → Keep listing deterministic and configuration-scoped; callers already know the absolute file they selected.

## Migration Plan

No configuration migration is required. Existing safe names retain identical behavior. Rollback restores rejection of absolute selectors without changing stored templates or configuration files.
