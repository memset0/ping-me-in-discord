## 1. Rename CLI entry points

- [x] 1.1 Rename the Cargo long binary target and source entry point to `ping-me-in-discord` while preserving `pingme`.
- [x] 1.2 Update command diagnostics, initialization comments, and CLI tests to expose exactly the two supported executable names.

## 2. Update installation and releases

- [x] 2.1 Package and install `ping-me-in-discord` plus `pingme` in the release workflow and Unix installer.
- [x] 2.2 Retire only the exact legacy `notify-me-on-discord` executable after both replacements succeed, with an isolated installer migration test that preserves neighboring user files.

## 3. Update documentation

- [x] 3.1 Update the English README, configuration examples, and release documentation to use the renamed long command and explain the migration.

## 4. Validate and bound build artifacts

- [x] 4.1 Run formatting, linting, all tests, metadata assertions, shell validation, repository scans, and strict OpenSpec validation with incremental compilation disabled.
- [x] 4.2 Record the validation build size, clean the project Cargo target, and verify no persistent project build directory remains.
