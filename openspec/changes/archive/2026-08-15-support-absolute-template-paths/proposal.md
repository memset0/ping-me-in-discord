## Why

Users and coding agents sometimes generate a one-off template outside the configured `templates` directory. Requiring every such file to be copied into that directory adds unnecessary state and prevents callers from selecting an already-existing template by its full path.

## What Changes

- Allow `--template` and `[defaults].template` to select an absolute path to an existing `.md` template file.
- Keep simple template names resolving to `<configured-templates-directory>/<name>.md`.
- Continue rejecting relative paths, parent-directory traversal, and absolute paths that do not identify a `.md` file.
- Validate and document absolute template selection, including dry-run coverage for a file outside the configured template directory.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-message-templates`: Expand template selection from safe names only to safe names or explicit absolute Markdown file paths.

## Impact

Template selector validation and resolution in `src/config.rs` and `src/template.rs`, CLI help text, offline configuration validation, integration/unit tests, and template documentation are affected. No dependency or Discord API changes are required.
