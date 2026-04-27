---
name: "agents-md-maintainer"
description: "AGENTS.md maintenance rules for deciding when durable repo guidance should be updated after implementation."
---

Use this skill after nontrivial implementation work and before finalizing when you need to decide whether `AGENTS.md` should change.

## Rules

- Treat `AGENTS.md` as durable repo guidance, not a task log.
- Run a brief `AGENTS.md` maintenance pass at the end of implementation.
- Do not modify `AGENTS.md` unless the implemented code, tests, and docs state already proves the new rule.
- Update `AGENTS.md` only for durable repo-wide guidance such as:
  - required build, test, lint, typecheck, or verification commands
  - architecture invariants or hard constraints
  - canonical data shapes, contracts, directory ownership, or file locations
  - repo-wide coding, review, or release expectations
  - recurring failure modes that future sessions must avoid
- Do not update `AGENTS.md` for one-off feature details, branch context, temporary migrations, issue notes, task plans, historical summaries, or implementation narration.
- Keep edits minimal and high-signal. Prefer replacing, tightening, deduplicating, or deleting existing bullets over appending prose.
- Keep `AGENTS.md` clean, concise, and production-facing.
- Do not include working-session phrasing such as `Phase`, `plan`, `follow-up`, or similar developer-tracking language.
- Write rules as short, concrete, imperative bullets that a coding agent can apply immediately in a new session with no prior chat context.
- Keep root `AGENTS.md` limited to broadly applicable guidance. Put scoped rules in the nearest folder-level `AGENTS.md`. Put reusable workflows in skills or referenced docs.
- Ensure every `AGENTS.md` edit matches the current codebase exactly, and remove stale or superseded guidance in the same pass.
- Use `$reducing-entropy`, `$clean-code`, and `$hard-cut` when deciding whether guidance should be added, tightened, or deleted.
- If delegation is allowed and helpful, use a small subagent to compare the implemented diff against `AGENTS.md` and propose the smallest valid patch.
- Before finishing, verify that each `AGENTS.md` change is:
  - durable
  - repo-relevant
  - non-duplicative
  - consistent with current code and tests
  - useful to a coding agent starting from zero context
- Leave `AGENTS.md` unchanged if any of those checks fail.

## Workflow

1. Compare the implemented diff against the current `AGENTS.md`.
2. Decide whether the completed work changed durable repo guidance.
3. Stop if the answer is no.
4. If the answer is yes, make the smallest valid patch.
5. Re-check the edited guidance against the current codebase, tests, and docs.
6. Delete or tighten any stale, duplicated, or superseded bullets in the same pass.

## Examples

- Update `AGENTS.md` if the repo now requires a new canonical verification command before completion.
- Do not update `AGENTS.md` if the work only added a feature-specific route, component, or migration note.
