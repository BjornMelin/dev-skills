---
name: codex-delegate
description: Delegate implementation, investigation, or bulk work to gpt-5.6 codex via pinned codex exec. Use for clear-spec builds, migrations, debugging, or any task MODELS.md routes to codex.
---

# Codex Delegate

Hand a task to the gpt-5.6 family through the Codex CLI. This is the delegation
path per MODELS.md: the main loop (Fable) invokes Codex **directly through Bash  - 
no Claude shim, and never a Claude worker whose only job is to launch, wait for,
or relay a Codex call**.

## Delegation gate (from MODELS.md - check before delegating at all)

Delegate only when at least one shape applies: `independence` (fresh-eyes or
adversarial value), `context` (repo/research mass stays out of the root
context), `contract` (clear-spec implementation with a written contract,
roughly >=2h), or `parallel` (independent taste-free lanes save wall-clock).
Otherwise Fable works inline. Cap concurrent model lanes at 2-3.

Never route design/UI/copy/naming/API-ergonomics or final architecture
decisions to codex.

## Model + effort routing (always pin BOTH `-m` and effort - never inherit defaults)

| lane | pin | use |
|---|---|---|
| bulk | `gpt-5.6-luna` + `"high"` (high ONLY) | retrieval, repo mapping, inventories, dependency tracing, evidence extraction, routine low-risk clear-spec bulk edits |
| worker | `gpt-5.6-sol` + `"medium"` | implementation, debugging, code review, bounded analysis/synthesis |
| lead | `gpt-5.6-sol` + `"high"` | consequential/cross-cutting backend work, difficult debugging, planning, research synthesis |
| validator | `gpt-5.6-terra` + `"max"` (max ONLY) | one independent adversarial check or alternate solution AFTER Sol high - not a routine worker or generic escalation tier |
| last resort | `gpt-5.6-sol` + `"max"` | root-gated; see below |

Escalation ladder: **Luna high → Sol medium → Sol high**, then one of:
Fable finishes the hard part inline, Terra max runs one adversarial/alternate
pass, or (rarely) Sol max.

Bans: **no Sol xhigh or ultra; no Luna effort other than high; no Terra effort
other than max; never mini/spark-class models.** Sol max requires one of:
critical blast radius with no cheap deterministic oracle; unresolved
disagreement after Sol high + Terra max; two failed strong attempts; a hard
implementation that materially benefits from Codex repo/tool context. Only one
active Sol max call.

## Composing the prompt

Codex sees NONE of the Claude conversation - prompts must be fully
self-contained: objective and expected deliverable, exact scope/files and
ownership, relevant context and constraints, permitted edits and sandbox,
required checks, output format and completion criteria. For structured
returns, pass `--output-schema <schema.json>`.

## Invocation

```bash
# Investigation / retrieval (read-only, bulk lane)
codex exec -C "<repo>" -m gpt-5.6-luna -c model_reasoning_effort="high" --sandbox read-only --output-last-message "<scratchpad>/codex-out-<ts>.md" "<self-contained prompt>"

# Implementation (write-capable, worker lane) - ALWAYS pass the sandbox
# explicitly; never inherit the config default (danger-full-access)
codex exec -C "<repo>" -m gpt-5.6-sol -c model_reasoning_effort="medium" --sandbox workspace-write --output-last-message "<scratchpad>/codex-out-<ts>.md" "<self-contained prompt>"
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

Delegated output is provisional until Fable closes it:
1. `git diff` - review what Codex actually changed before accepting it.
2. Run the relevant deterministic checks (tests / type-check / lint / build) and
   iterate until green.
3. If Codex went beyond the delegated scope, revert the extra scope and
   re-delegate with tighter constraints.

If output misses the bar, escalate up the ladder (or redo on opus-4.8/Fable
inline) without asking - judge the output, not the price.
