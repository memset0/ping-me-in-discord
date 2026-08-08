## MODIFIED Requirements

### Requirement: Release binaries contain complete Codex skills
The installed binary SHALL carry the `ping-me-send-message` and `ping-me-report-agent-status` skill assets needed for standalone installation. Each installed skill SHALL contain its `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh`; the runner SHALL be executable on Unix. Installing the skills SHALL not require Git, network access, a package manager, or a source checkout.

#### Scenario: Install from a standalone binary
- **WHEN** the CLI binary is run from a directory that does not contain the source repository
- **THEN** both complete renamed skill directories are created from assets embedded in that binary

#### Scenario: Runner permissions on Unix
- **WHEN** installation succeeds on a Unix system
- **THEN** each installed `scripts/run-pingme.sh` has executable permission

### Requirement: Skill installation is repeatable and narrowly owned
The installer SHALL create missing owned files, replace outdated files for `ping-me-send-message` and `ping-me-report-agent-status`, and leave byte-identical files unchanged. During migration, it SHALL remove only the previously owned `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh` files below the legacy `discord-notify` and `discord-agent-notify` names, and it MAY remove their directories only after those directories become empty. It SHALL NOT remove or modify any other skill directory or unrelated file below the selected skills root. A successful command SHALL report its resolved scope, destination, file update counts, and legacy removal count so the user can verify where Codex will load the skills.

#### Scenario: Refresh an older bundled skill
- **WHEN** an owned installed file under either new skill name differs from the version embedded in the running binary
- **THEN** the command replaces that file with the embedded version and succeeds

#### Scenario: Repeat an unchanged installation
- **WHEN** all owned installed files already match the embedded assets and no legacy owned files remain
- **THEN** the command succeeds without changing their contents or reporting a legacy removal

#### Scenario: Migrate a previous installation
- **WHEN** the destination contains files installed under `discord-notify` or `discord-agent-notify`
- **THEN** the command installs both new skill names, removes the six exact legacy owned files that are present, and reports how many legacy files were removed

#### Scenario: Preserve unrelated files inside a legacy directory
- **WHEN** a legacy skill directory contains a file outside the three exact paths previously owned by the installer
- **THEN** installation preserves that file and its containing directory while removing only the legacy owned files

#### Scenario: Preserve unrelated skills
- **WHEN** the destination contains a third-party skill outside the four current and legacy bundled skill names
- **THEN** installation leaves that skill and its files unchanged

### Requirement: Installation usage is documented
The README SHALL show copyable commands for project and global installation, explain their resolved Codex paths, refresh and legacy migration behavior, list `$ping-me-send-message` and `$ping-me-report-agent-status` as the installed invocation names, distinguish their free-form and lifecycle purposes, and tell users to restart or reopen Codex after installation.

#### Scenario: User follows project documentation
- **WHEN** a user follows the README from the root of another project
- **THEN** the documented command installs both renamed skills into that project's `.codex/skills` directory

#### Scenario: User follows global documentation
- **WHEN** a user follows the README global command
- **THEN** the documentation explains whether `CODEX_HOME` or the default home path determines the destination
