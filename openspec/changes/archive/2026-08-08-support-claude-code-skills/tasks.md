## 1. Add agent-aware installation

- [x] 1.1 Add the compatible `--agent <codex|claude-code>` selector and resolve Codex and Claude Code project/global destinations, including their configuration-directory overrides.
- [x] 1.2 Filter one embedded canonical asset manifest per selected agent, copy only applicable files, and report the selected agent and destination.

## 2. Enforce safe copied assets

- [x] 2.1 Replace exact owned symbolic-link paths with atomic regular-file copies while preserving byte-identical regular files and unrelated content.
- [x] 2.2 Keep legacy cleanup scoped to Codex and add isolated tests for regular files, symlink replacement, repeatability, permissions, and both agents' project/global paths.

## 3. Make the canonical skills cross-agent

- [x] 3.1 Update the single canonical `SKILL.md` sources and identical runners to preflight Codex or Claude Code session identity without adding a `.claude` source copy.
- [x] 3.2 Extend runner and installed-asset tests for Claude session priority, existing Codex behavior, bounded failure reporting, and standalone Claude installation.

## 4. Document and validate

- [x] 4.1 Update the English README and affected main-spec purposes with Codex and Claude Code commands, paths, copy semantics, invocation names, and discovery guidance.
- [x] 4.2 Run formatting, clippy, all tests, skill validation, repository scans, and strict OpenSpec validation with an ephemeral non-incremental Cargo target, then remove that target and verify the project `target` remains absent.
