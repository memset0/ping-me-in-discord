## Context

The two Codex skills currently live only under the repository's `.codex/skills/` tree. Release archives contain the Rust binaries, so users of those archives have no stable source path from which to copy the skills. The CLI already supports configuration-independent subcommands such as `init`; the new installer must likewise run before Discord configuration exists. See `proposal.md` and the `agent-skill-installation` delta spec for the behavior contract.

OpenSpec's current integration model writes tool-owned skills below a project-specific skills root and refreshes only its own named directories. Codex uses `.codex/skills` at project scope and its home `skills` directory globally. The notification CLI can adopt the same ownership boundary without taking on OpenSpec's multi-tool generation system.

## Goals / Non-Goals

**Goals:**

- Make the release binary self-contained for installing both Codex skills.
- Resolve project and global destinations deterministically and without loading Discord configuration.
- Make repeated installs safe, update owned files, and preserve every unrelated skill.
- Keep path resolution and filesystem behavior directly testable without writing to the developer's actual home directory.

**Non-Goals:**

- Installing Claude Code or other agent formats in this version.
- Managing Codex processes, automatically reloading an existing session, or installing the `pingme` binary itself.
- Removing unknown or obsolete files from a user-owned skills tree.

## Decisions

### Use an explicit `skills install --scope` command

The CLI will add a `skills` command group with an `install` subcommand and a required `--scope project|global` value. Project scope resolves from the process current working directory; users select another project by changing directory first, matching OpenSpec's project initialization flow. Global scope honors a non-empty `CODEX_HOME`, otherwise it resolves the current user's home and appends `.codex/skills`.

Requiring scope is preferable to silently choosing project or global state. The command group also leaves room for future `list`, `uninstall`, or additional-agent options without changing today's invocation.

### Embed the repository skill files at compile time

A dedicated installer module will define a static manifest for the six owned files and embed each source file as bytes at compile time. This makes the checked-in skill directories the single source of truth while allowing release binaries to install without Git, network access, or a package manager. The manifest records which runner files need executable permission on Unix.

Generating skill bodies from Rust string literals was rejected because it would create a second copy that could drift from the repository-local skills and their validators.

### Update known files without owning the entire destination root

The installer will create the two known skill directories, compare existing file bytes, and replace only missing or different manifest files. It will not enumerate and delete other skill names or unknown files. On Unix it will ensure the two runner files are executable even when their contents are already current.

This narrower ownership model follows OpenSpec's documented behavior of refreshing only its own prefixed skills and is safer than replacing `.codex/skills` wholesale. The result will report the resolved destination and counts for created, updated, and unchanged files.

### Separate destination resolution from environment access

Path resolution will accept captured current-directory, `CODEX_HOME`, and home-directory inputs in an internal helper. Production code captures the real environment, while unit tests provide temporary paths. CLI integration tests will additionally set `CODEX_HOME` on the child process to prove the standalone binary path without touching the real global installation.

### Document installation beside binary setup

The README's installation section will show both copyable commands, their exact Codex destinations, update semantics, and `$discord-notify` / `$discord-agent-notify` invocation names. It will explicitly say to start a new Codex session or restart/reopen Codex so newly written skills are discovered.

## Risks / Trade-offs

- **[Bundled assets can drift from tests]** → Compile directly from checked-in skill files and assert installed bytes match those files in integration tests.
- **[A partial filesystem failure leaves only some files updated]** → Write each file through a sibling temporary file and rename it into place; report the failing destination clearly. Already completed files remain valid.
- **[Concurrent installers target the same directory]** → Use unique temporary filenames and atomic per-file replacement; both processes converge on identical embedded bytes.
- **[Users modify bundled skill files in place]** → Reinstallation intentionally restores CLI-owned files; README documents the refresh behavior and recommends separate skill names for custom variants.
- **[Codex does not reload skills in an active session]** → Print and document a restart/reopen instruction after successful installation.

## Migration Plan

1. Ship the installer in the next binary without modifying any existing global or project skill tree automatically.
2. Users explicitly run one scope command to create or refresh the two owned skills.
3. Future releases reuse the same command to refresh embedded assets; unrelated skills remain untouched.

Rollback removes the command and embedded manifest from later binaries. Skills already written to user or project directories remain ordinary files and can continue to work or be removed manually.
