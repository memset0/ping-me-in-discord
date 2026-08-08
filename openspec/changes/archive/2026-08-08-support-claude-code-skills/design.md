## Context

See `proposal.md` for motivation and the two delta specs for observable behavior. The CLI currently embeds six files directly from two repository-local Codex skill directories and resolves only `.codex/skills` destinations. Both skill bodies require `CODEX_THREAD_ID`, while Claude Code supplies `CLAUDE_CODE_SESSION_ID` to Bash subprocesses. Claude Code officially discovers project skills below `.claude/skills` and personal skills below the configured Claude directory, normally `~/.claude/skills`.

OpenSpec keeps workflow templates centrally and writes concrete skill files into each selected tool directory. This change follows that generation model while honoring the stronger project constraint that the repository must not contain a second Claude-specific copy of the notification skill sources.

## Goals / Non-Goals

**Goals:**

- Select Codex or Claude Code explicitly while keeping existing install commands compatible.
- Copy embedded regular files into the selected agent directory with atomic refresh behavior.
- Keep each `SKILL.md` and runner authoritative in exactly one repository location.
- Make the canonical workflow functional in both agents without forking its instructions.

**Non-Goals:**

- Do not add checked-in `.claude/skills/ping-me-*` source copies.
- Do not create or retain installer-created symbolic links.
- Do not install Claude Code slash-command files or package these skills as a Claude plugin.
- Do not automatically change the developer's live global skill directories while implementing or testing this change.
- Do not rename the existing `runtime.codex_thread_id` template key in this change.

## Decisions

### Add an agent selector with a compatible default

Introduce an enum-backed `--agent <codex|claude-code>` option and accept `claude` as an alias. Omission selects Codex so existing scripts retain their behavior. A singular selector keeps each command's destination and summary unambiguous; users who want both agents run the command once per agent, matching the existing explicit-scope safety model.

Making the agent argument required was rejected because it would break all documented and automated Codex installations. A comma-separated multi-agent operation was rejected because partial success would complicate atomicity and ownership reporting across two independent roots.

### Resolve each agent's native project and global paths

Project installs join the current directory with `.codex/skills` or `.claude/skills`. Global Codex installs keep the existing `CODEX_HOME` behavior. Global Claude Code installs honor non-empty `CLAUDE_CONFIG_DIR` before falling back to the user's `~/.claude/skills`, consistent with Claude Code's configuration-directory contract.

### Use one canonical manifest with agent applicability

Retain the existing `.codex/skills/ping-me-*` files as the canonical repository source so the skills remain directly discoverable in this project and existing GitHub links remain valid. Mark `SKILL.md` and each runner as shared assets in the embedded manifest, and mark `agents/openai.yaml` as Codex-only. Claude Code receives regular-file copies of shared assets only.

Moving the source to a neutral third directory was rejected because it would remove direct Codex project discovery and break established review paths without improving installed behavior. Checking in generated Claude copies was rejected because two editable `SKILL.md` trees could drift.

### Guarantee regular installed files

Inspect owned destinations with symbolic-link-aware metadata. A byte-identical regular file remains untouched, but an owned symbolic link is treated as outdated and atomically replaced with a regular temporary-file rename. Newly created directories and files use the existing copy/write pipeline; the installer never calls a symlink API.

Codex-only legacy cleanup remains scoped to Codex destinations because the CLI never owned those paths under Claude Code. This avoids deleting similarly named Claude user content.

### Normalize agent session identity in the shared runner

Add a runner preflight mode that prints the selected agent session ID. The runner prefers `CLAUDE_CODE_SESSION_ID` and falls back to `CODEX_THREAD_ID`, then exports the selected value as `CODEX_THREAD_ID` before invoking the existing CLI. This makes current templates and `report-error` include the right session without duplicating skills or changing the public template object.

Adding a second platform-specific runner was rejected because it creates the exact source drift the user wants to prevent. Renaming the template runtime key was deferred because existing user templates depend on it and installation support does not require a template migration.

### Keep validation build output ephemeral

Run Rust formatting, clippy, and all tests with a temporary `CARGO_TARGET_DIR` and disabled incremental compilation. Clean and remove that exact temporary target after validation, leaving the repository without a `target` directory.

## Risks / Trade-offs

- **[The legacy runtime key remains Codex-named]** → Keep it as a compatibility transport hidden behind the platform-neutral runner workflow and document the normalization.
- **[A user expects one command to install both agents]** → Document two explicit commands; each has a clear destination and independent success result.
- **[An existing owned file is a symlink to user-maintained content]** → Replace only exact installer-owned file paths and never traverse or remove the symlink target.
- **[Claude Code does not notice a newly created top-level skills directory immediately]** → Print and document a restart/reopen hint; refreshes within an already watched directory remain ordinary file updates.
- **[Validation rebuilds a large cache]** → Use an ephemeral target, record its peak size, and clean it before archive.

## Migration Plan

1. Add agent-aware CLI parsing, destination resolution, manifest filtering, copy semantics, and summaries while retaining Codex defaults.
2. Update the canonical skills and identical runners for cross-agent session preflight; do not create a Claude source tree.
3. Add isolated project/global tests for both agents, environment overrides, regular-file installation, symlink replacement, and standalone binaries.
4. Update the English README and main-purpose wording, then validate using an ephemeral target directory.
5. Sync the delta specs, archive, commit, and push. Existing Codex users need no command change; Claude Code users run the same installer with `--agent claude-code`.
