## 1. Restructure and simplify the README

- [x] 1.1 Replace the separate installation, initialization, and skill-installation introductions with one ordered Setup section covering binary installation, `config.toml`, validation, and optional skills.
- [x] 1.2 Rewrite the remaining guide to remove duplicated or obsolete detail, keep skill descriptions agent-neutral, and use `pingme` for every CLI command example after the two-entry-point introduction.

## 2. Validate the documentation change

- [x] 2.1 Verify the README retains required paths, environment variables, safety notes, reference links, and adapter-specific installation details without creating framework-specific product framing.
- [x] 2.2 Run Markdown/diff checks, command-spelling scans, and strict OpenSpec validation, then confirm no build cache was created.
