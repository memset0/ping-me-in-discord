## 1. Rename and differentiate the Codex skills

- [x] 1.1 Rename both repository skill directories to `ping-me-send-message` and `ping-me-report-agent-status`, update their frontmatter and body boundaries, and repair the strict skill's GFM example.
- [x] 1.2 Regenerate matching `agents/openai.yaml` metadata and validate both renamed skill folders and their identical safe-execution runners.

## 2. Migrate bundled skill installation

- [x] 2.1 Update the embedded asset manifest, installation summary, and restart guidance to the two `ping-me-*` names.
- [x] 2.2 Remove only exact legacy-owned files after successful new-asset installation, prune only empty legacy directories, and report legacy removal counts.
- [x] 2.3 Update integration and unit tests for new installation, refresh, permission repair, idempotence, legacy cleanup, and preservation of unrelated files.

## 3. Rename project and distribution identity

- [x] 3.1 Rename the Cargo package and library target to `ping-me-in-discord` / `ping_me_in_discord`, update internal crate references and project HTTP user agents, and retain both explicit executable targets.
- [x] 3.2 Update the GitHub repository URL, installer default repository and archive prefix, release workflow artifact names, README branding and install links, release documentation, and notices while retaining the existing configuration namespace.

## 4. Verify the completed change

- [x] 4.1 Run formatting, all-target clippy, the complete test suite, both skill validators, and focused scans for stale current names or accidental compatibility renames.
- [x] 4.2 Verify the strict skill through GitHub's GFM renderer, validate the OpenSpec change strictly, and confirm the exact final diff is scoped to this change.
