## Context

`README.md` is the public overview and operational guide for installation, configuration, templates, avatars, agent skills, and development. Its examples and links are already authoritative and must survive the language rewrite. `AGENTS.md` is a symbolic link to `CLAUDE.md`, so one instruction edit reaches both Codex and Claude Code.

## Goals / Non-Goals

**Goals:**

- Translate all README headings and explanatory prose into natural technical English.
- Preserve every documented feature, security constraint, command, configuration sample, and local documentation link.
- Make the English-only README policy durable for future agent edits.

**Non-Goals:**

- Translating other documentation files in this change.
- Reorganizing product behavior, changing command syntax, or adding new features.
- Rewriting literal user-facing output, template content, or example values unless translation is necessary for explanation.

## Decisions

### Rewrite by semantic section rather than sentence substitution

Each existing section will be rewritten as coherent English documentation while retaining the same technical coverage and approximate structure. This produces more natural prose than a literal line-by-line translation and makes omissions easier to detect by comparing headings, code fences, commands, and links before and after.

### Preserve executable examples exactly

Shell commands, TOML, Jinja, Markdown templates, option names, environment variables, color values, and file paths will remain unchanged except for comment prose that must become English. Local Markdown links will retain their destinations while link labels become English.

### Store the language policy in the shared project instructions

Add an English documentation rule to `CLAUDE.md`. Because `AGENTS.md` points to that file, both supported agents receive the same instruction without duplicating or replacing the symlink.

## Risks / Trade-offs

- **[Technical detail is lost during translation]** → Compare section inventory, commands, code fences, and local links before and after; review the final diff in full.
- **[Chinese remains in prose or examples]** → Scan `README.md` for Han characters and treat any match as a validation failure.
- **[A documented command drifts from the CLI]** → Run representative `--help` checks and retain the already tested command spellings.
- **[The maintenance rule is invisible to one agent]** → Verify `AGENTS.md` remains a symbolic link to `CLAUDE.md` after editing.

## Migration Plan

Merge the English README and shared maintenance rule together. No runtime migration or rollback procedure is required; reverting the documentation commit restores the prior language without affecting the application.
