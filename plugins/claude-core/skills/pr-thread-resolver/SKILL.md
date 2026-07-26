---
name: pr-thread-resolver
description: "Fetches, fixes, and resolves unresolved GitHub PR review threads with the gh CLI. Use when addressing reviewer feedback, clearing blocking conversations before merge, or applying GitHub suggested changes. Verifies each finding against current code and resolves only what it fixed."
license: MIT
---

# PR thread resolver

Turn unresolved GitHub PR review threads into committed, verified, resolved fixes — natively, with
`gh` + Claude Code's edit/verify loop. No external orchestrator.

## When to use / route away

Use for **hosted PR review threads**. Route away:
- Local review notes (Codex/Zen/manual) → `review-remediation`.
- Passive watch until merge / CI status → `babysit-pr`.
- CI-log-only failure with no review thread → `gh` workflow-log remediation.

## Default loop (autonomous: fix → commit → push → resolve)

1. **Ground**: read repo `AGENTS.md`/`CLAUDE.md`; `git status --short`; capture the head SHA
   (`gh pr view <pr> --json headRefOid -q .headRefOid`).
2. **Fetch threads**: list unresolved review threads via `gh api graphql` into a compact worklist
   (see `references/gh-graphql.md`): thread id, path, line, `isResolved`, `isOutdated`, body, any
   ```suggestion``` hunk.
3. **Triage** each: `Read` the current line. Skip with a reason if already-fixed or stale
   (`isOutdated` + code no longer matches); else queue a minimal fix.
4. **Plan**: decide order; serial for shared files, **fan out** for independent threads (below).
5. **Fix**: apply the smallest correct edit. Apply a ```suggestion``` block verbatim **only** on an
   exact hunk match; otherwise write a hand-minimal fix.
6. **Verify**: run repo-native gates — focused test for the touched area first, then broad
   type-check / lint / build (use `/verify` and `/code-review` as the backbone).
7. **Commit**: scoped, semantically grouped Conventional Commits (see `references/closeout.md`).
8. **Push** the branch once after all intended commits pass verification.
9. **Resolve**: re-fetch head + thread state; resolve via `resolveReviewThread` **only** threads
   backed by a committed+pushed+verified fix (or already-fixed at current head).
10. **Re-poll** until zero actionable threads remain or a real blocker appears.

## Parallel resolution

When ≥3 threads touch **disjoint files**, dispatch one read-and-fix `Task` subagent per file/cluster.
Subagent contract: **fix + verify + report only** (file:line, change, verification result) — it does
**not** commit, push, or resolve. The **parent** serializes all staging, commits, pushes, and
`resolveReviewThread` mutations. Workers own disjoint files and never touch another's.

## Resolve safety policy

Resolve a thread ONLY when it maps to a committed, pushed, and verified fix (or is already fixed at
current head). **Never** resolve when: the PR head drifted unexpectedly since you pushed; verification
failed; the thread was skipped, ambiguous, or unmatched. Do not auto-reply by default — reply only
when a thread cannot be fixed cleanly or the user asks for a comment.

## Outputs

Worklist · verified-fix / skipped summary (file:line evidence) · verification commands + results ·
commit SHAs · thread→commit closeout map · terminal status `completed` | `blocked` | `no-op`.

## Resources

- `references/gh-graphql.md` — copy-paste `gh api graphql` queries + the resolveReviewThread mutation.
- `references/closeout.md` — Conventional Commit grouping, forbidden process-wording, head-drift guard.
