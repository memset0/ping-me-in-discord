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

The bundled skills are agent-framework-neutral: one canonical source teaches an agent how to send free-form messages or structured lifecycle reports. The CLI currently provides installation adapters for Codex and Claude Code.

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

`--agent claude` is an alias for `claude-code`; omitting `--agent` retains the historical Codex default. Installation copies independent regular files and never creates symbolic links. Re-run the same command to refresh CLI-owned files without touching unrelated skills, then restart the agent if it does not discover them immediately. Codex refreshes also clean up only the files owned by the former `discord-notify` and `discord-agent-notify` skill names.

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

`[defaults].template = "defaults"` selects `templates/defaults.md`. A new default template places host, local time, and an optional agent session ID above the message without an extra blank line:

```jinja
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`{% if runtime.codex_thread_id %}   🧵 `{{ runtime.codex_thread_id }}`{% endif %}**
{{ message }}
```

The template body preserves Discord Markdown. The reserved `runtime` object also provides Unix and ISO 8601 timestamps. Hostnames and session IDs may be internal metadata, so remove those fields from custom templates when they should not be sent.

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
pingme templates list
pingme 'preview only' --dry-run
```

Dry-run mode does not contact Discord or provision webhooks. Undefined variables fail before delivery, and mention parsing is disabled by default unless a template explicitly configures `allowed_mentions`. Complex embeds, components, polls, and allowed mentions belong in frontmatter; the [configuration reference](docs/configuration.md) lists every supported field.

## Avatars

Named profiles in `[avatars.<name>]` keep presentation policy in `config.toml`. Profiles can use a local image, emoji, centered text, or a font glyph. Generated avatars support foreground and background colors, dimensions, font settings, and an independent `scale` value whose default is `0.72`.

New configurations include `started`, `progress`, `success`, `needs-input`, `warning`, and `error` profiles for agent status reports. The skills select only a profile name; artwork, colors, and scale stay in configuration. Upgrades preserve existing configuration, so copy desired profiles from the [complete example](examples/config.toml).

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

The two skills deliberately have different responsibilities:

- `ping-me-send-message` sends intentional free-form Discord Markdown and can select any configured channel and optional avatar profile.
- `ping-me-report-agent-status` reports exactly one of `started`, `progress`, `success`, `needs-input`, `warning`, or `error` with the same-named configured profile.

The common runner obtains the active coding-agent session ID from the environment, dry-runs the template, verifies that the ID was rendered, and then sends the live notification. It currently recognizes `CLAUDE_CODE_SESSION_ID` and `CODEX_THREAD_ID`, normalizing either into the template's compatibility field `runtime.codex_thread_id`.

Every skill CLI call goes through the bounded runner. If a wrapped call fails, it makes one short error-report attempt and preserves the original exit status. The report uses the requested channel when possible, falls back to `[defaults].channel` when the requested alias is unknown or delivery fails, and never recurses:

```console
pingme report-error --channel alerts
```

Because the skill source and behavior are not tied to a particular agent product, another compatible framework can use the same installed files when it supports the skill format and provides one of the recognized session identifiers. The built-in destination adapters listed in [Setup](#3-optionally-install-the-agent-skills) handle the framework-specific directory layout and invocation syntax.

## Development and releases

```console
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --release
```

A `vX.Y.Z` tag triggers the release workflow. Linux x86_64 and ARM64 musl archives are the primary artifacts, with additional GNU/Linux, macOS, and Windows builds when their CI targets succeed. Every archive has a SHA-256 checksum. See [Releases](docs/releases.md) for details.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for emoji artwork attribution.
