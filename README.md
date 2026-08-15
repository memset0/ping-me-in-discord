# ping-me-in-discord

`ping-me-in-discord` is a rootless Rust CLI for sending templated Discord notifications. Releases install two equivalent executable names:

```console
pingme 'message content'
ping-me-in-discord 'message content'
```

This guide uses the shorter `pingme` spelling from here on. Messages preserve Discord Markdown and can select a channel, template, webhook username, and avatar for each send.

## Setup

### 1. Install the CLI

Releases provide prebuilt binaries, so end users need neither Rust nor root access. Inspect the installer before running it:

```console
curl --proto '=https' --tlsv1.2 -fsSL \
  https://raw.githubusercontent.com/memset0/ping-me-in-discord/master/install.sh \
  -o /tmp/ping-me-in-discord-install.sh
less /tmp/ping-me-in-discord-install.sh
sh /tmp/ping-me-in-discord-install.sh
```

Both executable names are installed in `~/.local/bin` by default. Add that directory to `PATH` if necessary; the installer does not edit shell configuration. To use another user-owned directory:

```console
DISCORD_NOTIFICATION_INSTALL_DIR="$HOME/bin" sh /tmp/ping-me-in-discord-install.sh
```

Upgrades preserve neighboring configuration and templates. They remove the exact legacy `notify-me-on-discord` executable only after both current entry points are installed successfully. See [Releases](docs/releases.md) for supported targets, checksums, and manual installation.

### 2. Create and configure `config.toml`

Standard initialization follows platform directory conventions. On Linux it creates configuration under `~/.config/discord-notification` and keeps runtime state and the emoji cache under `~/.local/share/discord-notification`:

```console
pingme init
pingme config path
```

For a portable installation, place `config.toml` and `templates/defaults.md` beside the executable instead:

```console
pingme init --portable
```

Choose one Discord credential model in `config.toml`:

```toml
# Simplest option: one fixed Discord destination.
[discord]
webhook_url = "https://discord.com/api/webhooks/WEBHOOK_ID/WEBHOOK_TOKEN"
webhook_name = "Ping Me"

[defaults]
template = "defaults"
```

Or use a Bot token when messages must route to multiple channels or use generated local avatars. The Bot needs `MANAGE_WEBHOOKS` in every target channel:

```toml
[discord]
bot_token = "YOUR_BOT_TOKEN"
webhook_name = "Ping Me"

[channels]
default = "123456789012345678"
test = "234567890123456789"

[defaults]
channel = "default"
template = "defaults"
```

Environment variables are recommended for secrets and override file values:

```console
export DISCORD_NOTIFICATION_WEBHOOK_URL='https://discord.com/api/webhooks/...'
# or
export DISCORD_NOTIFICATION_BOT_TOKEN='...'
```

Configuration lookup order is an explicit `--config` path, `DISCORD_NOTIFICATION_CONFIG`, `config.toml` beside the executable, then the platform user configuration directory. Validate the selected configuration without contacting Discord:

```console
pingme config validate
pingme channels list --json
pingme avatar list --json
```

The two JSON listings expose safe selection metadata only; they do not reveal credentials or local avatar source paths. See the [configuration reference](docs/configuration.md) and [complete example](examples/config.toml) for all settings.

### 3. Optionally install the agent skills

The three bundled skills are agent-framework-neutral and maintained from one canonical source:

- `ping-me-send-message` sends one intentional free-form message.
- `ping-me-report-work-progress` reports milestones while the agent keeps working.
- `ping-me-report-turn-outcome` sends one result immediately before each response or user-input wait.

The CLI currently provides installation adapters for Codex and Claude Code.

Install into the current project:

```console
pingme skills install --scope project --agent codex
pingme skills install --scope project --agent claude-code
```

Or install for the current user:

```console
pingme skills install --scope global --agent codex
pingme skills install --scope global --agent claude-code
```

| Adapter | Project directory | Global directory | Typical invocation |
| --- | --- | --- | --- |
| Codex | `.codex/skills` | `${CODEX_HOME}/skills` or `~/.codex/skills` | `$ping-me-send-message` |
| Claude Code | `.claude/skills` | `${CLAUDE_CONFIG_DIR}/skills` or `~/.claude/skills` | `/ping-me-send-message` |

