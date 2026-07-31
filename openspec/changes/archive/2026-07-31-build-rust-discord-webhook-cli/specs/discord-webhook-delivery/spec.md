## Purpose

Define secure Discord incoming-webhook acquisition and reliable message delivery for both preconfigured and Bot-provisioned workflows.

## ADDED Requirements

### Requirement: Direct incoming webhooks are supported
The CLI SHALL send with a configured Discord incoming webhook URL without requiring a Bot token. The webhook URL SHALL be treated as a secret and SHALL not appear in ordinary output.

#### Scenario: Direct webhook delivery
- **WHEN** a valid webhook URL and rendered message are available
- **THEN** the CLI executes that webhook and reports the created Discord message identifier

### Requirement: A Bot can provision the incoming webhook
When a channel is selected, the CLI SHALL use a Bot token, resolved channel ID, and configured webhook name to find an existing incoming webhook or create one. Provisioning SHALL require Discord's `MANAGE_WEBHOOKS` permission and SHALL cache the returned webhook URL by channel for later sends.

#### Scenario: Matching webhook already exists
- **WHEN** the Bot can manage webhooks and the configured channel already contains a matching incoming webhook with a token
- **THEN** the CLI reuses and caches that webhook instead of creating a duplicate

#### Scenario: Matching webhook does not exist
- **WHEN** the Bot can manage webhooks and no matching webhook exists
- **THEN** the CLI creates one, caches its execution URL, and uses it for delivery

#### Scenario: Bot lacks permission
- **WHEN** Discord denies webhook listing or creation
- **THEN** the CLI fails with an actionable message that identifies the required `MANAGE_WEBHOOKS` permission

#### Scenario: Different channels have independent cached webhooks
- **WHEN** sends resolve to two different channel IDs
- **THEN** each channel uses its own cached or newly provisioned webhook URL

### Requirement: Credential selection is deterministic
When no channel is selected, the CLI SHALL prefer an environment-provided direct webhook URL and then a configured direct URL. When a channel is selected, it SHALL use the cached webhook for that channel or Bot provisioning, because an incoming webhook cannot be redirected to another channel at execution time. An invalid selected credential SHALL be reported rather than silently falling back.

#### Scenario: Cached provisioned webhook
- **WHEN** a channel is selected and cached state contains a valid webhook URL for that channel
- **THEN** the CLI uses the cached webhook without sending a Bot-authenticated provisioning request

#### Scenario: Channel selection with only a direct webhook
- **WHEN** a channel is selected but no Bot token or matching cached channel webhook is available
- **THEN** the CLI explains that direct webhooks are channel-bound and Bot provisioning is required for channel selection

### Requirement: Delivery waits for confirmation
Webhook execution SHALL request a Discord response, surface non-success HTTP responses with secret-safe context, and return success only after Discord confirms message creation.

#### Scenario: Discord accepts a message
- **WHEN** Discord returns a successful message object
- **THEN** the CLI prints a concise success result containing the message ID

#### Scenario: Discord rejects a payload
- **WHEN** Discord returns a validation error
- **THEN** the CLI exits nonzero and reports the status and Discord error without exposing the webhook token

### Requirement: Rate limits receive bounded retries
The CLI SHALL honor Discord's retry delay for HTTP 429 responses and retry a bounded number of times. It SHALL not retry permanent authentication or validation failures.

#### Scenario: Temporary webhook rate limit
- **WHEN** Discord returns 429 with a valid retry delay and the retry budget remains
- **THEN** the CLI waits for that delay and retries the request

### Requirement: Discord API v10 is used for Bot operations
Bot-authenticated list and create operations SHALL use explicit Discord API v10 endpoints and a `Bot` authorization scheme.

#### Scenario: Create a webhook
- **WHEN** provisioning creates a webhook
- **THEN** the request targets the API v10 channel webhook endpoint with `Authorization: Bot <token>`
