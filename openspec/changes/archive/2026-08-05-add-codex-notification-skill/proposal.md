## Why

Codex agents can already invoke `pingme`, but they lack discoverable, safe workflows for selecting destinations and identities, attaching the current conversation identifier, or reporting a failed notification without going silent. A simple skill plus a policy-driven skill make agent notifications repeatable without exposing the full secret-bearing configuration. Live acceptance also exposed that locally rendered avatars mutate one shared webhook and can appear as Discord's default avatar, so strict status identities need isolated delivery.

## What Changes

- Add a simple `discord-notify` Codex skill that teaches agents how to inspect available channels and avatar profiles, obtain `CODEX_THREAD_ID`, and send a freely formatted Discord notification.
- Add a strict `discord-agent-notify` Codex skill with built-in `started`, `progress`, `success`, `needs-input`, `warning`, and `error` notification types. Each type selects the same-named avatar profile from `config.toml` without inspecting or duplicating its visual settings. Starter configuration owns the fixed status palette, and the `error` profile renders a white cross on its native red at 80% of the standard emoji scale.
- **BREAKING** Replace shared-webhook avatar mutation for local and generated images with Bot-provisioned webhook identities cached by resolved channel and image digest. Generated avatars now require a routed channel, a Bot token, and permission to manage webhooks; the CLI fails instead of silently showing a default avatar when those prerequisites are absent.
- Require strict notifications to use a short bold title, a one- or two-sentence summary, and an optional `Next:` action without logs, stack traces, or secrets.
- Add secret-safe channel and avatar profile listing commands with JSON output suitable for agents.
- Allow avatar profiles to carry an optional description so an agent can select a semantically appropriate preset.
- Add six starter status avatar profiles whose source, colors, dimensions, and scale remain user-configurable in `config.toml`; existing user configuration is documented and migrated explicitly rather than overwritten.
- Expose the current Codex thread ID as optional runtime template metadata and append it to the starter message header when available.
- Add a template-independent, single-attempt failure reporting command and bundled safe wrappers. A report targets the requested channel when usable, falls back to the configured default for an unknown or unreachable requested channel, and never recursively reports its own failure.
- Keep Claude Code support and advanced notification policy outside this initial version.

## Capabilities

### New Capabilities

- `agent-notification-skill`: Simple and strict Codex skills for sending notifications through the installed CLI, including fixed status presentation and bounded failure reporting.

### Modified Capabilities

- `portable-configuration`: Add secret-safe discovery of configured channel aliases and the effective default destination.
- `configurable-avatars`: Add optional profile descriptions and a machine-readable avatar profile listing command, provide config-owned status profiles, allow shape-preserving foreground recoloring of emoji artwork, and isolate locally rendered avatars into stable per-channel webhook identities.
- `markdown-message-templates`: Add optional Codex thread metadata and render it in the starter metadata header when present.

## Impact

- Affects CLI command parsing, inspection and failure-report output, configuration deserialization, emoji rasterization, runtime template context, Discord webhook provisioning and local state, starter/example configuration and templates, tests, and user documentation.
- Adds two repository-local Codex skills under `.codex/skills/`, each with standard UI metadata and a deterministic safe-execution script.
- Does not change Discord credentials, send precedence, or existing user-managed template files during upgrades.
