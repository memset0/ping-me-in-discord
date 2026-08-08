# ping-me-in-discord

`ping-me-in-discord` is a Rust CLI project for sending Discord notifications. It renders Markdown templates into Discord webhook messages and retains two equivalent command-line entry points:

```console
pingme 'message content'
notify-me-on-discord 'message content'
```

By default, the text becomes the `message` variable for `templates/defaults.md` beside the binary and is sent to the configured Discord channel. The template, channel, username, and avatar can all be overridden for an individual invocation.

## Installation

Releases provide prebuilt binaries, so end users need neither Rust nor root access. Inspect the installation script before running it:

```console
curl --proto '=https' --tlsv1.2 -fsSL \
  https://raw.githubusercontent.com/memset0/ping-me-in-discord/master/install.sh \
  -o /tmp/ping-me-in-discord-install.sh
less /tmp/ping-me-in-discord-install.sh
sh /tmp/ping-me-in-discord-install.sh
```

The installer places both `notify-me-on-discord` and `pingme` in `~/.local/bin` by default. Set `DISCORD_NOTIFICATION_INSTALL_DIR` to choose another user-owned directory:

```console
DISCORD_NOTIFICATION_INSTALL_DIR="$HOME/bin" sh /tmp/ping-me-in-discord-install.sh
```

Add `~/.local/bin` to your shell's `PATH` if necessary. The installer does not modify shell configuration.

## Installing the Codex skills

The binary embeds the `$ping-me-send-message` and `$ping-me-report-agent-status` Codex skills. Installing them does not require cloning this repository, reading Discord configuration, or accessing the network.

To install them in the current project, first enter the project root:

```console
cd /path/to/your-project
pingme skills install --scope project
```

This writes `.codex/skills/ping-me-send-message` and `.codex/skills/ping-me-report-agent-status` below the current directory.

To install them globally for the current user:

```console
pingme skills install --scope global
```

When `CODEX_HOME` is non-empty, the global destination is `${CODEX_HOME}/skills`; otherwise it is `~/.codex/skills`. Either command can be run again to refresh the CLI-owned skill files. Identical files remain unchanged, outdated or locally modified owned files are restored from the current binary, and unrelated skill directories are left untouched.

Upgrading from an older binary migrates the former `$discord-notify` and `$discord-agent-notify` installation. The installer removes only the three files it owned in each legacy directory; any additional files remain untouched.

Restart or reopen Codex, or begin a new Codex session, after installation so the new skills are discovered. You can then invoke them in chat:

```text
$ping-me-send-message
$ping-me-report-agent-status
```

The current release installs Codex skills only; it does not yet generate Claude Code skill files.

## Initialization

Portable mode places `config.toml` and `templates/defaults.md` beside the binary:

```console
notify-me-on-discord init --portable
```

Standard user mode follows platform directory conventions. On Linux, configuration is written below `~/.config/discord-notification`, while runtime state and the emoji cache live below `~/.local/share/discord-notification`:

```console
notify-me-on-discord init
```

Configuration lookup uses this precedence:

1. `--config /path/to/config.toml`
2. `DISCORD_NOTIFICATION_CONFIG`
3. `config.toml` beside the binary
4. The platform user configuration directory

Inspect the selected path and validate configuration offline with:

```console
notify-me-on-discord config path
notify-me-on-discord config validate
pingme channels list --json
pingme avatar list --json
```

The final two commands expose only channel and avatar metadata that is safe for agents to select. They do not print Bot tokens, webhook URLs, or avatar source paths.

## Discord credentials

### Direct webhook URL

This is the simplest and least-privileged setup. A Discord incoming webhook URL already contains its webhook token, so no Bot token is required:

```toml
[discord]
webhook_url = "https://discord.com/api/webhooks/WEBHOOK_ID/WEBHOOK_TOKEN"
webhook_name = "Notify Me"
```

### Bot token with automatic webhook provisioning

You can instead configure a Bot token and map readable aliases to multiple channel IDs under `[channels]`. The Bot must have `MANAGE_WEBHOOKS` in each target channel. On first delivery, the CLI reuses a matching incoming webhook or creates one and caches its URL by channel.

```toml
[discord]
bot_token = "YOUR_BOT_TOKEN"
webhook_name = "Notify Me"

[channels]
alerts = "123456789012345678"
releases = "234567890123456789"

[defaults]
channel = "alerts"
```

