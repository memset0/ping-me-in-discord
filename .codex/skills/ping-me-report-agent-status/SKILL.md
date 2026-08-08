---
name: ping-me-report-agent-status
description: Report one structured Codex agent lifecycle status through the local pingme CLI using fixed started, progress, success, needs-input, warning, or error presentation, same-named configured avatar profiles, the selected or default Discord channel, and the current Codex thread ID. Use only when an agent must report one of those lifecycle states with the strict compact format and bounded failure reporting. Do not use for arbitrary Discord content or custom avatar selection; use ping-me-send-message instead.
---

# Ping Me: Report Agent Status

Report exactly one compact, policy-driven agent lifecycle status. Select the fixed same-named configured avatar profile without querying or reproducing its visual settings. For arbitrary messages or custom avatar choices, stop and use `ping-me-send-message` instead.

## Status policy

Choose exactly one status and pass its profile selector exactly as shown.

| Status | Use when | Title emoji | Avatar profile |
|---|---|---|---|
| `started` | Beginning requested work | 🚀 | `--avatar started` |
| `progress` | A meaningful stage completed and work continues | 🔄 | `--avatar progress` |
| `success` | Requested work completed successfully | ✅ | `--avatar success` |
| `needs-input` | Work requires a user decision or missing input | ❓ | `--avatar needs-input` |
| `warning` | Work continues but the user should know a risk | ⚠️ | `--avatar warning` |
| `error` | Work or a required verification failed | ❌ | `--avatar error` |

## Message contract

Format only these lines:

```markdown
**<status emoji> <short imperative or outcome title>**
<one- or two-sentence summary>
**Next:** <one concrete action>
```

Omit the `Next:` line when no action is useful. Do not add headings, tables, blank sections, stack traces, logs, tokens, webhook URLs, or detailed diagnostics. Keep the whole body comfortably below Discord's limit because the template prepends runtime metadata.

## Workflow

1. Resolve `scripts/run-pingme.sh` relative to this `SKILL.md` and invoke it by absolute path for every `pingme` operation. Never invoke `pingme` directly.
2. Confirm `pingme` is installed with `command -v pingme`. The named generated-avatar profiles require the local configuration to define all six status names and route through a Bot with `MANAGE_WEBHOOKS`; never retry without the selected profile if resolution or provisioning fails.
3. Read the current conversation ID with `printenv CODEX_THREAD_ID`. If empty, use the runner's `--report-only` mode and stop. Do not pass it as Discord's unrelated `--thread-id` option.
4. If the user specified a channel, run `channels list --json` through the runner. Accept an exact alias from the result or an explicitly supplied numeric ID. An unknown nonnumeric selector is a preflight failure. If no channel was requested, omit `--channel` and use the configured default. Never run `avatar list` for this skill.
5. Choose one status, create content that satisfies the message contract, and add only the exact `--avatar <status>` selector from the table. Never add one-off avatar source or styling arguments.
6. Dry-run the exact invocation through the runner. Example for a successful test-channel notification:

   ```bash
   message=$(printf '%s\n%s' \
     '**✅ Initial skill is ready**' \
     'The Codex notification workflow passed local verification.')
   /absolute/path/to/run-pingme.sh --error-channel test -- \
     --channel test --avatar success --dry-run \
     "$message"
   ```

7. Inspect the rendered JSON and require its `content` to contain the exact `CODEX_THREAD_ID`. If absent, use `--report-only` for the selected channel and stop.
8. Repeat the same invocation without `--dry-run` exactly once. Report the returned Discord message ID to the user.

## Failure rules

- The runner sends one short `report-error` message after any wrapped CLI failure and returns the original failure status.
- Treat an absent selected status profile as a normal wrapped failure. Never synthesize a one-off avatar fallback.
- Use `--report-only` for semantic preflight failures. Do not retry a normal notification automatically.
- Never include the original diagnostic in the Discord failure message and never recurse when reporting itself fails.
