## Why

Sending a richly formatted Discord notification from scripts should be a single portable CLI invocation, without requiring root access, a language runtime, or hand-built webhook payloads. The project currently has no application code, so this change establishes the Rust CLI, its user-facing configuration model, and a reproducible release path.

## What Changes

- Add a Rust CLI named `notify-me-on-discord`, with a `pingme` alias, that renders a named Markdown template and sends it through a Discord incoming webhook.
- Make `pingme 'message content'` the zero-ceremony path: it injects `message` into `templates/defaults.md` and delivers the result to the configured channel.
- Discover `config.toml` and `templates/` beside the executable by default, with explicit-path and XDG user-directory fallbacks for packaged installations.
- Use `templates/defaults.md` when no template is named; support YAML frontmatter for Discord message fields and MiniJinja-style variables in both frontmatter and Markdown content.
- Support direct webhook URLs as well as optional Bot-token provisioning that finds or creates an incoming webhook in a configured channel.
- Resolve send options consistently as explicit CLI arguments, then template frontmatter, then `[defaults]` settings; this includes channel, webhook username, avatar, thread, and TTS options.
- Support channel aliases from `[channels]`, multi-channel Bot-managed webhook provisioning, and per-channel credential caching.
- Support reusable avatar profiles plus one-off CLI avatars sourced from a remote/local image, a rendered emoji, centered text, or a glyph from a user-supplied font.
- Add configuration initialization, validation, dry-run, template listing, and avatar preview commands.
- Add a rootless installer that places release binaries in `~/.local/bin` by default while keeping the executable-adjacent portable layout usable.
- Add release automation and archives for the primary Linux GNU and musl targets, including x86_64 and ARM64, with best-effort macOS and Windows artifacts.
- Document configuration, secret handling, templates, avatar behavior, installation, and release usage.

## Capabilities

### New Capabilities

- `portable-configuration`: Configuration and template discovery, initialization, validation, and secure secret overrides.
- `markdown-message-templates`: Markdown templates with YAML frontmatter, variables, default selection, and Discord payload rendering.
- `discord-webhook-delivery`: Direct webhook execution and optional Bot-token-based webhook provisioning.
- `configurable-avatars`: Remote/local image, emoji, text, and font-glyph avatar rendering and delivery.
- `user-installation-and-releases`: Rootless user installation and cross-platform release artifact production.

### Modified Capabilities

None.

## Impact

- Introduces a Rust binary crate and dependencies for CLI parsing, HTTP, serialization, templating, frontmatter, image processing, and font rendering.
- Adds calls to Discord API v10 and, for emoji assets, an optional configurable remote emoji artwork provider with local caching.
- Adds project-local configuration examples, shell installation tooling, GitHub Actions workflows, tests, and end-user documentation.
- Stores provisioned webhook credentials only in a user-owned state file or beside the executable in portable mode; credentials are never printed in normal or dry-run output.
