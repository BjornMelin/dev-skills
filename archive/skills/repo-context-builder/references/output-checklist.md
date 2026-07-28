# Output checklist

Use this checklist before finalizing `REPO_CONTEXT.md` and `REVIEW_BRIEF.md`.

## Required checks

- Every section in both templates is filled.
- No placeholder text remains.
- Unknowns are labeled explicitly as `Unknown` or `Not found in repo`.
- Commands are exact and executable-looking.
- Key claims cite concrete file paths.
- Risks are specific, not generic.
- Monorepo outputs distinguish root concerns from package concerns.
- The brief includes a ready-to-paste working prompt for the next session.

## Quality bar for `REPO_CONTEXT.md`

`REPO_CONTEXT.md` should answer these questions quickly:

1. What is this repository for?
2. What are the main apps, services, packages, or modules?
3. How do I run, lint, typecheck, test, build, and deploy it?
4. What files should I read first?
5. What risks or unknowns matter right now?

## Quality bar for `REVIEW_BRIEF.md`

`REVIEW_BRIEF.md` should answer these questions quickly:

1. What is the task?
2. What is in scope and out of scope?
3. Which files and systems matter?
4. What does the repo already do today?
5. What is the recommended next plan?
6. How do I verify the work safely?
7. What prompt should I use next?

## Compression rule

Be concise, but not shallow.

Good:

- short tables
- one-paragraph summaries
- explicit file paths
- exact commands
- bullets with decisions and risks

Bad:

- generic narrative
- repeated wording across sections
- giant code excerpts
- giant file trees
- vague advice like "review the backend" or "run tests"
