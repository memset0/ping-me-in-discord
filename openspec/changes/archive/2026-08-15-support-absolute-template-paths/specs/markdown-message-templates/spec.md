## MODIFIED Requirements

### Requirement: A default Markdown template is always addressable
The send command SHALL use `templates/defaults.md` when the user does not select another template. A simple template name SHALL resolve to a `.md` file inside the configured template directory. An absolute template selector ending in `.md` SHALL resolve to that exact file, including when the file is outside the configured template directory. Other relative paths, parent-directory traversal, and absolute paths that do not end in `.md` SHALL be rejected before reading a file.

#### Scenario: Send without a template name
- **WHEN** the user invokes `send` without selecting a template
- **THEN** the CLI renders `defaults.md`

#### Scenario: Named template
- **WHEN** the user selects `deployment`
- **THEN** the CLI renders `deployment.md` from the configured template directory

#### Scenario: Absolute template path from the CLI
- **WHEN** the user supplies `--template /tmp/pingme/custom.md`
- **THEN** the CLI renders that exact file without requiring it to be inside the configured template directory

#### Scenario: Absolute default template path
- **WHEN** `[defaults].template` is an absolute path ending in `.md` and the user does not supply `--template`
- **THEN** the CLI renders that exact file as the default template

#### Scenario: Traversal attempt
- **WHEN** a template selector is a relative path or contains a parent-directory component
- **THEN** the CLI rejects it before reading the file

#### Scenario: Absolute non-Markdown path
- **WHEN** an absolute template selector does not end in `.md`
- **THEN** the CLI rejects it before reading the file
