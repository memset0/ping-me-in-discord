## Context

See `proposal.md` for motivation. The Cargo package and library currently share the former `notify-me-on-discord` identity, release archives and installer URLs still point at `memset0/discord-notification`, and the embedded skill manifest owns six files below two similarly named skill directories. Both explicit binary targets are already declared separately, while configuration and state use a long-lived `discord-notification` directory namespace.

The skill installer promises narrow ownership: it may refresh its own files but must preserve unrelated skills. Renaming skill directories therefore needs an explicit migration instead of treating the old directories as generic third-party content. The strict skill also contains a multi-line fenced example nested in an ordered list whose unindented second message line ends the list item in GFM and causes the remainder of the document to render as code.

## Goals / Non-Goals

**Goals:**

- Give the two Codex skills visibly different, action-oriented names and mutually exclusive triggering language.
- Move all current embedded skill assets and installer output to the `ping-me-*` names without leaving active legacy copies.
- Align project, Cargo, repository, and release identity with `ping-me-in-discord`.
- Preserve existing command and configuration compatibility while making upgrades deterministic.

**Non-Goals:**

- Do not rename the `notify-me-on-discord` or `pingme` executables.
- Do not migrate or rename `discord-notification` configuration, state, cache, or environment-variable namespaces.
- Do not modify historical artifacts under `openspec/changes/archive/` merely to replace old names.
- Do not add Claude Code skill packaging in this change.

## Decisions

### Separate project identity from executable compatibility

Rename the Cargo package to `ping-me-in-discord` and the library target to `ping_me_in_discord`, while retaining the two explicit binary targets. Update internal crate references, repository metadata, README branding, third-party notices, HTTP user agents, and release archive prefixes. The installer will default to `memset0/ping-me-in-discord` and expect the new archive prefix.

Keeping the existing binary and application-directory names avoids breaking shell scripts, PATH entries, portable layouts, and existing XDG configuration. Renaming every user-facing path was rejected because the GitHub/project rename does not justify a configuration migration.

### Use action-specific `ping-me-*` skill names

Rename the free-form skill to `ping-me-send-message` and the lifecycle skill to `ping-me-report-agent-status`. Their frontmatter descriptions will state both positive and negative trigger conditions:

- `ping-me-send-message` handles intentional free-form content plus discoverable channel and avatar selection; it excludes lifecycle-only reports.
- `ping-me-report-agent-status` handles exactly one fixed lifecycle state, same-named status profiles, and the strict compact contract; it excludes arbitrary content and custom avatar selection.

UI metadata and default prompts will use the same exact names. Using the CLI namespace plus distinct verbs was chosen over retaining `discord-*` because the old names differed by only one word and encouraged overlapping implicit invocation.

### Install new assets before retiring exact legacy-owned files

The embedded manifest will contain only the six files below the new skill names. A separate legacy allowlist will identify the three previously owned relative files beneath each old name. Installation will:

1. Write or refresh all new assets using the existing atomic file replacement behavior.
2. Remove each present allowlisted legacy file and count successful removals.
3. Attempt to remove now-empty legacy subdirectories from deepest to shallowest, stopping at each legacy skill root and ignoring non-empty-directory results.

No wildcard or recursive deletion will be used. Extra files under either legacy directory and all other skills remain untouched. Installing new assets first ensures a failed write leaves the previous usable skills in place. A cleanup failure is still returned as an installation error because duplicate active skill definitions would leave the boundary ambiguous.

### Validate source structure without adding a Markdown dependency

Move the strict example's continuation line inside the ordered-list indentation and add a regression assertion that the closing fence precedes workflow step 7 and the `Failure rules` heading. Run both standard skill validation and GitHub's GFM rendering endpoint during completion verification; CI tests remain offline and dependency-free.

The `agents/openai.yaml` files will be regenerated with the skill-creator generator and checked against the renamed frontmatter rather than edited independently.

## Risks / Trade-offs

- **[Explicit prompts using old skill names stop working]** → Document the mapping and make a reinstall remove the old owned skill definitions so Codex exposes one unambiguous pair.
- **[A legacy directory contains user-added files]** → Remove only six exact allowlisted owned files and prune directories only when empty.
- **[Changing the package/library name breaks internal imports]** → Keep binary targets explicit and run all-target builds, clippy, and tests after updating every crate reference.
- **[Existing releases use the old archive prefix]** → The renamed installer targets new releases going forward; version overrides still select tags but use the new project archive convention.
- **[GitHub rendering behavior differs from a source-only test]** → Combine deterministic local assertions with one completion-time render through GitHub's GFM API.

## Migration Plan

1. Rename source skill directories and metadata, then update the embedded manifest and tests.
2. Add the exact legacy-file cleanup to `pingme skills install` and verify migration, idempotence, and unrelated-file preservation in temporary destinations.
3. Rename project/package/distribution metadata while retaining both executable targets and the existing application directory namespace.
4. Update documentation with the new repository URL, archive name, skill invocations, boundary descriptions, and old-to-new mapping.
5. Point the local `origin` remote at `memset0/ping-me-in-discord`, fetch `origin/master`, and use the repository's normal archive commit and push procedure.

Rollback consists of reverting the repository commit. Users who already migrated their installed skills can reinstall from an older binary to restore the legacy skill files, although explicit prompts written with the new names would then need the inverse rename.
