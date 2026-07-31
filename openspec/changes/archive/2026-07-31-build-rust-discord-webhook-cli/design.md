## Context

The repository currently contains only OpenSpec setup and a README. The application must be a standalone Rust CLI, work primarily on Linux without root privileges, discover `config.toml` and `templates/defaults.md` beside its own executable, and integrate with Discord's HTTP API.

Discord incoming webhooks execute with a URL containing a webhook ID and secret token; they do not require Bot authentication. Bot authentication is useful only to list or create channel webhooks and requires `MANAGE_WEBHOOKS`. Discord accepts `avatar_url` per execution, but a generated local PNG is not a public URL. The token-authenticated modify-webhook endpoint accepts image data, so generated avatars must update the webhook's persistent default avatar before message execution.

The XDG Base Directory specification identifies `~/.local/bin` for user executables, `~/.config` for configuration, and `~/.local/share` for data. The portable sibling layout has higher discovery priority because it is an explicit product requirement; XDG remains the safe installed fallback and state/cache location.

## Goals / Non-Goals

**Goals:**

- Keep the common send path to one process invocation and one Markdown file.
- Make every operation testable without real Discord credentials.
- Keep secrets out of logs and dry-run output.
- Produce binaries that do not require OpenSSL or other uncommon runtime libraries.
- Make local-avatar behavior explicit and deterministic despite Discord requiring a reachable URL or persistent webhook avatar.

**Non-Goals:**

- Maintaining a Discord Gateway connection or responding to Discord events.
- Managing multiple guilds, OAuth installation, slash commands, or general Bot administration.
- Hosting generated avatars on a public image service.
- Full fidelity rendering of arbitrary HTML/Markdown into images.
- Interactive message components for non-application-owned webhooks.

## Decisions

### Use a modular Rust core with two binary entry points and pure-Rust networking

The crate will expose shared application logic from a library and compile two tiny entry points: `notify-me-on-discord` and `pingme`. Both parse the same `clap` interface. A top-level positional message is syntactic sugar for sending `defaults.md` with a built-in `message` variable, so `pingme 'message content'` is the primary happy path.

The implementation will use `tokio` plus `reqwest` with rustls for HTTP, `serde` for TOML/YAML/JSON, and `anyhow`/`thiserror`-style context for diagnostics. Modules will separate CLI orchestration, path/config handling, templates, Discord transport, avatar rendering, and persisted state. Pure-rust TLS avoids an OpenSSL runtime dependency and makes musl release targets practical.

Alternative: a shell/Python client would be smaller to implement, but violates the standalone-binary and cross-system requirements.

### Treat executable-adjacent files as portable configuration

Discovery order is explicit path, environment path, executable sibling, then platform configuration directory. The template directory defaults relative to the selected config, not the working directory. `init --portable` writes beside the executable; ordinary `init` writes to the platform config directory. Persistent webhook state and emoji cache remain in the platform data directory so normal installs do not pollute `~/.local/bin`.

Alternative: only XDG discovery is more conventional but does not satisfy the requested copy-the-directory portability. Searching the current directory is intentionally excluded because scheduled jobs frequently have surprising working directories.

### Render the entire Markdown file before parsing frontmatter

Templates use strict MiniJinja syntax. JSON input is merged with repeatable `--var` values, then the complete file is rendered so variables work consistently in YAML and Markdown. The renderer splits optional `---` YAML frontmatter, converts supported fields into a JSON webhook payload, removes local fields such as `avatar` and `thread_id`, and uses the remaining Markdown as `content`. Default `allowed_mentions.parse` is empty.

Alternative: a strongly typed bespoke frontmatter schema gives better compile-time modeling but unnecessarily blocks new Discord payload fields. A whitelist plus JSON validation retains flexibility while catching typos and unsafe fields.

### Resolve send options through typed layers

Simple delivery and identity fields resolve through the same typed precedence pipeline: explicit CLI arguments, rendered template frontmatter, then `[defaults]`. Channel selectors are either numeric Discord IDs or exact aliases from `[channels]`. CLI flags use kebab-case while frontmatter and TOML use the corresponding snake_case names. Avatar selection is atomic: a CLI profile or one-off source replaces the entire lower-layer avatar instead of combining unrelated source and styling fields.

Secrets are excluded from send flags even though other settings have CLI equivalents, because command-line tokens and webhook URLs leak through shell history and process inspection. Complex Discord structures such as embeds, components, polls, and allowed mentions remain in frontmatter rather than becoming fragmented command-line JSON arguments.

### Support direct webhook URLs and lazy Bot provisioning

