# Closeout: commits & resolve safety

## Conventional Commits, semantically grouped

- One commit per coherent concern; do not lump unrelated thread fixes together.
- Subject: `type(scope): imperative summary` (e.g. `fix(auth): guard null session on refresh`).
  Types: `fix`, `feat`, `refactor`, `test`, `docs`, `chore`, `perf`.
- Body (when useful): what changed and why; reference the review concern, not the reviewer.
- Keep commits reviewable: small, focused diffs that a human can read in one pass.

## Forbidden process-wording

Commit messages describe the code change, not the workflow. Avoid: "as requested by reviewer",
"address review comments", "PR feedback", "resolve thread", "per review", AI/tooling attribution, or
filler. Say what the code now does instead (e.g. not "address review comment" → `fix(parse): handle
trailing comma in CSV header`).

## Verify before commit, push once

1. Focused check on the touched area (the narrowest test/command that exercises it).
2. Broad gates once before push: type-check, lint, build, and the suite the repo defines.
3. Push the branch a single time after all intended commits pass — not per-commit.

## Head-drift guard (hard safety)

- Record the SHA you pushed.
- Immediately before each `resolveReviewThread`, re-read `headRefOid`.
- If it differs from what you pushed, **stop**: someone/something changed the PR. Do not resolve;
  report the drift and re-fetch thread state.

## Resolve only on evidence

Resolve a thread ONLY when ALL hold:
- A committed + pushed fix maps to it (or the code is already fixed at current head), AND
- Verification passed, AND
- The head has not drifted since you pushed.

Never resolve a skipped, ambiguous, unmatched, or unverified thread. Leave it open with a one-line
reason in the summary.
