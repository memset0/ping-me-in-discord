## Context

The CLI already discovers one user-owned TOML file, resolves channel aliases, renders strict MiniJinja Markdown templates, and supports configured or one-off avatars. The full configuration contains credentials and local paths, so an agent must not inspect it merely to discover safe selectors. Templates are intentionally user-managed and upgrades never overwrite them. See `proposal.md` and the four delta specs for the requested behavior.

Live acceptance showed that the existing local-avatar path is not a message identity: it patches one base webhook's persistent avatar and immediately executes through that same identity. Although Discord records changing avatar hashes, clients can present the messages with the default or stale avatar. Execute Webhook accepts only HTTP(S) `avatar_url` values, so an in-memory PNG cannot become a reliable per-message override without external hosting.

Codex discovers project skills below `.codex/skills/`. Each skill must be useful in isolation and must treat Discord delivery as an external side effect: normal messages are sent only when the user asks or durable project instructions require them.

## Goals / Non-Goals

**Goals:**

- Give agents stable, secret-safe JSON discovery surfaces for channels and configured avatar profiles.
- Supply one flexible and one policy-driven Codex skill with deterministic failure handling.
- Include the current Codex conversation in agent notifications without overloading Discord's existing `thread_id` delivery field.
- Make failure reports independent of user templates and avatar rendering, with one bounded fallback attempt.
- Make each generated strict status avatar a stable, visibly distinct Discord identity without mutating the base webhook.
- Keep strict status visual settings in user-owned configuration while the skill selects only stable profile names.

**Non-Goals:**

- Claude Code packaging, implicit periodic notifications, and automatic notification timing policy.
- Replacing user templates during an upgrade or silently editing a user's configuration.
- Guaranteeing Discord error delivery when configuration, credentials, Discord, or the network itself is unavailable.
- Making the strict status-to-profile-name mapping configurable in this initial version.
- Supporting local or generated avatars in direct-webhook-only configurations that lack a Bot token and resolved channel.

## Decisions

### Add narrow inspection commands instead of teaching agents to read TOML

`pingme channels list [--json]` will return aliases, IDs, and an optional resolved default. Its JSON shape will be an object containing `channels` entries and a nullable `default` object. `pingme avatar list [--json]` will return a `profiles` array containing only `name`, `type`, nullable `description`, and `is_default`.

Human output will remain concise, while the JSON form is the contract used by the skills. Neither path serializes the complete configuration. This is preferable to parsing `config.toml`, which would unnecessarily expose Bot tokens, webhook URLs, and local file paths to agent context.

### Wrap configured avatars so descriptions remain metadata

Configuration will deserialize each profile into an `AvatarProfile` containing an optional description and a flattened existing `AvatarConfig`. Rendering continues to receive only the inner avatar value, so the metadata cannot alter image generation or precedence. The enum will expose a small type-name accessor for inspection output.

This is preferable to repeating `description` in every tagged enum variant and keeps one-off CLI avatars unchanged.

### Model Codex identity as optional runtime metadata

`RuntimeMetadata` will add `codex_thread_id: Option<String>`, populated from `CODEX_THREAD_ID` using the same single-line inline-code normalization applied to user and hostname values. It is deliberately named `codex_thread_id` so agents do not confuse it with the Discord destination option `--thread-id`.

The starter default template will conditionally append `🧵 \`{{ runtime.codex_thread_id }}\`` on the existing metadata line. A null value keeps the old two-field output. User templates are not rewritten; both skills perform a dry-run and verify that the exact current ID appears before making the real send.

### Implement failure delivery as a dedicated non-recursive command

`pingme report-error [--channel <selector>]` will bypass templates and avatars and construct a fixed payload with safe mention parsing:

```text
⚠️ Agent notification failed for thread `<id>`.
```

When no ID is available, the suffix is omitted. The command loads configuration once and produces at most two distinct destination candidates:

1. the requested selector, if it resolves;
2. the configured default destination, if it differs from the first candidate.

An unknown requested alias skips directly to the default. A failed delivery to the requested destination advances once to the default. No requested selector uses the default directly. The command reuses the existing webhook provisioning and state store, but never invokes itself and never includes the original error text in Discord content. If all candidates fail, it returns a combined local error.

A report cannot be delivered when configuration cannot be loaded or all credentials/network paths fail. Returning this clearly is safer than claiming an impossible guarantee or creating an unbounded retry loop.

### Bundle the same small safe runner with each skill

Each skill will contain its own `scripts/run-pingme.sh` so it remains self-contained. The runner accepts an optional error channel followed by `pingme` arguments, executes the command without suppressing its output, captures the original status, calls `pingme report-error` once on failure, and finally returns the original status. It refuses to wrap `report-error` itself.

Duplicating this small script is intentional: a globally installed strict skill must not depend on the simple skill's installation path. Both copies will be kept byte-identical and syntax-tested.

### Keep the simple and strict selection policies separate

`discord-notify` uses both JSON listing commands. It honors explicit user choices, otherwise uses the resolved default channel and selects a configured avatar only when its name or description clearly matches; ambiguity means no avatar argument and therefore Discord's default avatar.

`discord-agent-notify` may inspect channels but never lists avatar profiles. Its six status mappings expand only into `--avatar started`, `--avatar progress`, `--avatar success`, `--avatar needs-input`, `--avatar warning`, or `--avatar error`. It never copies emoji, source, color, size, scale, font, or path values into the skill. It formats exactly a bold status title, a short summary, and an optional bold `Next:` line.