Direct webhook configuration is the simplest and preferred path. When absent, the transport reads cached state; when no cached URL exists, it lists channel webhooks with `Authorization: Bot ...`, reuses a matching incoming webhook, or creates one. The returned URL is stored with owner-only permissions. Send always executes with `wait=true`, parses the returned message ID, and performs bounded 429 retries.

The API base remains fixed to Discord API v10 in production. Tests inject an internal endpoint into the transport rather than exposing a config option that could exfiltrate Bot tokens.

Alternative: always require a Bot token contradicts Discord's simpler webhook authorization model and retains a more powerful secret than many users need.

For multi-channel routing, cached provisioned URLs are keyed by resolved channel ID. A selected channel always uses its channel-specific cache or Bot provisioning; a direct URL is used only when routing is left unset because Discord does not accept an arbitrary destination channel during webhook execution. This prevents `--channel` from appearing to work while silently delivering to the direct webhook's bound channel.

### Split avatar delivery into URL override and persistent image update

Remote image avatars become `avatar_url` on the execute request. Local images are center-cropped and resized with the `image` crate. Emoji profiles download transparent Twemoji PNG artwork from a configurable, HTTPS-only asset base and cache it before compositing. Text and font-icon profiles rasterize a configured or system-discovered TTF/OTF face with `fontdue`/`fontdb`, then center the glyph mask over the requested background.

Generated/local PNGs are base64 image data sent to Modify Webhook with Token. A SHA-256 digest keyed by webhook ID prevents redundant updates. A local file lock spans update plus execute to avoid races between processes sharing state on one machine.

No avatar is the default state: the execute payload omits `avatar_url`. Because a prior generated avatar persists on the webhook, state also records whether this CLI applied one; a later avatar-less send resets the webhook avatar to `null` before execution and removes that digest. It does not reset webhooks that the CLI has never modified.

Alternative: uploading the PNG as a message attachment cannot make it the avatar of that same webhook execution. Hosting images would add a service and privacy burden. Persistent mutation is therefore the only self-contained Discord-supported route, but its limitations are documented.

### Publish archives and a small rootless installer

Release CI builds and tests on native runners, uses `cross` for Linux x86_64/ARM64 GNU and musl targets, packages both binary entry points plus documentation per archive, emits SHA-256 files, and uploads assets for `v*` tags. The POSIX installer detects platform/architecture, downloads to a temporary directory, verifies the checksum, and atomically installs both entry points into `${DISCORD_NOTIFICATION_INSTALL_DIR:-$HOME/.local/bin}`.

This mirrors current standalone Rust-tool behavior while following the XDG recommendation for user executables. A custom installer is preferred over cargo-dist because the required default is `~/.local/bin` rather than Cargo's bin directory and the portable companion-file behavior is application-specific.

### Test boundaries instead of live Discord

Unit tests cover path precedence, parsing, strict variables, limits, color/codepoint handling, crop/raster helpers, and installer target mapping. HTTP tests use a local mock server to assert Bot authorization, provisioning, webhook modification, execution, retry, and redaction behavior. An end-to-end CLI test uses temporary config/templates and dry-run mode. CI never requires real credentials.

## Risks / Trade-offs

- **[Persistent avatar mutation can race with another host using the same webhook]** → Serialize locally, document that generated avatars persist, and recommend distinct webhooks for concurrent identities; remote URL avatars do not have this limitation.
- **[System font coverage varies, especially for CJK]** → Discover fonts by glyph coverage and provide an explicit `font` setting with actionable errors; do not silently render replacement boxes.
- **[Emoji rendering depends on a third-party asset provider]** → Cache successful assets, allow a compatible HTTPS base URL, attribute the artwork provider, and fail clearly when uncached artwork cannot be fetched.
- **[Webhook URLs and cached tokens are high-value secrets]** → Prefer environment overrides, use owner-only state files, redact URLs, and never include credentials in examples or test fixtures.
- **[Cross-platform font discovery and file permissions differ]** → Keep Unix permission hardening conditional and test path/config logic on Linux, macOS, and Windows CI.
- **[Release workflow cannot be fully exercised before the first tag]** → Validate workflow syntax and build representative native/musl targets in CI; retain `cargo install --path .` as a developer fallback.

## Migration Plan

1. Land the new crate, examples, tests, installer, and CI without changing any existing application behavior because no application exists.
2. Create an initial tagged prerelease and verify Linux x86_64 and ARM64 assets plus checksum installation.
3. Promote the release after a real Discord smoke test with both a direct webhook and Bot provisioning.
4. Roll back by removing the installed binary; configuration/templates and OpenSpec history remain user-owned and untouched.
