# agent-skill-installation Specification

## Purpose

Allow users to install or refresh the notification skills bundled with the CLI at a predictable Codex project or user-global location without cloning the source repository.

## Requirements

### Requirement: Skill installation requires an explicit scope
The CLI SHALL provide `skills install --scope <project|global>`. Project scope SHALL install below `.codex/skills` in the current working directory. Global scope SHALL install below the `skills` directory of a non-empty `CODEX_HOME`, falling back to `.codex/skills` in the current user's home directory when that environment variable is absent. The command SHALL fail with an actionable diagnostic when it cannot resolve or create the selected destination.

#### Scenario: Install into the current project
- **WHEN** a user runs `pingme skills install --scope project` from a project root
- **THEN** the bundled skills are installed below that directory's `.codex/skills`

#### Scenario: Install into an overridden global home
- **WHEN** a user runs `pingme skills install --scope global` with a non-empty `CODEX_HOME`
- **THEN** the bundled skills are installed below `${CODEX_HOME}/skills`

#### Scenario: Install into the default global home
- **WHEN** global scope is selected without `CODEX_HOME` and the user's home directory is available
- **THEN** the bundled skills are installed below `~/.codex/skills`

#### Scenario: Scope is omitted
- **WHEN** a user runs `pingme skills install` without `--scope`
- **THEN** argument parsing fails without writing any skill files

### Requirement: Release binaries contain complete Codex skills
The installed binary SHALL carry the `discord-notify` and `discord-agent-notify` skill assets needed for standalone installation. Each installed skill SHALL contain its `SKILL.md`, `agents/openai.yaml`, and `scripts/run-pingme.sh`; the runner SHALL be executable on Unix. Installing the skills SHALL not require Git, network access, a package manager, or a source checkout.

#### Scenario: Install from a standalone binary
- **WHEN** the CLI binary is run from a directory that does not contain the source repository
- **THEN** both complete skill directories are created from assets embedded in that binary

#### Scenario: Runner permissions on Unix
- **WHEN** installation succeeds on a Unix system
- **THEN** each installed `scripts/run-pingme.sh` has executable permission

### Requirement: Skill installation is repeatable and narrowly owned
The installer SHALL create missing owned files, replace outdated files for the two bundled skill names, and leave byte-identical files unchanged. It SHALL NOT remove or modify any other skill directory or unrelated file below the selected skills root. A successful command SHALL report its resolved scope and destination so the user can verify where Codex will load the skills.

#### Scenario: Refresh an older bundled skill
- **WHEN** an owned installed file differs from the version embedded in the running binary
- **THEN** the command replaces that file with the embedded version and succeeds

#### Scenario: Repeat an unchanged installation
- **WHEN** all owned installed files already match the embedded assets
- **THEN** the command succeeds without changing their contents

#### Scenario: Preserve unrelated skills
- **WHEN** the destination contains a third-party skill outside `discord-notify` and `discord-agent-notify`
- **THEN** installation leaves that skill and its files unchanged

### Requirement: Installation usage is documented
The README SHALL show copyable commands for project and global installation, explain their resolved Codex paths and refresh behavior, list the installed skill invocation names, and tell users to restart or reopen Codex after installation.

#### Scenario: User follows project documentation
- **WHEN** a user follows the README from the root of another project
- **THEN** the documented command installs both skills into that project's `.codex/skills` directory

#### Scenario: User follows global documentation
- **WHEN** a user follows the README global command
- **THEN** the documentation explains whether `CODEX_HOME` or the default home path determines the destination
