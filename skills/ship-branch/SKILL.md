---
name: ship-branch
description: Commit + push relevant files, split conventional commits, open PR to main with title/body. Triggers—commit, push, open PR, scaffold PR/release defaults.
---

# Ship Branch

## Workflow

1. Inspect `git status --short` first.
2. Leave unrelated dirty files untouched.
3. Group changes by semantic purpose and split mixed work into separate commits when needed.
4. Use short scoped Conventional Commit messages that match the change.
5. If the current branch is `main`, create a working branch first; otherwise stay on the current branch.
6. Commit all intended changes before pushing.
7. Push the branch to the remote.
8. Open a PR into `main` with `gh pr create`.
9. Use a Conventional Commits PR title and a reviewer-ready PR body built from [repo-defaults.md](references/repo-defaults.md).

For one-shot PR shipping formerly covered by `ship-pr`, read [one-shot-pr.md](references/one-shot-pr.md) and use `scripts/inspect_git_state.py`.

## Commit Rules

- Prefer one commit per semantic change.
- Keep commit order aligned to reviewer value: `feat`, `fix`, `test`, `docs`, `refactor`, `chore`.
- Do not stage unrelated files or use `git add .` unless the whole task is one coherent change.
- Stop if the tree is too mixed to separate cleanly or ownership is ambiguous.

## Repo Defaults On Request

- If the user explicitly asks to create missing repo defaults, scaffold the files from [repo-defaults.md](references/repo-defaults.md).
- Otherwise, do not add repo-level template or Release Please files.

## Resources

- `scripts/inspect_git_state.py`
- [references/one-shot-pr.md](references/one-shot-pr.md)
- [references/repo-defaults.md](references/repo-defaults.md)
