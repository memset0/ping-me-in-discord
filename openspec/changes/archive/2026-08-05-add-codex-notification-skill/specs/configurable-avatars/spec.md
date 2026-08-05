## ADDED Requirements

### Requirement: Avatar profiles describe their intended use
Each `[avatars.<name>]` profile SHALL accept an optional `description` string for human and agent selection. A configured description SHALL be non-empty after trimming and no longer than 200 Unicode characters. It SHALL not affect rendering or send precedence.

#### Scenario: Profile contains selection guidance
- **WHEN** an avatar profile declares `description = "Use for completed tasks"`
- **THEN** configuration validation accepts it and avatar rendering behaves as it did without the description

#### Scenario: Empty profile description
- **WHEN** an avatar profile declares only whitespace as its description
- **THEN** configuration validation fails before delivery

### Requirement: Avatar profiles can be inspected safely
The CLI SHALL provide `avatar list` with human-readable output and a `--json` form for agents. Each entry SHALL contain only the profile name, avatar type, optional description, and whether settings select it by default. It SHALL NOT disclose local source paths, remote avatar URLs, font paths, or unrelated configuration values.

#### Scenario: Agent lists avatar profiles
- **WHEN** configuration contains described text and emoji profiles
- **THEN** `pingme avatar list --json` returns their names, types, descriptions, and default-selection flags

#### Scenario: No profiles are configured
- **WHEN** the avatar profile map is empty
- **THEN** the listing succeeds with an empty collection

### Requirement: Starter configuration provides agent status avatar profiles
Newly initialized configuration SHALL define ordinary user-editable emoji profiles named `started`, `progress`, `success`, `needs-input`, `warning`, and `error`. The profiles SHALL use square size `256`. `started` SHALL render 🚀 on `#FFFFFF`; `progress` SHALL render 🔄 on `#3B88C3`; `success` SHALL render ✅ on `#77B255`; `needs-input` SHALL render ❓ on `#F1C40F`; and `warning` SHALL render ⚠️ on `#E67E22`, each at scale `0.72`. `error` SHALL render ❌ recolored `#FFFFFF` on `#DD2E44` at scale `0.576`. These visual settings SHALL live in configuration rather than the strict skill, and upgrades SHALL NOT overwrite existing user-owned configuration.

#### Scenario: New configuration contains all strict profiles
- **WHEN** a user initializes a new configuration
- **THEN** all six status profile names resolve through `--avatar <status>` with their documented visual settings

#### Scenario: Existing profile remains user-controlled
- **WHEN** an existing user changes a status profile's source, color, size, or scale and later upgrades the binary
- **THEN** the user's configuration remains unchanged and the strict skill uses the customized profile through the same name

## MODIFIED Requirements

### Requirement: Emoji avatars have configurable backgrounds
An emoji avatar profile SHALL accept one Unicode emoji, a background color, optional sizing, an optional foreground color, and an optional scale. Scale SHALL default to `0.72` when omitted and SHALL remain independently configurable for every profile. The CLI SHALL expose the same optional foreground and scale modifiers for one-off emoji avatars. It SHALL resolve transparent emoji artwork, cache it in the user data directory, center it on the configured background, and render a square PNG. When a foreground is supplied, the renderer SHALL replace the artwork's visible RGB colors while preserving its original alpha mask and anti-aliased shape. When omitted, the original emoji colors SHALL remain unchanged.

#### Scenario: Render an emoji avatar without recoloring
- **WHEN** an emoji profile specifies `🚀` and background `#FFFFFF` without a foreground
- **THEN** the preview and webhook avatar contain the original centered rocket artwork over the white background

#### Scenario: Recolor an emoji silhouette
- **WHEN** an emoji profile specifies `❌`, background `#DD2E44`, foreground `#FFFFFF`, and scale `0.576`
- **THEN** the rendered avatar contains a shape-preserving white cross over the red background at the explicitly selected scale

#### Scenario: Emoji scale uses its default
- **WHEN** an emoji profile omits `scale`
- **THEN** the artwork is rendered at scale `0.72`

#### Scenario: Emoji asset is cached
- **WHEN** the same emoji is rendered after its artwork has been cached
- **THEN** the CLI uses the cached asset without another provider request

### Requirement: Locally rendered avatars can be used by Discord
Before executing a message with a local or generated avatar, the CLI SHALL select an incoming webhook identity dedicated to that rendered PNG. The identity SHALL be scoped to the resolved channel ID and PNG digest, initialized with the PNG as its default avatar, cached in private local state, and reused for later sends of the same image. It SHALL preserve the message's effective username and SHALL NOT mutate the base channel webhook's default avatar. Provisioning a generated identity SHALL require a resolved channel, a Bot token, and permission to manage channel webhooks; missing prerequisites SHALL fail before normal message delivery rather than falling back to Discord's default avatar.

#### Scenario: Apply a generated avatar
- **WHEN** a generated avatar digest has no cached identity for the resolved channel
- **THEN** the CLI provisions a dedicated webhook with that PNG avatar and executes the message through it with the effective username preserved

#### Scenario: Avatar identity is reused
- **WHEN** the same generated avatar digest is selected again for the same channel
- **THEN** the CLI reuses its cached webhook identity without modifying the base webhook or creating another identity

#### Scenario: Distinct generated avatars remain distinct
- **WHEN** two different generated avatar digests are sent consecutively to one channel
- **THEN** the messages use different webhook identities and each displays its selected avatar

#### Scenario: Generated identity cannot be provisioned
- **WHEN** a generated avatar is selected without a resolved channel, usable Bot token, or permission to manage webhooks
- **THEN** the CLI exits with an actionable error before sending the normal message

### Requirement: Omitted avatars use Discord's default
When CLI arguments, template frontmatter, and `[defaults]` all omit an avatar, the outgoing payload SHALL omit `avatar_url` and use the base channel webhook with Discord's default avatar. Dedicated generated-avatar identities SHALL not affect avatar-less sends. If legacy local state records that an older CLI modified the base webhook's avatar, the CLI SHALL reset that base avatar to `null` once and remove the legacy record before delivery.

#### Scenario: No avatar was ever applied
- **WHEN** no layer selects an avatar and state has no legacy applied-avatar digest for the base webhook
- **THEN** the CLI sends through the base webhook without an avatar override or avatar modification request

#### Scenario: Legacy generated avatar must be reset
- **WHEN** state records that an older CLI applied a generated avatar to the base webhook
- **THEN** the CLI resets the base webhook avatar, removes the legacy digest, and then sends using Discord's default avatar
