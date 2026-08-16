## 1. Runtime Host Override

- [x] 1.1 Add the shared `--host <LABEL>` send option and derive or validate the reserved `runtime.host` value without changing compatibility identity fields.
- [x] 1.2 Pass the host override through template context construction and update bundled starter templates to render `runtime.host`.

## 2. Automated Coverage

- [x] 2.1 Add runtime, template, and CLI tests for automatic labels, complete overrides in both invocation forms, normalization, blank rejection, and webhook-username independence.
- [x] 2.2 Replace CJK system-font assumptions in avatar unit tests with portable English glyph coverage while retaining production Unicode behavior.

## 3. Documentation

- [x] 3.1 Document `--host`, `runtime.host`, its relationship to `--username`, and the revised starter template in the English README.
- [x] 3.2 Update detailed configuration documentation and bundled examples for the override-aware runtime host field.

## 4. Validation

- [x] 4.1 Run formatting, linting, and the complete Rust test suite in a size-controlled temporary build directory.
- [x] 4.2 Validate the completed OpenSpec change strictly and reconcile its artifacts with the implementation.
