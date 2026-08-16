## Why

The runtime host shown by the default notification template is always derived from the local user and hostname, so callers such as Fleet cannot replace it with the node label that users recognize. The avatar test suite also depends on a CJK system font that is not present on standard GitHub Actions runners, causing otherwise unrelated builds to fail.

## What Changes

- Add a `--host <LABEL>` send option to both shorthand and explicit `send` invocations. When supplied, it replaces the complete displayed runtime host label, including any optional user portion.
- Expose a reserved `runtime.host` template field. Without `--host`, it is generated from the current `user@hostname`; with `--host`, it uses the caller-provided label exactly after safe normalization.
- Reject explicitly blank host labels instead of silently reverting to machine-derived metadata.
- Update the starter/default template to render `runtime.host`, while retaining `runtime.user` and `runtime.hostname` for compatible custom templates.
- Replace the unit test's CJK font dependency with an English glyph so the test remains portable across standard CI images without changing production Unicode avatar support.
- Document and test the distinction between runtime `--host` and Discord webhook `--username`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-message-templates`: Add a complete runtime host-label override and make the starter template consume the resulting `runtime.host` field.

## Impact

This change affects CLI argument parsing, runtime metadata capture, template context construction, the bundled starter templates, CLI and unit tests, and user-facing configuration documentation. It does not change Discord webhook identity handling, configuration-file precedence, or production Unicode avatar rendering.
