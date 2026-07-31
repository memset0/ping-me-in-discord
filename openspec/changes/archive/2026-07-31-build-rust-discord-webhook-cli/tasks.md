## 1. Rust project foundation

- [x] 1.1 Create the Rust crate, shared library, `notify-me-on-discord` and `pingme` binary entry points, and repository hygiene files
- [x] 1.2 Define the Clap command model, including the top-level message shorthand and structured subcommands
- [x] 1.3 Add pure-Rust TLS, serialization, templating, image, font, hashing, file-locking, and test dependencies with a committed lockfile

## 2. Portable configuration and state

- [x] 2.1 Implement explicit, environment, executable-adjacent, and platform-user configuration discovery in the specified precedence order
- [x] 2.2 Define and validate TOML configuration for Discord, templates, defaults, emoji assets, and named avatar profiles
- [x] 2.3 Implement user-data state/cache paths, atomic state persistence, Unix owner-only permissions, and secret redaction
- [x] 2.4 Implement portable and user-layout initialization without overwriting existing files
- [x] 2.5 Implement configuration path inspection and offline validation commands

## 3. Markdown templates and CLI send flow

- [x] 3.1 Implement safe template discovery, `defaults.md` selection, and template listing
- [x] 3.2 Implement strict MiniJinja rendering from JSON data and repeatable variable overrides
- [x] 3.3 Parse YAML frontmatter and Markdown bodies into validated Discord payloads with safe mention defaults
- [x] 3.4 Implement `pingme 'message content'`, the equivalent long-name invocation, named-template send, and dry-run JSON output
- [x] 3.5 Add unit and end-to-end tests for template paths, rendering, frontmatter, limits, shorthand invocation, and no-network dry runs

## 4. Discord webhook integration

- [x] 4.1 Implement secret-safe webhook URL parsing and deterministic credential selection
- [x] 4.2 Implement Bot-authenticated webhook listing/creation, matching, permission diagnostics, and cached provisioning state
- [x] 4.3 Implement token-authenticated webhook avatar modification and confirmed webhook execution
- [x] 4.4 Implement bounded Discord rate-limit retries and actionable API error reporting
- [x] 4.5 Add mock HTTP tests for direct delivery, Bot provisioning, avatar modification, retries, and credential redaction

## 5. Avatar rendering

- [x] 5.1 Implement color parsing, square image crop/resize, PNG encoding, and remote-image passthrough
- [x] 5.2 Implement emoji codepoint resolution, HTTPS artwork download/cache, and background compositing
- [x] 5.3 Implement configured/system font discovery with glyph checks and centered Unicode text rasterization
- [x] 5.4 Implement font-icon glyph parsing and rasterization through the text pipeline
- [x] 5.5 Implement avatar profile selection, applied-digest caching, update/send locking, and PNG preview output
- [x] 5.6 Add deterministic tests for image, emoji, text, font-icon, profile selection, and digest behavior

## 6. Installation, releases, and documentation

- [x] 6.1 Add a no-root POSIX installer for both entry points with platform detection, temporary downloads, checksum verification, and atomic replacement
- [x] 6.2 Add CI quality checks and tagged multi-target release packaging for Linux, macOS, and Windows
- [x] 6.3 Write configuration, template/frontmatter, Bot/webhook, avatar, security, rootless installation, and release documentation with working starter examples
- [x] 6.4 Add third-party artwork attribution and license metadata
- [x] 6.5 Run formatting, strict linting, tests, representative release builds, installer checks, and strict OpenSpec validation

## 7. Layered send options and multi-channel routing

- [x] 7.1 Add typed CLI/frontmatter/settings precedence, channel aliases, and validation for all agreed send arguments
- [x] 7.2 Provision and cache Bot-managed webhooks independently for each resolved channel while keeping unrouted direct webhooks safe
- [x] 7.3 Add named-profile selection, mutually exclusive one-off URL, file, emoji, text, and font-icon avatar arguments, and restoration of Discord's default avatar when no avatar is selected
- [x] 7.4 Update starter configuration, examples, user documentation, and the local portable installation without overwriting credentials
- [x] 7.5 Add precedence, alias, multi-channel, and avatar argument tests, then rerun formatting, MSRV, strict linting, release, installer, and OpenSpec validation
