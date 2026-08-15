## 1. Runtime Context

- [x] 1.1 Add optional `runtime.session.title` metadata while preserving the existing non-empty `runtime.session.name` compatibility behavior and cover both explicit-title and fallback cases.
- [x] 1.2 Update every bundled skill runner to preserve an absent session title and adjust runner regression tests.
- [x] 1.3 Update all bundled skill preflights and regression tests to validate the title-first session label with ID fallback.

## 2. Default Template

- [x] 2.1 Implement the conditional one-line starter header, update the bundled example, and test exact ordering, omission, and title-to-ID fallback behavior.
- [x] 2.2 Update the English README and detailed configuration reference for the new header and runtime field.

## 3. Verification and Demonstration

- [x] 3.1 Run formatting, unit/integration tests, lint checks, and strict OpenSpec validation.
- [x] 3.2 Install the updated binary and explicit local template outside Git, verify the rendered payload, and send one live message to the configured `test` channel.
