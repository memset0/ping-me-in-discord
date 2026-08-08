## Context

See `proposal.md` for motivation. The package already uses the `ping-me-in-discord` identity, but Cargo declares explicit binaries named `notify-me-on-discord` and `pingme`. Release packaging and `install.sh` copy those two files, while diagnostics and documentation still contain the old long command. The persistent application namespace remains `discord-notification` and must not follow the executable rename.

The Unix installer owns the long executable it previously installed. A successful upgrade must avoid leaving three discoverable command files, but cleanup must not run before both replacement entry points are safely in place.

## Goals / Non-Goals

**Goals:**

- Produce exactly `ping-me-in-discord` and `pingme` as Cargo binary targets and release entry points.
- Give installer upgrades a narrow, deterministic migration from the old long executable.
- Keep command examples, diagnostics, tests, and release contents aligned.
- Run validation without leaving a multi-gigabyte persistent build cache.

**Non-Goals:**

- Do not retain `notify-me-on-discord` as a compatibility alias or symlink.
- Do not rename `pingme`.
- Do not rename `DISCORD_NOTIFICATION_*`, `discord-notification` configuration/data directories, or existing user configuration files.
- Do not rewrite historical archived OpenSpec artifacts solely to replace command examples.

## Decisions

### Rename the explicit Cargo target and source entry point

Change the long `[[bin]]` target to `ping-me-in-discord`, rename its source file to match, and disable automatic binary discovery. Keep the shared library and `pingme` target unchanged. Explicit targets make the supported executable set deterministic and let metadata tests assert that the old name is absent.

Keeping an old compatibility target was rejected because the user explicitly wants the project to expose two CLI names, not three.

### Retire only the installer-owned legacy path after successful replacement

Update `install.sh` to validate, chmod, and atomically place both `ping-me-in-discord` and `pingme` first. Only after both moves succeed will it inspect `${install_dir}/notify-me-on-discord`; if that exact path exists or is a symlink, it will remove it and report the migration. It will not use globs or recursive deletion.

This ordering preserves a working old command if download, checksum, unpacking, validation, or either new-file installation fails. Removal failure makes the installer return nonzero rather than claiming that migration succeeded.

### Test the installer migration with local fixtures

Add a Unix integration test that creates a release archive containing two harmless fixture executables, intercepts `curl` with a temporary fake executable, and runs the real installer against a temporary installation directory. The test will verify both new entry points, removal of only the legacy executable, and preservation of portable `config.toml`, `templates/`, and unrelated files.

This exercises observable shell behavior without network access or modifying the developer's actual installation.

### Keep build artifacts ephemeral during validation

Disable Rust incremental compilation for validation. Build in the normal project target only while checks are running, record its peak size, then run `cargo clean` after validation so the repository returns to no persistent build directory. Global Cargo registry and Git caches remain untouched.

Using the existing global dependency cache avoids redundant downloads, while final cleanup satisfies the requested disk-space constraint.

## Risks / Trade-offs

- **[Existing automation invokes the retired command]** → Mark the change breaking and document both supported replacements prominently.
- **[Installer removes the old command before replacement succeeds]** → Perform legacy cleanup only after both atomic destination moves complete.
- **[Installer cleanup accidentally touches user files]** → Remove one explicit legacy path without recursion and test preservation of neighboring files.
- **[Executable rename accidentally changes configuration discovery]** → Keep `APP_DIR` and environment variables unchanged and retain configuration-discovery tests.
- **[Validation rebuilds a large target directory]** → Disable incremental compilation, report peak size, and clean the exact Cargo target at the end.

## Migration Plan

1. Rename the Cargo target and entry-point source, then update command-specific diagnostics and tests.
2. Update release packaging and installer file lists; add exact post-install legacy cleanup and an isolated migration test.
3. Replace active documentation and configuration examples with `ping-me-in-discord`, retaining a concise migration note where useful.
4. Run the complete validation suite with incremental compilation disabled, then clean the project target directory.
5. Sync specs, archive, commit, and push. Users upgrade by running the new installer or replacing explicit old command references with `ping-me-in-discord` or `pingme`.
