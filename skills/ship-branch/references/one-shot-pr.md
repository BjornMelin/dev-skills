# One-Shot PR Shipping

Use this `ship-branch` reference for one-shot shipping of the current task.

## Autonomous Invocation

If the user explicitly asks to ship the current task with no extra detail:

1. Read the repo `AGENTS.md`.
2. Inspect the git state with `ship-branch/scripts/inspect_git_state.py`.
3. If on `main` or `master`, create a `feat/` branch inferred from the diff.
4. Create exactly one scoped conventional commit unless the user explicitly asks for multiple commits.
5. Push the branch.
6. Open a PR to `main` with GitHub CLI if available; otherwise emit the title, body, and minimal manual steps.

Stop and ask only when the tree is empty, unrelated dirty changes make staging ambiguous, or repo policy conflicts with automatic shipping.

## Workflow

1. Read the repo `AGENTS.md`.
2. Run `python3 /home/bjorn/.agents/skills/ship-branch/scripts/inspect_git_state.py`.
3. If the tree is mixed, isolate the intended changes into scoped conventional
   commits before opening the PR.
4. Stage and commit the intended changes with a scoped conventional commit.
5. Push to the preferred remote.
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
