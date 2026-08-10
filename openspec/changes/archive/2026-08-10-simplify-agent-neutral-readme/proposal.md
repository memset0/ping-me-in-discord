## Why

The README repeats installation, initialization, and skill guidance across several long sections, making first-time setup harder to follow. Its agent documentation also overemphasizes individual frameworks even though the canonical skills are framework-neutral.

## What Changes

- Replace the separate installation, initialization, and skill-installation introductions with one concise setup flow: install the binaries, initialize and edit `config.toml`, then optionally install the agent skills.
- Introduce both equivalent executable entry points once, then use only the shorter `pingme` spelling in command examples.
- Describe the notification skills as agent-agnostic and mention Codex and Claude Code only where their currently supported installation targets and discovery conventions differ.
- Remove duplicated, obsolete, and overly detailed README explanations while retaining links to the complete configuration and release references.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `user-installation-and-releases`: Require a concise, ordered README setup path and consistent use of the short executable after the entry-point introduction.
- `agent-skill-installation`: Require framework-neutral skill framing while documenting the currently supported installation targets accurately and compactly.

## Impact

- Rewrites `README.md` in English without changing binaries, configuration formats, installation paths, or runtime behavior.
- Updates the two affected documentation contracts and archives this OpenSpec change after validation.
