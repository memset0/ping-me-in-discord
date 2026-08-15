---
name: ping-me-report-turn-outcome
description: Enable one structured Discord outcome notification immediately before every agent yield in the current conversation. Explicit invocation activates this policy for later turns until the user explicitly disables it. Use success, needs-input, warning, or terminal error according to the turn result. Do not use during continuing work or for arbitrary content; use ping-me-report-work-progress or ping-me-send-message instead.
---

# Ping Me: Report Turn Outcome

Send exactly one compact outcome immediately before each final response or blocking request for user input. Explicit invocation activates this policy for this conversation and later turns; remember it until the user explicitly asks to stop outcome notifications. Do not carry activation into another conversation.

## Activation state

On first activation, remember within the conversation:

- outcome notifications are enabled for every yield;
- an explicitly requested channel, or default routing when none was requested;
- one concise kebab-case session name based on the conversation's task.

Reuse this state on later turns without another invocation. If the user disables outcome notifications, mark the policy inactive and yield without sending one. This is a conversation-memory policy, not a hook or machine-persistent setting.

## Outcome policy

Choose exactly one status at the yield boundary.

| Status | Use when | Title emoji | Avatar profile |
|---|---|---|---|
| `success` | Requested work completed and required verification passed | ✅ | `--avatar success` |
| `needs-input` | A user decision or missing input prevents continuation | ❓ | `--avatar needs-input` |
| `warning` | The turn produced useful work with an important limitation | ⚠️ | `--avatar warning` |
| `error` | Requested work or required verification terminated unsuccessfully | ❌ | `--avatar error` |

`started`, `progress`, and recoverable problems belong to `ping-me-report-work-progress` because the agent continues afterward.

## Message contract

```markdown
**<status emoji> <short outcome title>**
<one- or two-sentence summary>
**Next:** <one concrete user action>
```

Use `Next:` only for `needs-input` or when another concrete user action is useful. Exclude logs, stack traces, credentials, tokens, webhook URLs, and detailed diagnostics.

## Workflow

1. Before every final response or blocking question while active, reserve exactly one outcome notification. Do not send it earlier if more work or tool calls remain.
2. Resolve `scripts/run-pingme.sh` relative to this file and invoke it by absolute path for every `pingme` operation. Never invoke `pingme` directly.
3. Confirm `pingme` exists with `command -v pingme`. Named profiles require the active config to define all six shared status names and a Bot with `MANAGE_WEBHOOKS`; never retry without the selected profile.
4. Pass the remembered session name as `--session-name <name>` to every runner call. The runner infers agent and project; use `--agent-name` or `--project-name` only to correct its context.
5. Read the session ID with `/absolute/path/to/run-pingme.sh --session-name "$session_name" --print-session-id`. If unavailable, run `--report-only` for the remembered channel, then return the user-facing response anyway.
6. If a channel was explicitly selected, validate it with `channels list --json` through the runner. Accept an exact alias or explicit numeric ID. Otherwise omit `--channel`. Never run `avatar list`.
7. Choose one outcome and add only its exact `--avatar <status>` selector.
8. Dry-run the exact invocation. For example:

   ```bash
   message=$(printf '%s\n%s' \
     '**✅ Conversation notifications are ready**' \
     'The split workflows and agent context passed verification.')
   /absolute/path/to/run-pingme.sh --session-name "$session_name" \
     --error-channel test -- --channel test --avatar success --dry-run "$message"
   ```

9. Require rendered `content` to contain the exact session ID. If absent, use `--report-only` and continue to the final response.
10. Repeat the invocation without `--dry-run` exactly once. Whether it succeeds or fails, immediately return the already-prepared user-facing response; do not send a second outcome.

## Failure rules

- The runner attempts one short `report-error` after a wrapped CLI failure and preserves the original status.
- Missing profiles and semantic preflight failures never trigger a one-off avatar fallback or retry.
- Notification failure must never suppress, replace, or materially delay the user-facing response.
- Never include the original diagnostic in Discord failure content and never recurse.
