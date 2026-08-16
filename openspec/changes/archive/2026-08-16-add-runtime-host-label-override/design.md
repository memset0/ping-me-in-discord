## Context

The CLI shares one `SendOptions` structure between shorthand sends and the explicit `send` subcommand, then captures runtime metadata while constructing the reserved template context. The current starter template independently combines `runtime.user` and `runtime.hostname`, which prevents a caller from replacing that combination as one display label. Existing custom templates may depend on both compatibility fields, so they cannot be repurposed or removed.

The avatar renderer discovers an installed font that contains the requested glyph. One unit test requests a CJK character, making its success depend on a font package that standard CI runners do not guarantee even though the production renderer itself is behaving correctly.

## Goals / Non-Goals

**Goals:**

- Give both send entry points one unambiguous, per-invocation host-label override.
- Preserve existing runtime identity fields for custom templates.
- Keep automatic metadata useful when the caller supplies no override.
- Make the font-discovery unit test portable without narrowing production glyph support.

**Non-Goals:**

- Add a host value to configuration files or template frontmatter.
- Change Discord webhook usernames or avatar behavior.
- Remove or reinterpret `runtime.user` and `runtime.hostname`.
- Guarantee that every Unicode glyph can render without a compatible user-installed font.

## Decisions

### Store a complete optional `runtime.host` label

Runtime capture will add an optional `host` field alongside the existing `user` and `hostname` fields. Automatic capture will produce `user@hostname` when both values are known, hostname alone when the user is unknown, and no host label when the hostname is unknown. The starter template will render this field directly.

This keeps display composition in one place and gives custom templates a simple field to consume. Keeping the component fields unchanged is preferable to overwriting them because an explicit label such as `mukai-h20` cannot be reliably split back into user and hostname components.

### Pass the CLI override into runtime capture

The shared send options will expose `--host <LABEL>` and pass it through template-context construction to runtime capture. A supplied value will replace only the computed `runtime.host`; it will not enter the config/frontmatter precedence system and will not alter webhook `username`.

Using the existing shared option structure ensures shorthand and explicit sends stay equivalent. A dedicated runtime flag is preferable to `--var host=...`, because ordinary template variables must not mutate the reserved `runtime` namespace. Separate hostname and omit-user flags were rejected because a complete label directly expresses both requested outcomes with less API surface.

### Validate and normalize explicit labels centrally

Runtime capture will reject an override after trimming when it contains no content, then normalize valid content with the same one-line, Discord-safe normalization used for other runtime strings. Validation occurs before template rendering completes and before any network work. This prevents a blank override from silently falling back to machine identity while keeping line breaks or formatting delimiters from corrupting the metadata footer.

### Test font discovery with an ASCII glyph

The system-font discovery test will render an English glyph that is available in standard Latin system fonts. Production code and its Unicode-capable lookup remain unchanged; the test continues to verify discovery, font parsing, and image output without asserting that the operating system installed a particular language pack.

## Risks / Trade-offs

- [Existing custom templates keep constructing `runtime.user@runtime.hostname` and therefore ignore `--host`] → Document `runtime.host` as the override-aware display field while retaining the old fields solely for compatibility.
- [A caller expects `--host` to persist or participate in template precedence] → Document it with other per-send CLI options and explicitly exclude it from settings/frontmatter layering.
- [Normalization changes an override containing newlines or Discord inline-code delimiters] → Apply the established runtime normalization consistently and test the resulting single-line value.
- [ASCII coverage does not exercise a non-Latin glyph] → Retain parser-level Unicode coverage and production behavior; keep system-font availability outside the portable unit-test contract.

## Migration Plan

Ship the new optional runtime field and CLI flag without rewriting existing user templates. New initialization uses the override-aware starter template; existing templates remain untouched until users choose to update them. Rolling back removes only the new flag and field, while the retained compatibility fields leave prior templates unaffected.