Use the default channel, an alias, or a numeric channel ID:

```console
pingme 'default destination'
pingme 'release completed' --channel releases
pingme 'one-off destination' --channel 345678901234567890
```

An incoming webhook cannot redirect a message to an arbitrary channel at execution time. When `--channel`, frontmatter `channel`, or `[defaults].channel` selects a destination, the CLI uses the Bot to manage that channel's webhook. A single `discord.webhook_url` is suitable only for a fixed destination when every configuration layer omits a channel.

Environment variables are recommended for secrets and override file values:

```console
export DISCORD_NOTIFICATION_WEBHOOK_URL='https://discord.com/api/webhooks/...'
# or
export DISCORD_NOTIFICATION_BOT_TOKEN='...'
```

The CLI redacts secrets from normal output, dry-run output, and API errors.

## Templates

When no template is specified, `[defaults].template` initially selects `defaults`, which resolves to `templates/defaults.md`. A newly initialized default template contains:

```jinja
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`{% if runtime.codex_thread_id %}   🧵 `{{ runtime.codex_thread_id }}`{% endif %}**
{{ message }}
```

Ordinary shell invocations show the hostname and time. When Codex supplies `CODEX_THREAD_ID`, the current conversation ID is appended after the timestamp:

```console
pingme 'build completed'
```

The first line contains the CLI host's `user@hostname` and local time, and `build completed` follows immediately on the next line. The template body preserves Discord Markdown, so bold text, lists, links, and other Discord Markdown in the message are not escaped.

Every render receives a reserved `runtime` object:

- `runtime.user` and `runtime.hostname`: the current system identity, falling back to `unknown-user` and `unknown-host` when unavailable.
- `runtime.codex_thread_id`: the optional current Codex conversation ID from `CODEX_THREAD_ID`; it is normally `null` in an ordinary shell.
- `runtime.timestamp.local`: local machine time in `M/D HH:mm:ss` format.
- `runtime.timestamp.unix`: Unix seconds for the same captured instant.
- `runtime.timestamp.iso8601`: a UTC ISO 8601 representation of the same instant.

Callers cannot replace `runtime` through `--data` or `--var`; collisions fail before any network request. Hostnames and thread IDs may be internal metadata, so remove those fields from your own template when they should not be sent. The installer and non-forced initialization preserve an existing `templates/defaults.md`, which means upgrading users must merge the conditional thread fragment manually if they want it.

Templates can begin with YAML frontmatter:

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

Send a named template with variables:

```console
notify-me-on-discord send \
  --template deployment \
  --var project=API \
  --var version=v1.2.3 \
  --var summary=successful \
  --var actor=CI
```

Variables can also come from a JSON object. Repeated `--var` arguments have the highest precedence:

```console
notify-me-on-discord send --template deployment --data event.json --var actor=manual
```

Inspect templates and the final payload without delivery:

```console
notify-me-on-discord templates list
pingme 'preview only' --dry-run
```

Dry-run mode does not provision webhooks, update avatars, or contact Discord. Undefined template variables fail immediately. When `allowed_mentions` is absent, the CLI disables mention parsing so template content cannot unexpectedly trigger `@everyone`.

Send options use one precedence order:

```text
CLI argument > template frontmatter > config.toml [defaults] > unset
```

Common per-message overrides include:

```console
pingme 'deploy completed' \
  --channel releases \
  --username 'Deploy Bot' \
  --avatar rocket \
  --no-tts
