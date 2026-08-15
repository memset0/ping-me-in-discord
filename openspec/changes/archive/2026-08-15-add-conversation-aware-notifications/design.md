## Context

See `proposal.md` for motivation. The CLI currently serializes user, host, timestamp, and an optional Codex-specific environment value. Both bundled skills use identical shell runners, while the status skill owns all six lifecycle states. Skill installation embeds the repository's `.codex/skills` files and copies framework-appropriate regular files for Codex and Claude Code.

Skill activation cannot be made mechanically persistent across model turns by a Markdown skill alone. The available agent frameworks do, however, retain the conversation and prior instructions, so a skill can establish an explicit conversation-scoped policy without adding Herdr, hooks, or machine state.

## Goals / Non-Goals

**Goals:**

- Make the lifecycle boundary unambiguous: progress means the agent keeps working; outcome means the agent is about to yield.
- Keep all framework-neutral skill instructions and runner behavior in one canonical repository source.
- Give template authors a stable agent-neutral runtime schema while retaining existing templates.
- Make the current conversation recognizable in Discord with concise, normalized metadata.
- Safely migrate files owned by the retired skill name for both supported installation adapters.

**Non-Goals:**

- Guaranteed notification enforcement by an external supervisor.
- Cross-conversation activation or a persistent notification toggle.
- Automatic Herdr injection, agent hooks, or edits to agent-framework configuration.
- Moving avatar artwork or status selection policy out of the existing configured profiles.

## Decisions

### 1. Split status reporting at the yield boundary

`ping-me-report-work-progress` owns `started`, `progress`, `warning`, and recoverable `error` messages that are followed by more agent work. `ping-me-report-turn-outcome` owns exactly one of `success`, `needs-input`, `warning`, or terminal `error` immediately before a final response or blocking question. The overlap for `warning` and `error` is intentional: the deciding property is whether work continues, not the icon.

Both skills select only `--avatar <status>`. The six profiles remain one shared visual vocabulary in `config.toml`; skills never copy colors, emoji, dimensions, or scale. `ping-me-send-message` remains available only for intentional free-form content.

Alternative considered: keep one six-state skill and add prose about timing. This leaves the invocation name and trigger description ambiguous, which is the issue this change resolves.

### 2. Represent activation as an explicit conversation policy

Each automatic skill tells the agent that explicit invocation enables its policy for every later turn in the same conversation until the user explicitly disables it. On first activation, the agent chooses a concise session name based on the conversation's task and reuses that name. The skill descriptions repeat this trigger-critical rule so agents can retain it after the initial skill read.

This is documented as best-effort conversation memory. A new conversation begins disabled. There is no hidden file, hook, or global preference whose lifetime could surprise the user.

Alternative considered: write an activation marker into a local config file. That would unintentionally cross conversations and machines and would not reliably tell an agent to act before every response.

### 3. Add an agent-neutral runtime schema with compatibility aliasing

The serialized shape becomes:

```text
runtime.agent.name
runtime.project.name
runtime.session.id
runtime.session.name
runtime.user
runtime.hostname
runtime.timestamp.local|unix|iso8601
runtime.codex_thread_id
```

The last field remains an optional alias of `runtime.session.id`, regardless of the producing adapter, so existing templates continue rendering unchanged. New `PINGME_AGENT_NAME`, `PINGME_PROJECT_NAME`, `PINGME_SESSION_NAME`, and `PINGME_SESSION_ID` environment variables are the explicit generic inputs. Session-ID fallback order is generic, Claude Code, then Codex.

When explicit values are absent, Rust runtime capture supplies safe direct-CLI defaults: adapter inference or `CLI`, current-directory basename or `unknown-project`, and a deterministic `session-<first-eight-ID-characters>` name or `interactive`. Existing inline-code normalization is applied to every new value.

Alternative considered: expose only top-level variables. Keeping all automatic values below the reserved `runtime` object prevents collisions with user data and preserves the current security boundary.

### 4. Let the shared runner enrich agent context

The identical runner copies accept `--agent-name`, `--project-name`, and `--session-name` before `--`. They export normalized intent through the generic `PINGME_*` inputs, select the current session ID once, and continue exporting `CODEX_THREAD_ID` only for compatibility with older binaries. If project name is omitted, the runner tries the canonical `origin` remote basename, then Git root basename, then working-directory basename. If session name is omitted, it uses the deterministic ID-prefix fallback. Agent detection follows the selected Claude Code or Codex environment.

The skill passes its remembered human-readable session name on every discovery, dry-run, and delivery call. The dry-run still verifies the exact session ID before the one live send. Existing single-attempt failure reporting remains unchanged.

Alternative considered: require users to configure all metadata in TOML. These values describe a process and conversation, not a Discord destination, and would become stale when reused.

### 5. Use a fixed two-line starter header

The starter template always renders agent/project/session name on line one and host/time/optional session ID on line two, followed immediately by content. Direct CLI defaults keep the layout valid without pretending an agent conversation exists. Existing local templates are never rewritten by an ordinary upgrade.

### 6. Migrate only installer-owned retired paths

The bundle changes from two skills to three. Codex receives nine files and Claude Code receives six. During refresh, the installer removes the three exact owned files under the retired Codex `ping-me-report-agent-status` directory and the two shared files under its Claude Code directory. It preserves unknown files and removes a directory only when empty. Older `discord-*` cleanup remains Codex-only.

## Risks / Trade-offs

- **[Conversation policy is not mechanically enforced]** → State the best-effort scope prominently in both skill descriptions and README; make activation and disable rules explicit.
- **[Agent environments expose different identifiers]** → Keep generic override inputs, support both known adapter variables, and fail normal skill delivery when no session ID exists.
- **[Project discovery could expose an unexpected directory label]** → Emit only a normalized basename, never a full path or credential-bearing remote URL, and permit an explicit runner override.
- **[Expanded headers reduce the 2,000-character content budget]** → Keep both lines compact and continue validating the fully rendered Discord content before delivery.
- **[Retired-name cleanup could encounter user additions]** → Remove only exact installer-owned files and preserve nonempty directories.

## Migration Plan

1. Add the generic runtime fields and starter template while retaining `runtime.codex_thread_id`.
2. Scaffold and author the renamed progress skill and new outcome skill, then update free-form boundary guidance and all identical runner copies.
3. Update embedded assets and adapter-specific legacy cleanup.
4. Update tests and English README/configuration reference.
5. Validate locally, install into isolated Codex and Claude Code destinations, then dry-run and send one outcome notification for the current session to the configured test channel.
6. Sync delta specs, archive the change, commit the exact change paths, and push `master`.

Rollback consists of restoring the prior binary and reinstalling the older two-skill bundle. Existing local templates and avatar profiles remain usable because their files and compatibility runtime key are preserved.
