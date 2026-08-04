---
name: codex-delegate
description: Delegate implementation, investigation, or bulk work to gpt-5.6 codex via pinned codex exec. Use for clear-spec builds, migrations, debugging, or any task MODELS.md routes to codex.
---

# Codex Delegate

Hand a task to the gpt-5.6 family through the Codex CLI. This is the delegation
path per MODELS.md: the main loop (Root) invokes Codex **directly through Bash  - 
no Claude shim, and never a Claude worker whose only job is to launch, wait for,
or relay a Codex call**.

## Delegation gate (from MODELS.md - check before delegating at all)

Delegate only when at least one shape applies: `independence` (fresh-eyes or
adversarial value), `context` (repo/research mass stays out of the root
context), `contract` (clear-spec implementation with a written contract,
roughly >=2h), or `parallel` (independent taste-free lanes save wall-clock).
Otherwise Root works inline. Cap concurrent model lanes at 2-3.

Never route design/UI/copy/naming/API-ergonomics or final architecture
decisions to codex.

## Model + effort routing (always pin BOTH `-m` and effort - never inherit defaults)

| lane | pin | use |
|---|---|---|
| **default** | **`gpt-5.6-luna` + `"max"`** | almost everything: implementation, debugging, code review, retrieval, repo mapping, inventories, dependency tracing, bounded analysis and synthesis |
| fast | `gpt-5.6-luna` + `"high"` | latency-sensitive work: exploration, file and symbol location, shallow inventories, a quick second read - anywhere a fast good answer beats a slow better one |
| maximum intelligence | `gpt-5.6-terra` + `"max"` | one independent adversarial check or alternate solution, and the rare task Luna max cannot carry |
| last resort | `gpt-5.6-sol` + `"max"` | rare and root-gated; burns quota disproportionately |

Rationale, per the MODELS.md recalibration dated 2026-08-01: Luna is
substantially cheaper than when the old Sol ladder was written and its weekly
limits are effectively unlimited, so it is the default at max effort; Terra
earns its premium over Luna max only when the last few points of capability are
load-bearing. The Sol medium/high worker tiers are retired.

These are commercial terms, not stable facts — they were true at that
recalibration and will drift. Re-check them against current provider pricing
before treating cost as the reason for a routing decision, and prefer the
measured latency figures below, which do not depend on pricing.

**Quota is no longer the binding constraint on Luna; wall-clock is.** Measured on
one read-only analysis lane over a single component and its imports:

| effort | wall clock | tokens | findings |
|---|---|---|---|
| `high` | ~4 min | ~75k | 2 of 6 |
| `max` | ~14 min | ~153k | 6 of 6 |

The gap is **traversal depth, not polish**: the fast lane never entered
`node_modules`, so it missed everything that depended on what an installed
dependency actually does — including a high-confidence defect. Treat `high` as a
genuinely shallower read, not a cheaper version of the same read. Use it for
exploration and location, where a fast partial answer is the point, and `max`
for depth-sensitive work where the answer turns on what a dependency or a
distant file actually does.

Effort is a depth signal, not a completeness guarantee. Nothing about a
reasoning tier establishes that the scope was covered or the claims verified -
only explicit scope, coverage, and verification evidence does that. Do not
report a `max` lane as complete merely because it ran at `max`.

Escalation ladder: **Luna high → Luna max → Terra max → Root finishes the hard
part inline** (or an Opus high worker when a delegation shape holds). Sol max
requires one of: critical blast radius with no cheap deterministic oracle;
unresolved disagreement after Luna max and Terra max; two failed strong
attempts. Only one active Sol max call.

Bans: **no Sol xhigh or ultra; no Terra effort other than max; never
mini/spark-class models.**

Consequential or cross-cutting implementation still routes to Opus high workers
or Root inline per MODELS.md; Codex is the independent second opinion there, or
the primary when Claude quota is the binding constraint.

## Composing the prompt

Codex sees NONE of the Claude conversation - prompts must be fully
self-contained: objective and expected deliverable, exact scope/files and
ownership, relevant context and constraints, permitted edits and sandbox,
required checks, output format and completion criteria. For structured
returns, pass `--output-schema <schema.json>`.

## Invocation

```bash
# Investigation / retrieval (read-only, default lane)
codex exec -C "<repo>" -m gpt-5.6-luna -c model_reasoning_effort="max" --sandbox read-only --output-last-message "<scratchpad>/codex-out-<ts>.md" "<self-contained prompt>"

# Fast exploration (read-only) - trade depth for latency
codex exec -C "<repo>" -m gpt-5.6-luna -c model_reasoning_effort="high" --sandbox read-only --output-last-message "<scratchpad>/codex-out-<ts>.md" "<self-contained prompt>"

# Implementation (write-capable) - ALWAYS pass the sandbox
# explicitly; never inherit the config default (danger-full-access)
codex exec -C "<repo>" -m gpt-5.6-luna -c model_reasoning_effort="max" --sandbox workspace-write --output-last-message "<scratchpad>/codex-out-<ts>.md" "<self-contained prompt>"
```

Rules:
- ONE bare command per call - no pipes, no `cd &&` chains (keeps the RTK hook inert).
- Outside a git repository, add `--skip-git-repo-check` (codex exec errors there otherwise).
- Short blocking calls: foreground Bash. Long or parallel calls: Bash
  `run_in_background: true`; read the result file when the harness notifies  - 
  inspect logs only on failure or empty output. Never attach a Monitor just to
  detect completion.
- Use isolated worktrees when parallel writers could overlap; one owner per
  file/domain.
- Follow-ups continue the same Codex session:
  `codex exec resume --last "<follow-up instruction>"`. Sessions are looked up
  from the working directory, so run this from the same cwd/repo scope as the
  original call (or pass the recorded session id instead of `--last`).

## Closing the loop

Delegated output is provisional until Root closes it:
1. `git diff` - review what Codex actually changed before accepting it.
2. Run the relevant deterministic checks (tests / type-check / lint / build) and
   iterate until green.
3. If Codex went beyond the delegated scope, revert the extra scope and
   re-delegate with tighter constraints.

If output misses the bar, escalate up the ladder (or redo on opus-5/Root
inline) without asking - judge the output, not the price.
