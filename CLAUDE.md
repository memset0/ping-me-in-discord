# Project Instructions

## Documentation

- Write `README.md` and all future modifications to it in English.

## OpenSpec and Git Workflow

- Use the repository's generated OpenSpec workflows.
- Before implementation, read `openspec/config.yaml`, the relevant main specs under `openspec/specs/`, and the active change artifacts under `openspec/changes/`.
- Keep OpenSpec artifacts aligned with the implementation and validate the completed change before archiving.

### Installation and updates

- After every successful OpenSpec installation, reinstallation, repair, or update, create a separate Git commit containing only the repository files that operation modified or created.
- Stage the exact affected paths explicitly. Do not mix application changes, archive changes, or unrelated work into the OpenSpec setup commit.
- Create the commit on `main` and follow the commit-message rules below. Prefer `chore(openspec): install OpenSpec tooling`, `chore(openspec): reinstall OpenSpec tooling`, or `chore(openspec): update OpenSpec tooling`, as applicable.
- Do not create an empty commit when the operation changed no repository files. Do not commit when the installation or update failed.
- Keep this setup commit separate from the automatic archive commit. Do not push the setup commit unless the user separately requests a push.

### During apply

- Keep the current change's spec in sync throughout `/opsx:apply`.
- If implementation requires small bug fixes or feature tweaks beyond what the change originally described, promptly fold them back into the current change's `proposal.md`, delta specs under `openspec/changes/<name>/specs/`, and `tasks.md` as applicable.
- Do not let implementation drift ahead of the spec. Before archiving, ensure the change artifacts describe what was actually built.

### Archiving

- When `/opsx:archive` prompts about delta spec sync, default to syncing: choose `Sync now (recommended)` so the change's delta specs under `openspec/changes/<name>/specs/` are merged into the main specs under `openspec/specs/<capability>/spec.md`.
- Only skip syncing if the user explicitly asks to archive without syncing.
- After archiving, verify both the updated main specs and the archived change.
- After every successful archive, including the spec sync above, automatically commit the files involved in that change and push the commit to `main`. Do not wait for a separate request or confirmation.
- Do not auto-commit or push if the archive step fails.

### Concurrent-agent commits

- Assume multiple agents may be editing the same working tree.
- Track every file modified or created by the current task and inspect the final diff before staging.
- Stage only those exact paths with `git add -- <path>...`: the archived OpenSpec artifacts, synced main specs, and code or documentation modified by the change.
- Never use `git add -A`, `git add .`, or `git add -u`; those commands can sweep up other agents' work in progress.
- File-level staging is sufficient; line-level or hunk-level staging is not required.
- If a file was modified by both this change and another agent concurrently, it is acceptable to commit the whole file because the on-disk state is the state that runs. Confirm the final file list before committing and explicitly report any such shared file afterward.
- Do not amend, rewrite, or discard another agent's work.

### Concurrency-safe push to main

- All commits must be created on the `main` branch unless the user explicitly authorizes another branch for the current task.
- Before committing, confirm the checked-out branch is `main`, its upstream is `origin/main`, and the staged file list contains only the paths described above. If not on `main`, stop and report; do not switch branches or push another branch to `main` implicitly.
- Fetch `origin/main` and verify local `main` is not behind or diverged before committing. Do not merge, rebase, reset, stash, or alter other agents' working-tree changes automatically to catch up.
- Create the commit only after the archive and sync have succeeded and the final staged diff has been reviewed.
- Push with `git push origin main`. Never force-push.
- If the push is rejected because `origin/main` advanced, fetch and report the race. Do not rewrite history or disturb the shared working tree; leave the local commit intact and ask for the safest integration decision.

### Branches and worktrees

- Do not create or switch branches unless the user explicitly authorizes it for the current task.
- Do not create, enable, switch, manage, or move work into a Git worktree unless the user explicitly authorizes it for the current task.

### Commit messages

- Use Conventional Commits, following the Angular-style format, for every commit: `<type>(<scope>): <description>`.
- Use an appropriate standard type such as `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `ci`, or `chore`.
- Keep the subject concise and imperative.
- For a breaking change, add `!` before the colon and include a `BREAKING CHANGE:` footer.
- Example: `feat(research): add automated experiment runner`.
- For a pure OpenSpec archive, prefer `chore(openspec): archive <change-name>`.
- When implementation files are included, choose the conventional type and scope that best describe the change.

### Repository-specific OpenSpec and Git Workflow Requirements

- After completing a proposal, proceed directly to applying it without waiting for another confirmation, unless the user explicitly asks to stop after the proposal or the requirements are ambiguous enough to require clarification.
- This repository uses `master` as its primary branch; apply branch-specific rules above to `master` and `origin/master` instead of `main` and `origin/main`.
- Override the canonical setup-push rule above: after every successful OpenSpec installation, reinstallation, repair, or update commit, automatically push `master` to `origin/master` without waiting for another request.
