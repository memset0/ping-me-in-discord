## Why

The repository has been renamed to `ping-me-in-discord`, while its package metadata, release assets, documentation, and bundled Codex skills still use older or easily confused names. Aligning the project identity and giving the two skills action-specific names makes installation and skill selection predictable, and it also provides an opportunity to repair the strict skill's broken GitHub Markdown rendering.

## What Changes

- Rename the Rust package, library crate, repository metadata, README identity, release archives, installer source repository, and related project-facing references to `ping-me-in-discord`.
- Preserve the existing `notify-me-on-discord` and `pingme` executable names and the existing `discord-notification` configuration/data directories for compatibility.
- **BREAKING**: Rename the free-form Codex skill from `discord-notify` to `ping-me-send-message` and narrow it to intentional free-form Discord messages with discoverable channel and avatar choices.
- **BREAKING**: Rename the strict lifecycle skill from `discord-agent-notify` to `ping-me-report-agent-status` and make its trigger boundary explicitly limited to fixed agent lifecycle reports.
- Fix the strict skill's nested example so GitHub Flavored Markdown does not absorb the remaining workflow and failure rules into a code block.
- Update the CLI skill installer to install the new names and remove only the exact legacy skill files it previously owned, while preserving unrelated files and skill directories.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-notification-skill`: Rename both Codex skills, define mutually exclusive free-form-message and agent-lifecycle boundaries, and require valid rendered Markdown for their instructions.
- `agent-skill-installation`: Bundle and install the renamed skills and safely retire the exact files owned under the legacy skill names.
- `user-installation-and-releases`: Publish and download release archives under the renamed `ping-me-in-discord` project identity while retaining both existing executable entry points.

## Impact

- Affected skill assets: `.codex/skills/`, their metadata, runners, and validation tests.
- Affected Rust code: Cargo package/library identity and the embedded skill installer manifest and migration behavior.
- Affected distribution surfaces: `install.sh`, release workflow archive names, repository URLs, release documentation, and README.
- Existing explicit references to `$discord-notify` or `$discord-agent-notify` must migrate to `$ping-me-send-message` or `$ping-me-report-agent-status` after reinstalling the bundled skills.