`--agent claude` is an alias for `claude-code`; omitting `--agent` retains the historical Codex default. Installation copies independent regular files and never creates symbolic links. Re-run the same command to refresh CLI-owned files without touching unrelated skills, then restart or reopen the agent if it does not discover them immediately. Refreshes remove only installer-owned files under the retired `ping-me-report-agent-status` name; Codex also cleans up installer-owned files under the older `discord-notify` and `discord-agent-notify` names.

## Send messages

The shortest command sends through the default template and destination:

```console
pingme 'build completed'
```

Use a configured channel alias or a numeric Discord channel ID:

```console
pingme 'release completed' --channel releases
pingme 'one-off destination' --channel 345678901234567890
```

Common per-message overrides are:

```console
pingme 'deploy completed' \
  --channel releases \
  --username 'Deploy Bot' \
  --avatar rocket \
  --no-tts
```

Send fields use one precedence order:

```text
CLI argument > template frontmatter > config.toml [defaults] > unset
```

The CLI also supports `--thread-id`, `--thread-name`, `--tts`, `--avatar-url`, and the one-off avatar options below. If every layer omits an avatar, Discord's default webhook avatar is used.

## Templates

`[defaults].template = "defaults"` selects `templates/defaults.md`. It may instead contain an absolute path ending in `.md`. A new default template places available host, project, session, and agent context in one line, followed by the local time and the message without an extra blank line. The thread segment prefers the session title and falls back to the full session ID:

```jinja
> **{% if runtime.hostname != "unknown-host" %}🏠 `{% if runtime.user != "unknown-user" %}{{ runtime.user }}@{% endif %}{{ runtime.hostname }}`   {% endif %}{% if runtime.project.name != "unknown-project" %}📦 `{{ runtime.project.name }}`   {% endif %}{% if runtime.session.title %}🧵 `{{ runtime.session.title }}`   {% elif runtime.session.id %}🧵 `{{ runtime.session.id }}`   {% endif %}{% if runtime.agent.name != "CLI" %}🤖 `{{ runtime.agent.name }}`   {% endif %}📅 `{{ runtime.timestamp.local }}`**
{{ message }}
```

The template body preserves Discord Markdown. `runtime.session.title` is optional and comes from `PINGME_SESSION_NAME`; the required `runtime.session.name` remains available to older custom templates with `session-<ID prefix>` or `interactive` fallbacks. The reserved `runtime` object also provides `runtime.timestamp.unix`, `runtime.timestamp.iso8601`, and `runtime.codex_thread_id` as a compatibility alias for `runtime.session.id`. Direct CLI use falls back to agent `CLI` and the current directory name, while agent runners supply richer values through `PINGME_AGENT_NAME`, `PINGME_PROJECT_NAME`, `PINGME_SESSION_NAME`, and `PINGME_SESSION_ID`. The starter header omits unavailable context and its direct-CLI agent fallback. Hostnames, project names, and session identifiers may be internal metadata, so remove fields from custom templates when they should not be sent.

Templates can use YAML frontmatter for message metadata and Discord payload fields:

```markdown
---
username: "{{ project }} Deploy"
avatar: rocket
embeds:
  - title: "Release {{ version }}"
    description: "{{ summary }}"
    color: "#5865F2"
---
Triggered by **{{ actor }}**.
```

Render a named template with command-line or JSON variables:

```console
pingme send --template deployment --var project=API --var version=v1.2.3
pingme send --template deployment --data event.json --var actor=manual
pingme send --template /absolute/path/custom.md 'one-off template'
pingme templates list
pingme 'preview only' --dry-run
```

A simple template name stays inside the configured templates directory and receives the `.md` extension automatically. An absolute `.md` path is read exactly as supplied, including when it is outside that directory. Relative paths such as `./custom.md` and `../custom.md` remain invalid, and `templates list` lists only named templates in the configured directory.

