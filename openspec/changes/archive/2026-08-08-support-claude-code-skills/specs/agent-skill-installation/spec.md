## RENAMED Requirements

- FROM: `### Requirement: Release binaries contain complete Codex skills`
- TO: `### Requirement: Release binaries contain complete agent skills`

## MODIFIED Requirements

### Requirement: Skill installation requires an explicit scope
The CLI SHALL provide `skills install --scope <project|global> [--agent <codex|claude-code>]`. Scope SHALL remain required, while an omitted agent SHALL default to `codex` for compatibility. Codex project scope SHALL resolve to `.codex/skills` in the current directory; Codex global scope SHALL use the `skills` directory of a non-empty `CODEX_HOME`, falling back to `~/.codex/skills`. Claude Code project scope SHALL resolve to `.claude/skills` in the current directory; Claude Code global scope SHALL use the `skills` directory of a non-empty `CLAUDE_CONFIG_DIR`, falling back to `~/.claude/skills`. The command SHALL fail with an actionable diagnostic when it cannot resolve or create the selected destination.

#### Scenario: Install into the current project
- **WHEN** a user runs `pingme skills install --scope project` from a project root without selecting an agent
- **THEN** the bundled skills are installed below that directory's `.codex/skills`

#### Scenario: Install into an overridden global home
- **WHEN** a user runs `pingme skills install --scope global --agent codex` with a non-empty `CODEX_HOME`
- **THEN** the bundled skills are installed below `${CODEX_HOME}/skills`

#### Scenario: Install into the default global home
- **WHEN** Codex global scope is selected without `CODEX_HOME` and the user's home directory is available
- **THEN** the bundled skills are installed below `~/.codex/skills`

#### Scenario: Scope is omitted
- **WHEN** a user runs `pingme skills install --agent claude-code` without `--scope`
- **THEN** argument parsing fails without writing any skill files

#### Scenario: Install into a Claude Code project
- **WHEN** a user runs `pingme skills install --scope project --agent claude-code` from a project root
- **THEN** the bundled skills are installed below that directory's `.claude/skills`

#### Scenario: Install into an overridden Claude Code configuration directory
- **WHEN** a user selects Claude Code global scope with a non-empty `CLAUDE_CONFIG_DIR`
- **THEN** the bundled skills are installed below `${CLAUDE_CONFIG_DIR}/skills`

#### Scenario: Install into the default Claude Code configuration directory
- **WHEN** Claude Code global scope is selected without `CLAUDE_CONFIG_DIR` and the user's home directory is available
- **THEN** the bundled skills are installed below `~/.claude/skills`

### Requirement: Release binaries contain complete agent skills
The installed binary SHALL carry one canonical source for the `ping-me-send-message` and `ping-me-report-agent-status` instructions and runners. A Codex installation SHALL contain each skill's `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh`; a Claude Code installation SHALL copy the same canonical `SKILL.md` and runner but SHALL omit Codex-only `agents/openai.yaml`. Every installed owned path SHALL be a regular file rather than a symbolic link, and the runner SHALL be executable on Unix. Installing the skills SHALL not require Git, network access, a package manager, or a source checkout.

#### Scenario: Install from a standalone binary
- **WHEN** the CLI binary installs Codex skills from a directory that does not contain the source repository
- **THEN** both complete Codex skill directories are created from assets embedded in that binary

#### Scenario: Runner permissions on Unix
- **WHEN** installation succeeds on a Unix system for either supported agent
- **THEN** each installed `scripts/run-pingme.sh` is a regular executable file

#### Scenario: Install Claude Code from a standalone binary
- **WHEN** the CLI binary installs Claude Code skills without a source checkout
- **THEN** both skill directories contain regular-file copies of the canonical `SKILL.md` and runner without a duplicated Claude-specific source or Codex-only UI metadata

### Requirement: Skill installation is repeatable and narrowly owned
For the selected agent, the installer SHALL create missing owned files, replace outdated or symbolic-link owned files with regular-file copies, and leave byte-identical regular files unchanged. Codex migration SHALL remove only the previously owned `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh` files below the legacy `discord-notify` and `discord-agent-notify` names, and it MAY remove their directories only after those directories become empty. Claude Code installation SHALL NOT claim or remove those never-installed legacy paths. The installer SHALL NOT remove or modify any other skill directory or unrelated file below the selected skills root. A successful command SHALL report its selected agent, resolved scope, destination, file update counts, and legacy removal count.

#### Scenario: Refresh an older bundled skill
- **WHEN** an owned installed file under either current skill name differs from the version embedded in the running binary
- **THEN** the command replaces that file with a regular-file copy of the embedded version and succeeds

#### Scenario: Repeat an unchanged installation
- **WHEN** all owned installed files already match the embedded assets as regular files and no applicable legacy owned files remain
- **THEN** the command succeeds without changing their contents or reporting a legacy removal

#### Scenario: Migrate a previous installation
- **WHEN** the Codex destination contains files installed under `discord-notify` or `discord-agent-notify`
- **THEN** the command installs both current skill names, removes the six exact legacy owned files that are present, and reports how many legacy files were removed

#### Scenario: Preserve unrelated files inside a legacy directory
- **WHEN** a Codex legacy skill directory contains a file outside the three exact paths previously owned by the installer
- **THEN** installation preserves that file and its containing directory while removing only the legacy owned files

#### Scenario: Preserve unrelated skills
- **WHEN** the selected destination contains a third-party skill outside the current and applicable legacy bundled skill names
- **THEN** installation leaves that skill and its files unchanged

#### Scenario: Replace an owned symbolic link
- **WHEN** an owned destination file is a symbolic link even if its target has byte-identical content
- **THEN** installation replaces that path with a regular file containing the embedded asset

### Requirement: Installation usage is documented
The English README SHALL show copyable Codex and Claude Code commands for project and global installation, explain `.codex/skills`, `.claude/skills`, `CODEX_HOME`, and `CLAUDE_CONFIG_DIR` resolution, state that installation copies regular files rather than creating symbolic links, and describe refresh and Codex legacy migration behavior. It SHALL list `$ping-me-send-message` and `$ping-me-report-agent-status` for Codex plus `/ping-me-send-message` and `/ping-me-report-agent-status` for Claude Code, distinguish their free-form and lifecycle purposes, and explain when the selected agent may need to be restarted to discover a newly created top-level skills directory.

#### Scenario: User follows project documentation
- **WHEN** a user follows the README from the root of another project
- **THEN** the documented commands can install both skills into either that project's `.codex/skills` or `.claude/skills` directory

#### Scenario: User follows global documentation
- **WHEN** a user follows the README global commands
- **THEN** the documentation explains how `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, or the corresponding default home path determines each destination
