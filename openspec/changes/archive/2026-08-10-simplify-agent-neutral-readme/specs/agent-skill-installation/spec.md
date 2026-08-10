## MODIFIED Requirements

### Requirement: Installation usage is documented
The English README SHALL present the bundled notification skills as agent-agnostic workflows maintained from one canonical source. It SHALL describe supported framework installation adapters compactly within the unified setup flow, currently including Codex and Claude Code, without using either framework as the product or skill identity. It SHALL show copyable project and global installation commands, explain `.codex/skills`, `.claude/skills`, `CODEX_HOME`, and `CLAUDE_CONFIG_DIR` resolution, state that installation copies regular files rather than creating symbolic links, and summarize refresh and applicable legacy migration behavior. It SHALL distinguish the free-form and lifecycle-status skills and tell users to restart or reopen an agent when a newly created skills directory is not discovered immediately.

#### Scenario: Skills are introduced without a framework identity
- **WHEN** a reader reaches the README skill overview
- **THEN** it describes `ping-me-send-message` and `ping-me-report-agent-status` as agent notification skills rather than Codex or Claude Code skills

#### Scenario: User follows project documentation
- **WHEN** a user follows the README from the root of another project
- **THEN** the documented adapter commands can install both skills into either that project's `.codex/skills` or `.claude/skills` directory

#### Scenario: User follows global documentation
- **WHEN** a user follows the README global commands
- **THEN** the documentation explains how `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, or the corresponding default home path determines each supported adapter's destination

#### Scenario: Framework-specific details stay localized
- **WHEN** the README needs to explain a supported agent's destination or invocation convention
- **THEN** it names that framework only in the compact adapter-specific installation guidance rather than emphasizing it in the product introduction or general skill description
