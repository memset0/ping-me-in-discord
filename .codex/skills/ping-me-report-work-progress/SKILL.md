---
name: ping-me-report-work-progress
description: Enable conversation-scoped Discord progress reporting through pingme. Explicit invocation activates this policy for the current conversation until the user explicitly disables it. Use started, progress, warning, or recoverable error only while the agent will continue working. Do not use for arbitrary messages or the final notification before a response or user-input wait; use ping-me-send-message or ping-me-report-turn-outcome instead.
---

# Ping Me: Report Work Progress

Report compact milestones while work continues. Explicit invocation activates this policy for this conversation and later turns; remember it until the user explicitly asks to stop progress notifications. Do not carry activation into another conversation.

## Activation state

On first activation, remember within the conversation:

- progress notifications are enabled;
- an explicitly requested channel, or default routing when none was requested;
- one concise kebab-case session name based on the conversation's task.

Reuse this state on later turns without requiring another invocation. If the user disables progress notifications, mark the policy inactive before continuing and do not send a final progress notification merely to acknowledge disablement.

## Status policy

Choose exactly one status only when the agent will continue working afterward.

| Status | Use when | Title emoji | Avatar profile |
|---|---|---|---|
| `started` | Beginning substantial requested work | 🚀 | `--avatar started` |
| `progress` | A meaningful stage completed and work continues | 🔄 | `--avatar progress` |
| `warning` | Work continues with a risk the user should know | ⚠️ | `--avatar warning` |
| `error` | A step failed but an in-scope recovery path remains | ❌ | `--avatar error` |

Never use `success` or `needs-input` here. Never use this skill for terminal `warning` or `error`; the yield boundary belongs to `ping-me-report-turn-outcome`.

## Message contract

```markdown
**<status emoji> <short progress title>**
<one- or two-sentence summary>
**Next:** <one concrete next action>
```

Include `Next:` because work continues. Exclude logs, stack traces, credentials, tokens, webhook URLs, and detailed diagnostics.

## Workflow

1. Resolve `scripts/run-pingme.sh` relative to this file and invoke it by absolute path for every `pingme` operation. Never invoke `pingme` directly.
2. Confirm `pingme` exists with `command -v pingme`. Named profiles require the active config to define all six shared status names and a Bot with `MANAGE_WEBHOOKS`; never retry without the selected profile.
3. Pass the remembered session name as `--session-name <name>` to every runner call. The runner infers agent and project; use `--agent-name` or `--project-name` only to correct its context.
4. Read the session ID with `/absolute/path/to/run-pingme.sh --session-name "$session_name" --print-session-id`. If unavailable, run `--report-only` for the remembered channel, skip this notification, and continue the user's task.
5. If a channel was explicitly selected, validate it with `channels list --json` through the runner. Accept an exact alias or explicit numeric ID. Otherwise omit `--channel`. Never run `avatar list`.
6. Choose one continuing-work state and add only its exact `--avatar <status>` selector.
7. Dry-run the exact invocation. For example:

   ```bash
   message=$(printf '%s\n%s\n%s' \
     '**🔄 Runtime metadata is implemented**' \
     'The new context fields and compatibility alias pass focused tests.' \
     '**Next:** Split and validate the notification skills.')
   /absolute/path/to/run-pingme.sh --session-name "$session_name" \
     --error-channel test -- --channel test --avatar progress --dry-run "$message"
   ```

8. Require the rendered `content` to contain the exact session ID. If absent, use `--report-only`, skip the notification, and continue work.
9. Repeat the invocation without `--dry-run` exactly once, then continue working.

## Failure rules

- The runner attempts one short `report-error` after a wrapped CLI failure and preserves the original status.
- Missing profiles and semantic preflight failures never trigger a one-off avatar fallback or retry.
- A notification failure does not end the user's task. Surface a concise local note at the next natural user update when useful.
- Never include the original diagnostic in Discord failure content and never recurse.