Dry-run mode does not contact Discord or provision webhooks. Undefined variables fail before delivery, and mention parsing is disabled by default unless a template explicitly configures `allowed_mentions`. Complex embeds, components, polls, and allowed mentions belong in frontmatter; the [configuration reference](docs/configuration.md) lists every supported field.

## Avatars

Named profiles in `[avatars.<name>]` keep presentation policy in `config.toml`. Profiles can use a local image, emoji, centered text, or a font glyph. Generated avatars support foreground and background colors, dimensions, font settings, and an independent `scale` value whose default is `0.72`.

New configurations include `started`, `progress`, `success`, `needs-input`, `warning`, and `error` profiles shared by the automatic notification skills. The skills select only a profile name; artwork, colors, and scale stay in configuration. Upgrades preserve existing configuration, so copy desired profiles from the [complete example](examples/config.toml).

Inspect or preview profiles:

```console
pingme avatar list
pingme avatar preview rocket --output rocket.png
```

Define a one-off avatar for one message:

```console
pingme 'rocket launched' --avatar-emoji '🚀' --avatar-background '#5865F2'
pingme 'verification failed' --avatar-emoji '❌' --avatar-foreground '#FFFFFF' --avatar-background '#DD2E44'
pingme 'build completed' --avatar-text 'N' --avatar-foreground '#FFFFFF' --avatar-background '#57F287'
pingme 'custom image' --avatar-file ./avatar.png --avatar-size 256
pingme 'remote image' --avatar-url https://example.com/avatar.png
```

Only one avatar source may be selected at a time. Font icons additionally require `--avatar-font`; styling options include `--avatar-foreground`, `--avatar-background`, `--avatar-size`, `--avatar-font-size`, and `--avatar-scale`.

Discord must be able to fetch a remote `avatar_url`. Local and generated avatars require a resolved channel plus a Bot token with `MANAGE_WEBHOOKS`; the CLI assigns each image digest a dedicated incoming webhook identity instead of repeatedly mutating one base webhook. Missing prerequisites fail explicitly rather than silently falling back to the default avatar.

## Agent notification skills

The boundaries are based on intent and whether the agent continues working:

| Skill | Use it for | Status profiles |
| --- | --- | --- |
| `ping-me-send-message` | One user-requested free-form Discord message | Any configured profile or Discord default |
| `ping-me-report-work-progress` | Optional updates followed by more work | `started`, `progress`, `warning`, recoverable `error` |
| `ping-me-report-turn-outcome` | Exactly one update immediately before yielding | `success`, `needs-input`, `warning`, terminal `error` |

Explicitly invoking either automatic reporting skill enables its policy for later turns in the same conversation. It remains active until the user explicitly asks to stop that policy; activation does not carry into a new conversation and does not use hooks, Herdr, or machine-persistent state. The agent chooses one concise session name on activation and reuses it throughout the conversation.

The common runner obtains the active session ID, infers agent and project context, dry-runs the template, verifies that the exact ID was rendered, and then sends once. Session-ID precedence is `PINGME_SESSION_ID`, `CLAUDE_CODE_SESSION_ID`, then `CODEX_THREAD_ID`. Explicit runner context can override agent, project, or session name without changing `config.toml`.

Every skill CLI call goes through the bounded runner. If a wrapped call fails, it makes one short error-report attempt and preserves the original exit status. The report uses the requested channel when possible, falls back to `[defaults].channel` when the requested alias is unknown or delivery fails, and never recurses:

```console
pingme report-error --channel alerts
```

Because the skill source and behavior are not tied to a particular agent product, another compatible framework can use the same files when it supports the skill format and exports `PINGME_SESSION_ID`. The built-in adapters listed in [Setup](#3-optionally-install-the-agent-skills) handle framework-specific directories and invocation syntax.

## Development and releases

```console
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --release
```

A `vX.Y.Z` tag triggers the release workflow. Linux x86_64 and ARM64 musl archives are the primary artifacts, with additional GNU/Linux, macOS, and Windows builds when their CI targets succeed. Every archive has a SHA-256 checksum. See [Releases](docs/releases.md) for details.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for emoji artwork attribution.
