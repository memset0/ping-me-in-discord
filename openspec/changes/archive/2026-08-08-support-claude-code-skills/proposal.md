## Why

The bundled notification skills can currently be installed only for Codex, so Claude Code users cannot discover or run the same supported workflows through the CLI. Supporting both agents from one canonical source avoids divergent instructions while preserving predictable project and user-global installation.

## What Changes

- Add `--agent <codex|claude-code>` to `skills install`, defaulting to `codex` for compatibility with existing commands.
- Copy the shared `ping-me-send-message` and `ping-me-report-agent-status` assets into the selected agent's project or global skills directory; never install them through symbolic links.
- Resolve Claude Code project installs to `.claude/skills` and global installs to `${CLAUDE_CONFIG_DIR}/skills` when configured, otherwise `~/.claude/skills`.
- Continue resolving Codex through `.codex/skills`, `CODEX_HOME`, and the existing home-directory fallback.
- Keep each skill's instructions and runner in one canonical repository location, selecting only agent-specific auxiliary metadata when producing installed copies.
- Make the shared skill workflow accept either a Codex thread ID or Claude Code session ID while retaining the existing Discord template behavior and bounded error reporting.
- Document copyable Codex and Claude Code project/global installation commands, paths, invocation syntax, and refresh behavior in the English README.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-skill-installation`: Select Codex or Claude Code, resolve its project/global directory, and install regular-file copies from one canonical bundled source.
- `agent-notification-skill`: Run the same two skills under Codex or Claude Code using the active agent conversation identifier.

## Impact

- Affected CLI surface: `pingme skills install` and its installation summary.
- Affected implementation: embedded skill manifest, destination resolution, atomic file refresh, and runner session-ID normalization.
- Affected assets: the existing canonical `SKILL.md`, runner scripts, and Codex-only UI metadata; no duplicate Claude source tree is added.
- Affected documentation and tests: installation guide, destination/regular-file assertions, standalone-binary coverage, and cross-agent runner behavior.
