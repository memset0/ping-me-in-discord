## Why

The project README currently mixes Chinese prose with English commands and identifiers, which limits accessibility and makes future edits inconsistent. The public entry-point documentation should be fully English, with a durable repository rule that keeps it English.

## What Changes

- Rewrite every prose section and heading in `README.md` in clear English while preserving its commands, configuration examples, links, feature coverage, and security guidance.
- Add a project maintenance instruction requiring `README.md` and all future README edits to be written in English.
- Verify that the rewritten README contains no Chinese text and that its documented commands and local links remain valid.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This change is documentation-only and declares `skip_specs: true`.

## Impact

- Affects `README.md`, the shared `CLAUDE.md`/`AGENTS.md` project instructions, and this change's documentation artifacts.
- Does not change CLI behavior, configuration formats, release assets, or runtime dependencies.
