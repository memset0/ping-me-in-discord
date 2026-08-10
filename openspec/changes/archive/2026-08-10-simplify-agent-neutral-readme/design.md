## Context

See `proposal.md` for motivation. The current README is 328 lines and introduces binary installation, skill installation, initialization, and agent notification behavior in separate sections, with duplicated skill guidance and mixed executable spellings.

## Goals / Non-Goals

**Goals:**

- Give a new user one ordered path from installation to a working configuration and optional skills.
- Keep general product and skill language framework-neutral.
- Make `pingme` the canonical documentation spelling after both executable names are introduced.
- Preserve links to complete references for advanced configuration and release details.

**Non-Goals:**

- Change CLI arguments, binary names, install destinations, configuration formats, or supported framework adapters.
- Hide the adapter-specific paths and invocation conventions users need to install the skills correctly.

## Decisions

### Use one top-level Setup section

The README will place binary installation, `pingme init`, essential `config.toml` editing and validation, and optional skill installation in one numbered flow. Separate introductory Installation, Installing skills, and Initialization sections will be removed. Keeping those sections was rejected because it forces readers to assemble the setup order themselves.

### Separate neutral concepts from adapter details

General descriptions will call the two assets “agent notification skills.” Codex and Claude Code will appear only in a compact supported-target table or commands where paths, environment variables, and invocation syntax actually differ. Removing all framework names was rejected because users still need accurate destination guidance.

### Canonicalize examples on `pingme`

The introduction will show both equivalent entry points once. Every later executable invocation will use `pingme`; long-name mentions remain only when discussing the project identity, installed files, release assets, or URLs.

### Prefer references over exhaustive inline explanation

The README will retain quick examples and safety-critical behavior, then link to `docs/configuration.md`, `examples/config.toml`, and `docs/releases.md` for complete details. This reduces duplication without removing discoverability.

## Risks / Trade-offs

- **[Important edge cases become less visible]** → Keep short notes for rootless installation, secret environment variables, precedence, avatar prerequisites, and skill copy semantics, then link to the detailed references.
- **[Framework-neutral wording could imply every framework has a built-in installer]** → State that the skill content is agent-agnostic while listing the currently supported installation adapters explicitly.
- **[Mechanical command replacement could corrupt URLs or artifact names]** → Validate executable command lines separately from prose references to the long project name.
