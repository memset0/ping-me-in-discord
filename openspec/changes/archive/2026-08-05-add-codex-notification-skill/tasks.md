## 1. Secret-safe configuration discovery

- [x] 1.1 Add optional validated avatar profile descriptions without changing avatar rendering behavior.
- [x] 1.2 Implement human and JSON `channels list` output containing aliases and the effective default only.
- [x] 1.3 Implement human and JSON `avatar list` output containing safe profile metadata only.
- [x] 1.4 Cover configuration compatibility, description validation, JSON schemas, defaults, and secret omission with tests.

## 2. Codex runtime template metadata

- [x] 2.1 Read and normalize optional `CODEX_THREAD_ID` into reserved runtime template metadata.
- [x] 2.2 Conditionally append the Codex thread ID to the embedded and example starter metadata header.
- [x] 2.3 Test rendering with and without the environment value and preserve reserved-runtime collision handling.

## 3. Bounded error delivery

- [x] 3.1 Add the template-independent `report-error` command and fixed secret-safe payload.
- [x] 3.2 Implement requested-channel resolution, distinct default fallback, state persistence, and non-recursive terminal failure behavior.
- [x] 3.3 Test unknown channels, duplicate/default candidates, payload content, and successful or failed fallback paths without live Discord access.

## 4. Codex skills

- [x] 4.1 Scaffold `discord-notify` and `discord-agent-notify` with the standard skill initializer and generated UI metadata.
- [x] 4.2 Implement the simple skill's thread check, JSON-guided channel/avatar selection, dry-run verification, and send workflow.
- [x] 4.3 Implement the strict skill's six fixed status/profile mappings and compact message policy.
- [x] 4.4 Add byte-identical safe runners that report each wrapped CLI failure once while preserving the original exit status.
- [x] 4.5 Document the commands, optional avatar descriptions, template migration, and both skill workflows without exposing local configuration.

## 5. Verification and local acceptance

- [x] 5.1 Run formatting, clippy, the complete Rust test suite, shell syntax checks, skill validation, and strict OpenSpec validation.
- [x] 5.2 Verify both runner copies are identical and repository output contains no local token or channel configuration.
- [x] 5.3 Update only the external local user template and send one simple-skill and one strict-skill message to the configured `test` channel.

## 6. Generated avatar identity repair

- [x] 6.1 Replace generated-avatar mutation with Bot-provisioned webhook identities cached by resolved channel and PNG digest, while preserving the effective message username.
- [x] 6.2 Add backward-compatible state migration that restores legacy-modified base webhooks and preserve remote-URL and avatar-less delivery behavior.
- [x] 6.3 Add deterministic coverage for identity creation, cache reuse, distinct digests, prerequisite failures, username preservation, and legacy reset behavior.
- [x] 6.4 Run all repository verification and send all six strict states to the configured `test` channel, confirming distinct webhook identities and non-default avatar hashes.

## 7. Strict status avatar palette

- [x] 7.1 Measure the cached Twemoji artwork colors and revise the strict `started`, `progress`, `success`, and `error` mappings.
- [x] 7.2 Add optional emoji foreground configuration and CLI arguments with alpha-preserving, shape-preserving recoloring.
- [x] 7.3 Add deterministic renderer, configuration, CLI, documentation, and skill validation coverage.
- [x] 7.4 Run repository verification, send the four revised states to the configured `test` channel for visual acceptance, and reinstall the local release binary.

## 8. Config-owned strict status profiles

- [x] 8.1 Add all six status avatar profiles to starter and example configuration, with `error` using the accepted scale `0.576`.
- [x] 8.2 Simplify the strict skill to select `--avatar <status>` only, and update regression tests and documentation so visual fields do not appear in the skill.
- [x] 8.3 Update only the external private configuration, run complete validation, reinstall the local release binary, and send a final `--avatar error` acceptance message to the configured `test` channel.
