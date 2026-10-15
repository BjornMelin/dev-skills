---
name: code-quality
description: "Implement, fix, simplify, refactor, or review code with evidence-based reuse, clear ownership, precise types, focused tests, and removal of accidental complexity. Use for substantive code changes and code-quality reviews. Skip prose-only, generated-only, dependency-only, or purely mechanical edits."
---

# Code quality

Build the smallest clear solution that fully satisfies the approved behavior.
Understand the system before shortening it. Repository contracts and explicit
user decisions take precedence over generic style preferences.

## Establish the scope

- Use the user-named scope. Otherwise use the current task's changed files or
  branch diff. If neither exists, ask for scope rather than auditing the repo.
- Infer the mode: implementation, bug fix, simplification, or review. A review
  request authorizes findings, not edits. Simplification preserves required
  behavior; an approved behavior change implements the intended end state.
- Preserve unrelated work. Read applicable guidance, relevant call paths, tests,
  and history only where they resolve a concrete uncertainty.
- Establish required behavior, failure handling, domain rules, and verification
  before editing. Do not turn a clear task into a planning ceremony.

## Find the owner before adding code

Search in this order: existing project behavior and patterns, platform or
standard library, installed dependencies, then new code or a dependency.

Trace callers and consumers. Look for duplicate validation, defaults, contracts,
state, cache, transformations, permissions, and error handling. Fix the root
cause at the appropriate owner. Reuse equivalent behavior; do not centralize
superficially similar logic with different domain rules.

Use the following lenses together, in the same scoped pass:

| Lens | Questions |
| --- | --- |
| Behavior | Are outputs, errors, side effects, ordering, and recovery correct? |
| Ownership | Is each rule owned once, at a boundary that has the needed context? |
| Complexity | Can nesting, flags, indirection, wrappers, duplication, or dead code disappear? |
| Types | Do contracts preserve precise domain evidence without casts or broad escape types? |
| Efficiency | Is there demonstrated unnecessary work, I/O, allocation, rendering, or contention? |
| Tests | Does coverage protect meaningful behavior and failure modes without mirroring implementation? |

## Implement the coherent end state

- Prefer clear names, direct control flow, and cohesive responsibilities.
  Extract only when a concept, boundary, or real reuse earns the abstraction.
  Keep a useful helper even when inlining would save lines.
- Do not add speculative configuration, interfaces with no meaningful boundary,
  redundant state synchronization, forwarding wrappers, or unmeasured caching
  and memoization. Preserve intentional performance and concurrency controls.
- Keep precise types. Use `unknown` at untrusted boundaries and narrow or parse
  it there. Do not widen domain contracts and recover them with assertions.
- Preserve necessary trust-boundary validation, authorization, accessibility,
  data integrity, error handling, and recovery. Fewer checks is not a success
  metric. Neither is fewer lines by itself.
- For an approved contract change, update producers, consumers, fixtures, tests,
  and directly affected docs together. Remove superseded paths within scope.
  Establish real deployed, persisted, public, external, or dependent-branch
  obligations before deleting compatibility. Old code alone proves no such
  obligation, but absence of a static match alone proves no safe deletion.
- Inspect dynamic references, configuration, generated consumers, migrations,
  and relevant release boundaries before declaring a path unused. If callers
  or obligations remain outside the approved scope, preserve the boundary and
  report the limitation; do not silently widen the task.
- Do not force architecture changes into a behavior-preserving cleanup. When
  existing structure obstructs the approved solution, refactor the affected
  surface progressively and remove what the replacement makes obsolete.

## Verify proportionally

Choose the smallest checks that can expose the actual regression risk and obey
repository-required gates. Reuse passing evidence for unchanged code. Broaden
for shared boundaries, failures, material risk, or unresolved concerns.

Treat tests as maintained code: preserve useful coverage, consolidate overlap,
and remove stale or implementation-coupled cases in the affected surface.
Add a regression when it protects a plausible failure, boundary, or capability.
Do not require a test for every edit, rewrite good tests to fit a bug, or weaken
production behavior to satisfy a bad test.

Use independent review when warranted and authorized by the task or repository.
Do not impose a fixed reviewer count, model, full-suite loop, function-size limit,
line-count threshold, or separate-PR rule. If delegation is unavailable or not
authorized, review inline. Follow the existing validation owner rather than
adding another wrapper or gate.

## Finish with an ablation pass

Review the final diff for defects, missed reuse, duplicate ownership, type
weakening, stale code and tests, and unnecessary churn. For each added or retained
helper, branch, test, configuration, compatibility path, and layer, identify the
required behavior, clarity, safety, or coverage it provides. Remove ceremonial
complexity without removing its load-bearing responsibility.

Stop when the approved behavior works, relevant checks pass, and the affected
surface is coherent. Do not manufacture cleanup findings or widen the scope to
justify invoking this skill.

## Severity

- **HIGH:** demonstrable correctness, security, data-integrity, or required-flow
  failure. Explain the concrete trigger and consequence.
- **MEDIUM:** material regression risk, conflicting ownership, or maintainability
  burden with a concrete affected path and justified correction.
- **LOW:** a local clarity improvement with modest impact. Omit taste-only nits.

## Review output format

Lead with the outcome. For a review, report actionable findings in priority
order with location, evidence, consequence, and the smallest justified fix.
Say when there are no meaningful findings. For implementation, list what
changed and why, verification results, and material unresolved limits.
Distinguish confirmed defects from unverified concerns. Do not claim measured
speed, cost, or token savings without a comparable baseline.
