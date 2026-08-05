## Why

The repository ships two Codex skills, but users currently have to copy their directories manually and know Codex's project and global search paths. The CLI should install the exact skill version bundled with the binary so setup is reproducible and independent of a source checkout.

## What Changes

- Add `pingme skills install --scope project|global` as an explicit, repeatable installer for the bundled `discord-notify` and `discord-agent-notify` skills.
- Install project-scoped skills below the current project's `.codex/skills/` directory and global skills below `${CODEX_HOME}/skills` or the default `~/.codex/skills` location.
- Embed all required skill files in the release binary, preserve executable permissions for runner scripts, update only the skill directories owned by this application, and report the resolved destination.
- Document both installation forms, update behavior, supported Codex invocation names, and the need to restart or reopen Codex after installation in `README.md`.
- Keep this initial installer Codex-only while leaving the command structure extensible to other agents later.

## Capabilities

### New Capabilities

- `agent-skill-installation`: Installing and refreshing the application's bundled agent skills at project or global scope through the CLI.

### Modified Capabilities

None.

## Impact

- Affects CLI parsing and command dispatch, embedded release assets, filesystem writes and Unix permissions, integration tests, and installation documentation.
- Adds no runtime dependency on Git, GitHub, a package manager, or the source repository.
- Does not install Claude Code skills in this initial version and does not modify Discord configuration or credentials.
