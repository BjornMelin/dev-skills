---
name: hotspot-optimizer
description: >-
  Find algorithmic complexity and performance hotspots across a codebase, then optimize them safely
  behind tests without changing behavior. Use when asked to scan for slow code, inefficient or nested
  loops, repeated scans, costly re-renders or recomputation, N+1 queries, avoidable quadratic work, or
  to reduce complexity while preserving tests, APIs, ordering, and outputs; or when a request mentions
  performance hotspots, hot paths, render churn, "this is slow", or "optimize for speed". Fans out
  across files with Task subagents, ranks findings by impact, proves current behavior with focused
  tests, applies localized edits, and re-runs the narrow test plus broad type, lint, and build to
  verify. Defaults to an analysis-only report unless you ask to implement, fix, apply, or refactor.
license: MIT
---

# Hotspot optimizer

Find and (on request) fix complexity/performance hotspots with **behavior preserved and proven by
tests**. Bias toward small, verified wins over broad rewrites.

## Core rule

Never change observable behavior. Every optimization preserves outputs, ordering, error semantics,
and public APIs — and is proven by a test that passed before and after.

## Loop: Triage → Prove → Optimize → Verify

1. **Triage** — get candidate leads fast, then reason about them:
   - First pass (cheap, optional): `python3 scripts/scan_hotspots.py <root> --format json` for a
     ranked multi-language lead list. Treat output as *leads, not proof*.
   - Confirm with `Grep`/`Glob`/`Read`: nested loops over the same data, membership tests in a loop
     (list vs set), sort-in-loop, pairwise comparisons, repeated scans, render recomputation, N+1
     queries. Rank by impact (hot path × input size), not raw count.
2. **Prove** — before touching hot code, run/author a focused test that pins current behavior and, if
   feasible, a quick timing or complexity witness.
3. **Optimize** — apply the smallest localized `Edit` that lowers complexity (see
   `references/optimization-playbook.md`). No drive-by refactors or formatting churn.
4. **Verify** — re-run the focused test, then the broad gates (type-check, lint, build, suite). If any
   regresses, revert that change.

## Subagent fan-out

For large repos, shard files/dirs across `Task` subagents (read-only). Each returns a ranked findings
list: `file:line`, pattern, estimated current → target complexity, risk. The main agent dedupes and
merges into one ranked plan, then optimizes serially (it owns all edits). Cap concurrency at ~4-6
subagents; prefer fewer, broader shards over many tiny ones.

## Optimization safety checklist

- Same outputs for the same inputs (including ordering and ties).
- No swallowed/altered exceptions; same error types.
- No new unbounded memory (a set/dict cache must be bounded by the same data).
- Equivalent for empty / single / duplicate / large inputs.
- Public API and call sites unchanged.

## Analysis-only by default

Default to a report (see `references/report-template.md`): ranked findings with location, current and
target complexity, risk, and a proposed fix — **but do not edit**. Apply fixes only when the user says
implement / fix / apply / refactor (or pre-approves low-risk wins like list→set membership).

## Resources

- `scripts/scan_hotspots.py` — heuristic multi-language hotspot scanner (leads only).
- `references/optimization-playbook.md` — transformation catalog + correctness checks + what-not-to-do.
- `references/report-template.md` — analysis-only report skeleton.
