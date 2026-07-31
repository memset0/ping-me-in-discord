## Context

See `proposal.md` for motivation. Template context construction currently starts with optional JSON data, overlays the positional `message`, and then overlays repeatable variables. The embedded and example default templates contain only `{{ message }}`. The project targets Linux first, also publishes Windows and macOS binaries, and enforces Rust 1.87 as its minimum supported version.

## Goals / Non-Goals

**Goals:**

- Produce one deterministic, serializable runtime snapshot per template render.
- Discover account and hostname without invoking platform shell commands.
- Format local time consistently across supported operating systems.
- Preserve caller Markdown and existing user-owned templates.
- Make tests independent of the executing machine, username, timezone, and clock.

**Non-Goals:**

- Making the default footer layout configurable through TOML; users customize the Markdown template itself.
- Exposing the working directory, OS, architecture, process ID, or other potentially sensitive runtime details.
- Replacing Discord's own message timestamp or converting the static local timestamp for each viewer.

## Decisions

### Use a typed, reserved `runtime` object

Construct a serializable runtime object with `user`, `hostname`, and nested `timestamp.local`, `timestamp.unix`, and `timestamp.iso8601` fields. Insert it after validating caller data and reject any caller-supplied top-level `runtime` key with an actionable error. A namespace avoids ambiguity with webhook frontmatter `username` and leaves room for compatible metadata additions.

An alternative was independent top-level variables. That is terser but more collision-prone and makes the relationship among the three timestamp forms unclear. Allowing callers to replace the runtime object was also considered, but would make the default template silently display untrusted or structurally invalid metadata.

### Separate discovery from context merging

Represent the snapshot as a value passed into an internal context builder. Production obtains one live snapshot; tests inject fixed strings and a fixed instant. This keeps formatting and precedence tests deterministic without environment mutation or sleeps.

### Use cross-platform identity and time libraries compatible with Rust 1.87

Use `whoami` with only its standard platform support for fallible username and hostname discovery. Normalize empty or multiline results and fall back independently to `unknown-user` and `unknown-host`. Use `jiff` to capture a system-zoned instant once, format `%-m/%-d %H:%M:%S`, and derive Unix and UTC ISO 8601 forms from that same instant. Both dependencies have MSRVs below the project floor and avoid subprocesses.

Environment-variable-only discovery was rejected because `USER`, `USERNAME`, `HOSTNAME`, and `COMPUTERNAME` are not consistently present or authoritative across supported platforms. UTC-only formatting was rejected because the accepted design explicitly shows the emitter's local wall-clock time.

### Keep the starter template exact and user-owned

The embedded starter and repository example will contain exactly:

```jinja
> **🏠 `{{ runtime.user }}@{{ runtime.hostname }}`   📅 `{{ runtime.timestamp.local }}`**
{{ message }}
```

The metadata remains a Discord blockquote because `>` begins the first line. The body begins on the next line with no blank separator. Installation continues replacing binaries only, and initialization retains its existing non-overwrite behavior. The currently configured local portable template is migrated separately only because the user explicitly approved this layout.

## Risks / Trade-offs

- [Local timestamps are ambiguous outside their origin timezone] → Also expose Unix and UTC ISO 8601 representations for custom templates; keep the accepted concise local form in the starter.
- [Machine identity may reveal infrastructure naming] → Expose only user and hostname, and let users remove the footer by editing their local template.
- [A prior custom `runtime` variable collides with the new namespace] → Fail before sending with a migration message instead of silently changing its structure.
- [Platform identity lookup fails or returns unsafe layout characters] → Normalize values to a single display-safe line and use explicit fallbacks without blocking delivery.
- [Starter output consumes part of Discord's content limit] → Continue validating the fully rendered payload against the existing limit and report an error before delivery.

## Migration Plan

1. Add dependencies and the injectable runtime snapshot.
2. Update starter/example templates, tests, and documentation.
3. Verify all supported behavior, Rust 1.87 compatibility, and release compilation.
4. Build and replace the two local binaries, then update only the explicitly approved executable-adjacent default template.
5. Validate locally, dry-run the exact layout, and send one confirmation to the configured `test` channel.

Rollback is a binary downgrade plus restoration of the prior one-line local template; configuration and credential files are unchanged.
