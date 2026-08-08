---
name: ping-me-send-message
description: Send an intentional free-form Discord message through the local pingme CLI with configured channel aliases, optional configured avatar profiles, and the current coding-agent session ID. Use when the user asks the agent to send, ping, or forward arbitrary content to Discord, or when a project explicitly requires a free-form Discord message. Do not use for a structured started, progress, success, needs-input, warning, or error agent lifecycle report; use ping-me-report-agent-status instead.
---

# Ping Me: Send Message

Send one intentional free-form Discord message without reading the secret-bearing TOML file. Do not use this workflow merely to report an agent lifecycle state.

## Workflow

1. Resolve `scripts/run-pingme.sh` relative to this `SKILL.md` and invoke it by absolute path for every `pingme` operation. Never invoke `pingme` directly.
2. Confirm `pingme` is installed with `command -v pingme`. Local, emoji, text, and font-icon avatars require a routed channel and a Bot with `MANAGE_WEBHOOKS`; never remove a requested avatar to work around a provisioning failure.
3. Read the current coding-agent session ID through the runner: `agent_session_id=$(/absolute/path/to/run-pingme.sh --print-session-id)`. If that command fails or returns an empty value, use the runner's `--report-only` mode, then stop. Never substitute Discord's `--thread-id`; that option targets a Discord thread.
4. Run the following discovery commands through the runner before choosing values:

   ```bash
   /absolute/path/to/run-pingme.sh -- channels list --json
   /absolute/path/to/run-pingme.sh -- avatar list --json
   ```

5. Select the destination:

   - Honor an explicit user channel. Accept an exact listed alias or an explicitly supplied numeric Discord channel ID.
   - If an explicit nonnumeric channel is not listed, report the failure once and stop.
   - If the user did not choose a channel, omit `--channel`; the CLI will use `[defaults].channel`.

6. Select the avatar:

   - Honor an explicit profile only when `avatar list --json` contains its exact name.
   - Otherwise choose a profile only when its name or description clearly matches the message purpose.
   - On ambiguity or no suitable profile, omit every avatar argument so Discord uses its default avatar.

7. Compose concise free-form Discord Markdown. Exclude credentials, webhook URLs, raw logs, and other secrets.
8. Dry-run the exact send through the runner. When a channel was explicitly selected, pass it to the runner as `--error-channel <channel>` as well as to `pingme`:

   ```bash
   /absolute/path/to/run-pingme.sh --error-channel test -- \
     --channel test --avatar release --dry-run "message content"
   ```

9. Inspect the rendered JSON and require its `content` to contain the exact `agent_session_id`. If it does not, use `--report-only` for the selected channel and stop; the user's existing template needs the coding-agent session field.
10. Repeat the same invocation without `--dry-run` exactly once. Report the returned Discord message ID to the user.

## Failure rules

- The runner automatically calls `pingme report-error` once after any wrapped CLI failure and preserves the original exit status.
- Use `--report-only` for preflight failures such as a missing coding-agent session ID or a rendered template that omits it.
- Do not retry a failed normal message automatically. Do not wrap or directly call `report-error` yourself.
- If the error report also fails, surface the local diagnostic; the runner never recurses.
