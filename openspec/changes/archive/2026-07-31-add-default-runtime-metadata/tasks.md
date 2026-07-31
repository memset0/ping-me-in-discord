## 1. Runtime metadata context

- [x] 1.1 Add Rust 1.87-compatible cross-platform identity and zoned-time dependencies and refresh the lockfile
- [x] 1.2 Implement a typed runtime snapshot with normalized user and hostname fallbacks plus consistent local, Unix, and ISO 8601 timestamps
- [x] 1.3 Inject the reserved runtime object into every render and reject `runtime` collisions from JSON data or `--var` before network access

## 2. Default template and documentation

- [x] 2.1 Replace the embedded and example starter templates with the exact approved blockquote-first two-line Markdown layout
- [x] 2.2 Document runtime fields, formatting, collision behavior, privacy implications, customization, and upgrade preservation

## 3. Automated coverage

- [x] 3.1 Add deterministic unit tests for runtime fields, a single timestamp instant, identity fallbacks, normalization, and reserved-key rejection
- [x] 3.2 Add rendering and initialization tests for the exact starter layout, caller Markdown preservation, content limits, and non-overwrite behavior

## 4. Verification and local adoption

- [x] 4.1 Run formatting, strict linting, the complete test suite, Rust 1.87 checks, installer checks, release compilation, and strict OpenSpec validation
- [x] 4.2 Install the verified binaries locally and update only the approved executable-adjacent `defaults.md` without changing credentials or repository state
- [x] 4.3 Validate and dry-run the local configuration, then send one live confirmation through the `test` channel alias