```

The CLI also supports `--thread-id`, `--thread-name`, `--tts`, `--avatar-url`, and the one-off avatar arguments described below. Complex embeds, components, polls, and allowed mentions remain in template frontmatter.

See the [configuration reference](docs/configuration.md) for every frontmatter field, avatar type, and configuration option.

## Avatars

Define reusable profiles under `[avatars.<name>]` in `config.toml`. CLI `--avatar <name>`, template frontmatter `avatar`, and `[defaults].avatar` can all select a profile:

- `image`: an HTTPS image URL becomes the current message's `avatar_url`; a local image is center-cropped into a square PNG.
- `emoji`: transparent Twemoji artwork is downloaded, cached, and rendered over the configured background. Optional `foreground` recolors visible artwork while preserving its alpha outline and anti-aliasing. Omitted `scale` defaults to `0.72` and remains configurable per profile.
- `text`: a Chinese character, Latin letter, or short Unicode string is centered with configurable foreground and background colors.
- `font-icon`: a glyph from a user-supplied TTF, OTF, or TTC font is rendered through the same pipeline, including Font Awesome icons.

An optional `description` helps people and agents select a profile without affecting rendering:

```toml
[avatars.release]
description = "Use for successful releases"
type = "emoji"
emoji = "🚀"
background = "#5865F2"
```

New configurations also include the `started`, `progress`, `success`, `needs-input`, `warning`, and `error` agent status profiles. The strict notification skill passes only `--avatar <status>`; emoji, color, size, and scale remain entirely in `config.toml`. The `error` profile uses the accepted `scale = 0.576`. Upgrades never overwrite existing configuration, so existing users must manually copy these common profile blocks from the [complete example](examples/config.toml).

List the safe profile summary:

```console
pingme avatar list
pingme avatar list --json
```

Preview a local or generated avatar:

```console
notify-me-on-discord avatar preview rocket --output rocket.png
```

You can also define an avatar for one invocation:

```console
pingme 'rocket launched' --avatar-emoji '🚀' --avatar-background '#5865F2'
pingme 'verification failed' --avatar-emoji '❌' --avatar-foreground '#FFFFFF' --avatar-background '#DD2E44'
pingme 'build completed' --avatar-text 'N' --avatar-foreground '#FFFFFF' --avatar-background '#57F287'
pingme 'custom image' --avatar-file ./avatar.png --avatar-size 256
pingme 'remote image' --avatar-url https://example.com/avatar.png
```

Only one of `--avatar`, `--avatar-url`, `--avatar-file`, `--avatar-emoji`, `--avatar-text`, and `--avatar-icon` can be used at a time. Font icons additionally require `--avatar-font`. Style arguments include `--avatar-foreground`, `--avatar-background`, `--avatar-size`, `--avatar-font-size`, and `--avatar-scale`.

Discord requires `avatar_url` to be an HTTPS URL it can access, so remote images are sent as per-message avatar overrides. Local images, emoji, text, and font icons are rendered as PNG and assigned to dedicated incoming webhook identities keyed by target channel and image digest. Different generated avatars therefore do not repeatedly mutate one base webhook. This path requires a resolved channel and a Bot token with `MANAGE_WEBHOOKS` in that channel. Missing prerequisites cause an explicit failure instead of a silent fallback to the default avatar.

When every configuration layer omits an avatar, the CLI always uses the base webhook with Discord's default avatar. The first time an upgraded CLI encounters legacy state indicating that an older version modified a base webhook, it resets that avatar to `null` and clears the legacy state entry.

## Codex agent notification skills

The project provides two Codex skills with intentionally separate responsibilities:

- `$ping-me-send-message`: sends intentional free-form Discord Markdown after using JSON commands to select a configured channel and optional avatar profile. It is not used merely to report an agent lifecycle state.
- `$ping-me-report-agent-status`: reports exactly one of `started`, `progress`, `success`, `needs-input`, `warning`, or `error` and passes the same-named configured profile. It does not handle arbitrary messages or custom avatar selection, and it contains no emoji, color, or scale settings.

Both skills read `CODEX_THREAD_ID`, dry-run the message, and verify that the local template rendered the exact ID before performing the live send. Every CLI call runs through the skill's `scripts/run-pingme.sh`; after a wrapped command fails, the runner makes exactly one short error-report attempt and preserves the original exit status.

Error reporting can also be invoked directly:

```console
pingme report-error --channel alerts
```

This command bypasses templates and avatars and sends only `⚠️ Agent notification failed ...`. If the selected channel is unknown or delivery fails, it attempts a different `[defaults].channel` once. If configuration, credentials, or the network are unavailable, it fails locally without recursion.

Install these skills at project or global scope with the earlier `pingme skills install` commands. This initial release does not include a Claude Code version. Existing installations with an older `defaults.md` must manually adopt the `runtime.codex_thread_id` conditional fragment and merge the six status profiles into `config.toml`; upgrades preserve user templates and configuration.

## Development and releases

```console
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --release
```

A `vX.Y.Z` tag triggers the release workflow. Linux x86_64 and ARM64 musl archives are the primary artifacts, with additional GNU/Linux, macOS, and Windows builds when their CI targets succeed. Every archive has a SHA-256 checksum. See the [release documentation](docs/releases.md) for details.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for emoji artwork attribution.
