---
name: new-branch
description: Create and switch to a new conventional-commit and semver-compliant branch before doing any work, then present the full execution plan for that branch and its pull request. Use when the user wants a branch prepared first and a clear plan in chat before implementation.
---

# New Branch

1. Before any other work, inspect the current branch/worktree state.
2. Create and checkout a new branch for the task using a conventional-commit and semver-compliant name, keeping it short and scoped, for example `feat/add-x`, `fix/y`, or `chore/z`.
3. Stop after switching to the new branch unless the user separately asks for implementation work.
4. In chat, provide the complete plan for work to be done on that branch and its pull request: intended changes, key files, verification, commit shape, and any risks or blockers.
