# configurable-avatars Specification

## Purpose

Define reusable avatar profiles that turn URLs, local images, emoji, text, and font glyphs into Discord webhook identities.

## Requirements

### Requirement: Templates select named avatar profiles
Configuration SHALL define named avatar profiles under `[avatars.<name>]`. CLI `--avatar`, template frontmatter `avatar`, and `[defaults].avatar` SHALL select those profiles using the standard precedence order.

#### Scenario: Template-specific avatar
- **WHEN** a template frontmatter selects `release` and configuration defines that profile
- **THEN** the release profile is used for that message

#### Scenario: Unknown avatar
- **WHEN** a template selects a profile that does not exist
- **THEN** the CLI fails before sending

### Requirement: CLI supports one-off avatar sources
The CLI SHALL accept mutually exclusive one-off avatar sources for an HTTPS URL, local file, emoji, text, or font glyph. Type-appropriate `--avatar-*` modifiers SHALL configure colors, font, dimensions, and emoji scale. Selecting any CLI avatar source SHALL atomically override avatar choices from frontmatter and settings; omitted modifiers SHALL use documented built-in defaults rather than fields from a lower-layer avatar.

#### Scenario: One-off emoji avatar
- **WHEN** the user supplies `--avatar-emoji 🚀 --avatar-background '#5865F2'`
- **THEN** the CLI renders that one-off avatar without requiring a named profile

#### Scenario: Profile selected from CLI
- **WHEN** the user supplies `--avatar release` and configuration defines `[avatars.release]`
- **THEN** the configured profile is selected ahead of frontmatter and settings

#### Scenario: Conflicting avatar sources
- **WHEN** the user supplies both `--avatar release` and `--avatar-text R`
- **THEN** argument parsing fails before configuration or network access

### Requirement: Remote and local images are supported
An image avatar profile SHALL accept either an HTTPS URL or a local image path relative to the configuration file. Remote URLs SHALL be passed as per-message `avatar_url`; local images SHALL be decoded, center-cropped to a square, resized, and encoded as PNG.

#### Scenario: Remote image URL
- **WHEN** an image profile contains an HTTPS URL
- **THEN** the rendered webhook payload uses that URL without downloading or modifying the image

#### Scenario: Local image
- **WHEN** an image profile references a readable local JPEG, PNG, GIF, or WebP
- **THEN** the CLI produces a square PNG suitable for a Discord webhook avatar

### Requirement: Emoji avatars have configurable backgrounds
An emoji avatar profile SHALL accept one Unicode emoji, a background color, and optional sizing. The CLI SHALL resolve transparent emoji artwork, cache it in the user data directory, center it on the configured background, and render a square PNG.

#### Scenario: Render an emoji avatar
- **WHEN** an emoji profile specifies `🚀` and background `#5865F2`
- **THEN** the preview and webhook avatar contain centered rocket artwork over that background

#### Scenario: Emoji asset is cached
- **WHEN** the same emoji is rendered after its artwork has been cached
- **THEN** the CLI uses the cached asset without another provider request

### Requirement: Text avatars support Unicode and colors
A text avatar profile SHALL accept a short Unicode string, foreground color, background color, and optional font path and size. The CLI SHALL center the text both horizontally and vertically and SHALL use a configured or system font that contains the requested glyphs.

#### Scenario: Chinese character avatar
- **WHEN** a text profile specifies `告` and an available font containing that glyph
- **THEN** the CLI renders the centered character using the configured foreground and background colors

#### Scenario: Missing glyph support
- **WHEN** no configured or discoverable font contains all requested characters
- **THEN** the CLI fails with guidance to configure a compatible font file

### Requirement: Font-icon avatars use user-provided fonts
A font-icon avatar profile SHALL accept a glyph or Unicode code point and a font file, plus foreground and background colors. It SHALL render through the same centered raster pipeline as text avatars.

#### Scenario: Render a notification icon
- **WHEN** a profile supplies a Font Awesome font and its bell glyph
- **THEN** the CLI renders that glyph as the avatar

### Requirement: Locally rendered avatars can be used by Discord
Before executing a message with a local or generated avatar, the CLI SHALL update the incoming webhook's default avatar using PNG image data. It SHALL cache the applied image digest to avoid redundant updates and SHALL not send the local path or data URI as `avatar_url`.

#### Scenario: Apply a generated avatar
- **WHEN** a generated avatar differs from the last locally applied avatar
- **THEN** the CLI updates the webhook avatar and then executes the message without an `avatar_url` override

#### Scenario: Avatar is unchanged
- **WHEN** the generated avatar digest matches the state recorded for that webhook
- **THEN** the CLI skips the avatar update and executes the message

### Requirement: Omitted avatars use Discord's default
When CLI arguments, template frontmatter, and `[defaults]` all omit an avatar, the outgoing payload SHALL omit `avatar_url` and use the webhook's default Discord avatar. If the CLI previously applied a local or generated avatar to that webhook, it SHALL reset the webhook avatar to `null` once before sending and clear the recorded digest.

#### Scenario: No avatar was ever applied
- **WHEN** no layer selects an avatar and state has no applied avatar digest for the webhook
- **THEN** the CLI sends without an avatar override or avatar modification request

#### Scenario: Generated avatar must be reset
- **WHEN** no layer selects an avatar and state records a generated avatar previously applied by the CLI
- **THEN** the CLI resets the webhook avatar, removes the digest, and then sends using Discord's default avatar

### Requirement: Avatars can be previewed
The CLI SHALL provide a command that renders a named local or generated avatar to a user-selected PNG path without contacting Discord.

#### Scenario: Preview a text avatar
- **WHEN** the user previews a configured text profile
- **THEN** the command writes its PNG and reports the output path
