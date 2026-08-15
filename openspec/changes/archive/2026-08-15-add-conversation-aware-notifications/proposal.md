## Why

The existing agent-status skill mixes notifications sent while work is continuing with notifications sent immediately before the agent yields to the user, so agents cannot enable a clear, conversation-wide notification policy. The default template also exposes only host, time, and a Codex-specific thread identifier, which is not enough to identify the agent, project, and session that produced a notification.

## What Changes

- **BREAKING** Rename `ping-me-report-agent-status` to `ping-me-report-work-progress` and limit it to status updates sent while the agent continues working.
- Add `ping-me-report-turn-outcome`, which sends exactly one outcome notification immediately before each final response or request for user input.
- Make explicit activation of either automatic notification skill persist as a best-effort policy for later turns in the same conversation until the user explicitly disables it; do not add Herdr integration, hooks, or machine-persistent activation state.
- Keep `ping-me-send-message` as the explicit free-form messaging skill and give all three skills non-overlapping selection guidance.
- Give both automatic notification skills the same configured six-profile vocabulary while assigning each skill only the statuses appropriate to its lifecycle boundary.
- Add agent-neutral runtime metadata for agent name, project name, session name, and session ID; preserve `runtime.codex_thread_id` as a compatibility alias for the generic session ID.
- Expand the starter message template to render a compact two-line context header containing agent, project, session name, host, time, and session ID before the message.
- Bundle, install, migrate, document, and test the three-skill set for both Codex and Claude Code.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-notification-skill`: Split continuing-work and turn-outcome reporting, define conversation-scoped activation, shared profiles, runtime context, and bounded delivery behavior.
- `agent-skill-installation`: Install the renamed progress skill and new turn-outcome skill from one canonical source, and remove only files owned by the retired skill name.
- `markdown-message-templates`: Expose generic agent/project/session runtime metadata and use it in the starter template while retaining the legacy session-ID alias.

## Impact

This affects the bundled skill source, agent adapters, safe runner, skill installation and migration logic, default template/runtime serialization, configuration documentation, README, and their Rust and integration tests. Existing user templates continue to work through the `runtime.codex_thread_id` compatibility alias, while users invoking the retired skill name must adopt `ping-me-report-work-progress`.
