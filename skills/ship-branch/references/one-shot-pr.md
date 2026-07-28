# One-Shot PR Shipping

Use this `ship-branch` reference for one-shot shipping of the current task.

## Autonomous Invocation

If the user explicitly asks to ship the current task with no extra detail:

1. Read the repo `AGENTS.md`.
2. Inspect the git state with `scripts/inspect_git_state.py`.
3. Put the work on a named branch. Create a `feat/` branch inferred from the
   diff when on `main`/`master`, **and also when `HEAD` is detached** — there
   `git rev-parse --abbrev-ref HEAD` returns the literal `HEAD`, and a bare
   `git push origin` fails with `fatal: You are not currently on a branch`.
4. Create exactly one scoped conventional commit unless the user explicitly asks for multiple commits.
5. Push the branch.
6. Open a PR to `main` with GitHub CLI if available; otherwise emit the title, body, and minimal manual steps.

Stop and ask only when unrelated dirty changes make staging ambiguous, or repo
policy conflicts with automatic shipping.

An empty worktree is **not** a reason to stop. The work may already be
committed and simply unpushed, which is a normal state for this flow. Check
whether the branch is ahead of its upstream before concluding there is nothing
to ship:

```bash
git status --porcelain=v1 --branch --ahead-behind
git log --oneline @{upstream}..HEAD 2>/dev/null || git log --oneline -5
```

Stop only when the worktree is clean *and* the branch has no unpushed commits.

## Workflow

1. Read the repo `AGENTS.md`.
2. Run the bundled helper. Resolve it relative to this skill rather than to any
   absolute home directory, so the flow works for other users and for
   project-local installs:

   ```bash
   python3 "$(dirname "$0")/../scripts/inspect_git_state.py"   # from a script beside this reference
   python3 scripts/inspect_git_state.py                        # from the skill root
   ```

3. If the tree is mixed, isolate the intended changes into scoped conventional
   commits before opening the PR.
4. Stage and commit the intended changes with a scoped conventional commit.
5. Push to the preferred remote, creating the upstream on first push.
6. Open the PR to `main` with `gh pr create` when possible.
7. Report the branch, commit, and PR result concisely.

## Use When

- The user wants to ship the current work in one explicit flow.
- The task is branch, commit, push, and PR creation together.

## Do Not Use When

- The user only wants commits organized locally.
- The user asks for merge or deployment after PR creation.

## Outputs

- git state summary
- commit summary
- push target
- PR URL or manual PR payload
- terminal status: `completed`, `blocked`, or `needs-user`

## Resources

- `scripts/inspect_git_state.py`
