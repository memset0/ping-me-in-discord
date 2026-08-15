## 1. Template Selection

- [x] 1.1 Add shared validation and resolution for safe template names or exact absolute `.md` paths, and use it for CLI and configured defaults.
- [x] 1.2 Make offline configuration validation compile an absolute default template while keeping template listing scoped to the configured directory.

## 2. Verification

- [x] 2.1 Add unit coverage for named templates, absolute Markdown paths, relative traversal, and absolute non-Markdown paths.
- [x] 2.2 Add CLI coverage for rendering a template outside the configured directory and using an absolute configured default.

## 3. Documentation and Quality

- [x] 3.1 Update CLI help, the English README, and the configuration reference with the absolute-path syntax and boundaries.
- [x] 3.2 Run formatting, Clippy, the full locked test suite, and strict OpenSpec validation.
