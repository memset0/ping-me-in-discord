## 1. CLI and destination resolution

- [x] 1.1 Add the `skills install` command group with a required project/global scope and configuration-independent dispatch.
- [x] 1.2 Implement deterministic project, `CODEX_HOME`, and default user-home destination resolution with actionable errors.

## 2. Bundled skill installation

- [x] 2.1 Add a compile-time manifest containing every file from both repository-local Codex skills.
- [x] 2.2 Implement directory creation, byte comparison, per-file atomic replacement, and created/updated/unchanged reporting without touching unrelated skills.
- [x] 2.3 Preserve executable runner permissions on Unix and print the resolved destination plus Codex reload guidance.

## 3. Verification coverage

- [x] 3.1 Add unit tests for all destination-resolution branches and installation update accounting.
- [x] 3.2 Add CLI integration tests for project and overridden-global installation, embedded byte fidelity, executable runners, idempotent refresh, missing scope, and preservation of unrelated skills.

## 4. Documentation and validation

- [x] 4.1 Add project and global skill installation commands, path rules, refresh semantics, invocation names, and restart guidance to `README.md`.
- [x] 4.2 Run formatting, clippy, the complete test suite, skill validators, strict OpenSpec validation, and repository secret checks.
