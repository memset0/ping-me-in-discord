## MODIFIED Requirements

### Requirement: Release binaries contain complete agent skills
The installed binary SHALL carry one canonical source for the `ping-me-send-message`, `ping-me-report-work-progress`, and `ping-me-report-turn-outcome` instructions and runners. A Codex installation SHALL contain each skill's `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh`; a Claude Code installation SHALL copy the same canonical `SKILL.md` and runner but SHALL omit Codex-only `agents/openai.yaml`. Every installed owned path SHALL be a regular file rather than a symbolic link, and the runner SHALL be executable on Unix. Installing the skills SHALL not require Git, network access, a package manager, or a source checkout.

#### Scenario: Install from a standalone binary
- **WHEN** the CLI binary installs Codex skills from a directory that does not contain the source repository
- **THEN** all three complete Codex skill directories are created from assets embedded in that binary

#### Scenario: Runner permissions on Unix
- **WHEN** installation succeeds on a Unix system for either supported agent
- **THEN** each installed `scripts/run-pingme.sh` is a regular executable file

#### Scenario: Install Claude Code from a standalone binary
- **WHEN** the CLI binary installs Claude Code skills without a source checkout
- **THEN** all three skill directories contain regular-file copies of the canonical `SKILL.md` and runner without a duplicated Claude-specific source or Codex-only UI metadata

### Requirement: Skill installation is repeatable and narrowly owned
For the selected agent, the installer SHALL create missing owned files, replace outdated or symbolic-link owned files with regular-file copies, and leave byte-identical regular files unchanged. Codex migration SHALL remove only the previously owned `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh` files below the legacy `discord-notify`, `discord-agent-notify`, and retired `ping-me-report-agent-status` names, and it MAY remove their directories only after those directories become empty. Claude Code migration SHALL remove only the previously owned `SKILL.md` and `scripts/run-pingme.sh` files below the retired `ping-me-report-agent-status` name while leaving Codex-only metadata and older never-installed legacy names untouched. The installer SHALL NOT remove or modify any other skill directory or unrelated file below the selected skills root. A successful command SHALL report its selected agent, resolved scope, destination, file update counts, and legacy removal count.

#### Scenario: Refresh an older bundled skill
- **WHEN** an owned installed file under any current skill name differs from the version embedded in the running binary
- **THEN** the command replaces that file with a regular-file copy of the embedded version and succeeds

#### Scenario: Repeat an unchanged installation
- **WHEN** all owned installed files already match the embedded assets as regular files and no applicable legacy owned files remain
- **THEN** the command succeeds without changing their contents or reporting a legacy removal

#### Scenario: Migrate a previous installation
- **WHEN** the Codex destination contains owned files installed under `discord-notify`, `discord-agent-notify`, or `ping-me-report-agent-status`
- **THEN** the command installs all three current skill names, removes each exact applicable legacy owned file that is present, and reports how many legacy files were removed

#### Scenario: Migrate the retired Claude Code status skill
- **WHEN** the Claude Code destination contains the previously installed `SKILL.md` and runner under `ping-me-report-agent-status`
- **THEN** the command installs all three current skill names and removes only those two retired owned files

#### Scenario: Preserve unrelated files inside a legacy directory
- **WHEN** an applicable legacy skill directory contains a file outside the exact paths previously owned by the installer
- **THEN** installation preserves that file and its containing directory while removing only the legacy owned files

#### Scenario: Preserve unrelated skills
- **WHEN** the selected destination contains a third-party skill outside the current and applicable legacy bundled skill names
- **THEN** installation leaves that skill and its files unchanged

#### Scenario: Replace an owned symbolic link
- **WHEN** an owned destination file is a symbolic link even if its target has byte-identical content
- **THEN** installation replaces that path with a regular file containing the embedded asset

### Requirement: Installation usage is documented
The English README SHALL present the bundled notification skills as agent-agnostic workflows maintained from one canonical source. It SHALL describe supported framework installation adapters compactly within the unified setup flow, currently including Codex and Claude Code, without using either framework as the product or skill identity. It SHALL show copyable project and global installation commands, explain `.codex/skills`, `.claude/skills`, `CODEX_HOME`, and `CLAUDE_CONFIG_DIR` resolution, state that installation copies regular files rather than creating symbolic links, and summarize refresh and applicable legacy migration behavior. It SHALL distinguish explicit free-form messages, continuing-work updates, and end-of-turn outcomes; explain same-conversation activation and explicit disablement for automatic policies; and tell users to restart or reopen an agent when a newly created skills directory is not discovered immediately.

#### Scenario: Skills are introduced without a framework identity
- **WHEN** a reader reaches the README skill overview
- **THEN** it describes all three `ping-me-*` workflows as agent notification skills rather than Codex or Claude Code skills

#### Scenario: User follows project documentation
- **WHEN** a user follows the README from the root of another project
- **THEN** the documented adapter commands can install all three skills into either that project's `.codex/skills` or `.claude/skills` directory

#### Scenario: User follows global documentation
- **WHEN** a user follows the README global commands
- **THEN** the documentation explains how `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, or the corresponding default home path determines each supported adapter's destination

#### Scenario: Framework-specific details stay localized
- **WHEN** the README needs to explain a supported agent's destination or invocation convention
- **THEN** it names that framework only in the compact adapter-specific installation guidance rather than emphasizing it in the product introduction or general skill description

#### Scenario: Reader chooses the correct workflow
- **WHEN** a reader compares the three bundled skills
- **THEN** the README directs arbitrary messages to `ping-me-send-message`, in-progress reports to `ping-me-report-work-progress`, and one pre-yield result per enabled turn to `ping-me-report-turn-outcome`
