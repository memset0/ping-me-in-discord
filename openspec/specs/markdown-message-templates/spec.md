# markdown-message-templates Specification

## Purpose

Define a human-editable Markdown template format that can produce structured Discord webhook messages from CLI-provided data.

## Requirements

### Requirement: Plain message invocation uses the default template
Both `notify-me-on-discord` and its `pingme` alias SHALL accept a message as their top-level positional argument. The CLI SHALL expose that value as the `message` template variable and render `templates/defaults.md`.

#### Scenario: Send through the short alias
- **WHEN** the user invokes `pingme 'message content'`
- **THEN** the CLI renders `defaults.md` with `message` equal to `message content` and sends the resulting payload to the configured Discord channel

#### Scenario: Full binary name has the same shorthand
- **WHEN** the user invokes `notify-me-on-discord 'message content'`
- **THEN** it performs the same default-template send behavior as `pingme`

### Requirement: A default Markdown template is always addressable
The send command SHALL use `templates/defaults.md` when the user does not name a template. A named template SHALL resolve to a `.md` file inside the configured template directory, and path traversal SHALL be rejected.

#### Scenario: Send without a template name
- **WHEN** the user invokes `send` without selecting a template
- **THEN** the CLI renders `defaults.md`

#### Scenario: Named template
- **WHEN** the user selects `deployment`
- **THEN** the CLI renders `deployment.md` from the configured template directory

#### Scenario: Traversal attempt
- **WHEN** a template name contains an absolute path or a parent-directory component
- **THEN** the CLI rejects it before reading the file

### Requirement: Templates combine frontmatter and Markdown
Each template SHALL support optional YAML frontmatter delimited by `---`, followed by a Markdown body. The rendered Markdown body SHALL become Discord `content`, while supported frontmatter fields SHALL populate the remaining webhook payload and local delivery options.

#### Scenario: Frontmatter and body render together
- **WHEN** a template contains a `username`, an `avatar` reference, an embed, and a Markdown body
- **THEN** the payload contains the rendered username, embed, and body while the avatar reference is handled locally rather than sent as an unknown Discord field

#### Scenario: Body-only template
- **WHEN** a template has no frontmatter and has a non-empty body
- **THEN** the body is sent as Discord message content

### Requirement: Template variables are strict
The CLI SHALL render variables in both frontmatter and the Markdown body. Variables SHALL come from a JSON object and repeatable `--var key=value` arguments, with explicit `--var` values taking precedence. Referencing an undefined variable SHALL fail before any network request.

#### Scenario: Variables render in all sections
- **WHEN** `project=api` is provided and the template uses `{{ project }}` in its username and body
- **THEN** both locations contain `api`

#### Scenario: Missing variable
- **WHEN** a template references `{{ version }}` and no version is provided
- **THEN** rendering fails with an error that names `version` and no Discord request is made

### Requirement: Discord message structures are supported
Frontmatter SHALL support `channel`, `username`, `avatar`, `avatar_url`, `tts`, `embeds`, `allowed_mentions`, `components`, `poll`, `flags`, `thread_id`, and `thread_name`. The CLI SHALL reject an empty payload and SHALL validate Discord's basic content and embed count limits before delivery.

#### Scenario: Structured embed message
- **WHEN** frontmatter defines one or more embeds and the Markdown body is empty
- **THEN** the CLI produces a valid embed-only webhook payload

#### Scenario: Empty message
- **WHEN** a rendered template has no content, embeds, components, poll, or attachment
- **THEN** the CLI rejects the message before delivery

### Requirement: Send options use one consistent precedence
For every supported scalar send option, the CLI SHALL resolve an explicitly supplied command-line argument before template frontmatter and template frontmatter before the corresponding `[defaults]` setting. An absent value SHALL fall through, and a value absent from all layers SHALL remain unset. The initial layered fields SHALL be `channel`, `username`, avatar selection, `thread_id`, `thread_name`, and `tts`.

#### Scenario: CLI overrides template and settings
- **WHEN** CLI, frontmatter, and settings each specify a different `username`
- **THEN** the outgoing payload uses the CLI username

#### Scenario: Frontmatter overrides settings
- **WHEN** CLI omits `channel`, frontmatter selects `releases`, and settings select `alerts`
- **THEN** the `releases` channel selector is resolved and used

#### Scenario: Template normally omits routing
- **WHEN** CLI and frontmatter omit `channel` and `[defaults].channel` selects `alerts`
- **THEN** the configured `alerts` channel is used

### Requirement: Common send options are command-line arguments
Both the shorthand invocation and `send` subcommand SHALL accept `--channel`, `--username`, `--avatar`, `--avatar-url`, `--avatar-file`, `--avatar-emoji`, `--avatar-text`, `--avatar-icon`, `--avatar-background`, `--avatar-foreground`, `--avatar-font`, `--avatar-size`, `--avatar-font-size`, `--avatar-scale`, `--thread-id`, `--thread-name`, `--tts`, and `--no-tts`. Avatar source arguments SHALL be mutually exclusive, and conflicting options SHALL fail during argument validation.

#### Scenario: Override identity from the shorthand
- **WHEN** the user invokes `pingme --username "Deploy Bot" --avatar release "done"`
- **THEN** the final payload uses `Deploy Bot` and the configured `release` avatar profile

#### Scenario: Explicitly disable template TTS
- **WHEN** frontmatter sets `tts: true` and the user supplies `--no-tts`
- **THEN** the outgoing payload sets `tts` to false

### Requirement: Mentions are safe by default
When a template does not define `allowed_mentions`, the generated payload SHALL disable automatic mention parsing.

#### Scenario: Template contains an at-everyone string
- **WHEN** rendered content includes `@everyone` and frontmatter omits `allowed_mentions`
- **THEN** the payload includes an empty allowed-mention parse list

### Requirement: Templates can be inspected without delivery
The CLI SHALL list available templates and SHALL offer a dry-run mode that renders the selected template as redacted, formatted JSON without making network requests.

#### Scenario: Dry run
- **WHEN** the user sends a valid template with `--dry-run`
- **THEN** the CLI prints the final non-secret payload and performs no webhook provisioning, avatar update, or message execution
