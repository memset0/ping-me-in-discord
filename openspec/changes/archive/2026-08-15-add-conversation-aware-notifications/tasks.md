## 1. Runtime and template context

- [x] 1.1 Add normalized agent, project, session name, and generic session ID fields with documented environment precedence and the legacy compatibility alias.
- [x] 1.2 Replace the starter and example default templates with the compact two-line context header and update focused rendering/runtime tests.

## 2. Notification skill workflows

- [x] 2.1 Scaffold and author `ping-me-report-work-progress` with continuing-work states and conversation-scoped activation guidance.
- [x] 2.2 Scaffold and author `ping-me-report-turn-outcome` with one pre-yield outcome, conversation-scoped activation guidance, and non-blocking notification failure behavior.
- [x] 2.3 Update `ping-me-send-message` boundary guidance and enhance all three identical runners with generic agent/project/session context handling.

## 3. Skill distribution and migration

- [x] 3.1 Embed the three canonical skills, install the correct adapter-specific files, and migrate exact owned files under the retired status-skill name.
- [x] 3.2 Update installation and runner integration tests for file counts, migration safety, context precedence, GFM structure, status boundaries, and identical runner copies.

## 4. User documentation

- [x] 4.1 Update the English README to distinguish all three skills and explain same-conversation activation, disablement, context metadata, and adapter installation.
- [x] 4.2 Update the configuration reference and complete examples for the generic runtime schema and two-line default template.

## 5. Verification and demonstration

- [x] 5.1 Run formatting, linting, isolated full tests, skill validation, and strict OpenSpec validation without recreating the repository `target/` cache.
- [x] 5.2 Dry-run the exact current-session outcome notification, verify all context fields and the exact session ID, then send it once to the configured `test` channel.
