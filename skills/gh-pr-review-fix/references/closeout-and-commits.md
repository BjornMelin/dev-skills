# Closeout And Commit Rules

## Review-Thread Closeout

- Treat fresh hosted review-thread state as the closeout source of truth.
- Resolve fixed threads only after the relevant fix is committed, pushed, and validated on the current PR head.
- Keep a thread-to-commit evidence map for every resolved item:
  - current hosted thread ID
  - worklist item ID
  - commit SHA or already-fixed head SHA
  - validation command that passed
  - closeout status from `codex-dev pr review closeout`
- Do not resolve skipped, stale, ambiguous, or failed-validation items.
- Do not post explanatory replies unless the user asks or a thread cannot be safely resolved.

## Semantic Commit Grouping

Follow the `$commit` staging rules:

1. Read `git status --short` before staging.
2. Touch only files owned by the current remediation task.
3. Inspect diffs before staging and group by semantic purpose, not reviewer comment order.
4. Stage one coherent review unit at a time; do not use `git add .` in a mixed tree.
5. Default to one scoped Conventional Commit when the fix set is cohesive.
6. Split commits only when changes are clearly separate semantic groups.
7. Prefer commit order: `feat`, `fix`, `test`, `docs`, `refactor`, `chore`.
8. Run `git status -sb` between groups and leave unrelated dirty files untouched.
9. Stop before staging unrelated user changes, generated churn you do not understand, or ownership-ambiguous files.

## Conventional Commit And SemVer Rules

- Required subject format: `<type>(<scope>): <code/docs/test behavior>`.
- Scope must name the real codebase owner or surface, such as `codex-dev`, `codex-dev-core`, `gh-pr-review-fix`, `review-remediation`, `docs`, `sdk`, `mobile`, or the affected package/module.
- Do not use process scopes such as `review`, `feedback`, `comments`, or `codex`.
- SemVer meaning:
  - `feat` -> minor
  - `fix` or `perf` -> patch
  - `docs`, `test`, `refactor`, `chore`, `ci`, `build` -> no public SemVer impact by default
  - use `!` and a `BREAKING CHANGE:` footer for public CLI/schema/API/behavior breaks
- The subject must describe what changed, not why the reviewer asked for it.

Allowed examples:

- `fix(codex-dev): preserve review-thread closeout evidence`
- `feat(gh-pr-review-fix): resolve fixed hosted threads by default`
- `docs(review-remediation): route local notes through codex-dev review`
- `test(codex-dev): cover semantic commit subject rejection`

Use `codex-dev --json commit validate --subject "<subject>"` before each commit. That command owns process-wording rejection; if validation fails, rewrite the subject until it describes the semantic behavior.