The starter configuration defines all six same-named emoji profiles as ordinary `[avatars.<name>]` entries. This makes `config.toml` the single source of truth and allows users to adjust presentation without editing the skill. The first five profiles use scale `0.72`; visual comparison selected scale `0.576` for `error`. Existing configuration remains user-owned and is never upgraded implicitly, so current users must add the six profiles explicitly before using the strict skill.

Both skills first require a non-empty `CODEX_THREAD_ID`, dry-run the final send through the safe runner, check that the rendered content contains that exact ID, and only then perform the live invocation. A missing ID or a template that omits it becomes a bounded failure-report event rather than a degraded normal message.

### Recolor emoji artwork through its alpha mask

Emoji profiles and one-off emoji arguments will accept an optional foreground color. When present, the renderer replaces every visible source pixel's RGB channels with that color and multiplies its alpha by the foreground alpha, preserving the source silhouette and anti-aliased edges before compositing it over the configured background. Omitting the foreground leaves cached artwork byte-for-byte unchanged before resizing and compositing.

The config-owned strict status palette uses dominant opaque colors measured from the cached Twemoji artwork: progress blue `#3B88C3`, success green `#77B255`, and error red `#DD2E44`. Started uses a white background. The `error` profile additionally supplies white as the foreground and scale `0.576`, producing the accepted smaller white ❌ silhouette on its native red. The skill sees only the profile name.

### Isolate each generated avatar behind a stable webhook identity

Remote image profiles remain message-level `avatar_url` overrides. For a local image, emoji, text, or font icon, the CLI renders the PNG and computes its existing SHA-256 digest, then selects an identity cache entry keyed by resolved channel ID and digest. A cache miss creates a Bot-managed incoming webhook whose persistent avatar is initialized in the creation request; a cache hit reuses that webhook without another avatar update. Different digests therefore have different webhook IDs, avoiding mutation and client cache collisions on one shared identity.

Generated-identity webhook names include a digest suffix for internal discovery. Before execution, the CLI supplies the configured base webhook name as `username` only when no higher-precedence username is already present, so the implementation detail is not exposed in the message presentation. The private state file stores the additional secret webhook URLs alongside existing channel webhook state.

This requires a resolved channel plus `discord.bot_token` and `MANAGE_WEBHOOKS`. The CLI will reject a generated avatar when those are unavailable instead of retaining the unreliable shared-mutation fallback. Remote `avatar_url` profiles and avatar-less delivery continue to work through the base webhook. On first use of a base webhook recorded in the legacy `avatar_digests` map, the CLI resets its persistent avatar once and removes that record before selecting any identity, restoring the expected Discord default for later avatar-less sends.

### Generate and validate standard Codex skill metadata

Both directories will be scaffolded with the system `skill-creator` initializer, then reduced to the required `SKILL.md`, `agents/openai.yaml`, and safe runner. Descriptions will contain the complete triggering conditions, while bodies remain procedural and concise. The standard validator and shell syntax check will gate completion.

## Risks / Trade-offs

- **[Existing user templates omit the new field]** → Skills verify dry-run output and stop with a short error report; documentation provides the conditional header snippet, while upgrades preserve user ownership.
- **[The primary send may have reached Discord before a response failure]** → A later warning can be a duplicate signal; the bounded reporter favors visibility over attempting unsafe delivery inference.
- **[Requested and default selectors can resolve to the same ID]** → Resolve and deduplicate candidates before network access so the reporter never retries the same destination.
- **[A malformed config prevents both work and reporting]** → Return a local nonzero diagnostic; no implementation can recover a destination or credential safely from an unreadable config.
- **[Status visuals may not match every user's preferences]** → Keep every source and style field in named config profiles so users can customize them without changing the skill's stable status mapping.
- **[Existing configurations lack the six strict profile names]** → Never overwrite them; document the profile blocks, fail visibly through the bounded wrapper when one is absent, and update this development machine's private configuration explicitly.
- **[Foreground recoloring can erase multicolor emoji detail]** → Make it explicitly optional and use it only when a silhouette treatment is intended; unchanged configurations retain the source artwork colors.
- **[Skill copies of the runner can drift]** → Compare them byte-for-byte in verification in addition to validating each skill.
- **[Generated identities consume channel webhook slots]** → Reuse one identity per channel and image digest, never delete unrecognized webhooks, and surface Discord provisioning limits as actionable errors; the fixed strict policy needs only six reusable identities.
- **[A cached generated-identity webhook is deleted externally]** → Treat the resulting Discord failure as a normal bounded notification failure; a later change can add automatic stale-entry recovery without weakening failure visibility.

## Migration Plan

1. Add optional avatar descriptions and inspection commands; existing TOML remains valid.
2. Add the generated-identity state map while retaining the legacy digest map for deserialization and one-time base-avatar restoration.
3. Add optional runtime Codex metadata and update only embedded/example starter templates.
4. Add the independent error reporter, six starter status avatar profiles, and skills that select those profiles by name.
5. Document the conditional template line and generated-avatar provisioning requirements; do not overwrite user files automatically.
6. For this development machine only, update the external user template separately and run all six strict states in the `test` channel, confirming distinct webhook identities and non-default avatar hashes.
7. Refine the four affected strict status avatars from measured Twemoji colors, select 80% of the standard scale (`0.576`) for `error`, and migrate all visual values out of the strict skill into config-owned profiles before archiving.

Rollback removes the new commands and skills and restores the prior starter template. Existing `description` fields would then be rejected by older binaries because configuration denies unknown fields, so users who adopted descriptions must remove them before downgrading. Generated-identity webhooks already created in Discord are left intact rather than deleting external resources implicitly.
