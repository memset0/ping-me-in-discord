# Project maintenance

## OpenSpec workflow

- After completing a proposal, proceed directly to applying it without waiting for another confirmation, unless the user explicitly asks to stop after the proposal or the requirements are ambiguous enough to require clarification.
- After successfully archiving a change, commit the completed work and archive artifacts, then push the current branch automatically. If the branch has no upstream, set `origin/<current-branch>` as its upstream. Do not include unrelated user changes in the commit.

## Git commits

- Use Conventional Commits (the Angular-style format) for every commit: `<type>(<scope>): <description>`.
- Use an appropriate standard type such as `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `ci`, or `chore`.
- Keep the subject concise and imperative. Add `!` and a `BREAKING CHANGE:` footer when a change is breaking.
- Example: `feat(research): add automated experiment runner`.
