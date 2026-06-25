---
name: fan-out
description: >-
  Delegate work to bounded subagents and synthesize their evidence in Claude Code. You stay the
  planner and sole decision-maker; subagents are tightly scoped, return structured evidence, and you
  read every result before acting. Use when the user asks to delegate, fan out, parallelize, spawn
  subagents, or set a spawn policy; when a task splits into 2-5 independent subtasks (multi-area code
  search, parallel research, multi-file review); when you need a Task subagent, a custom .claude/agents
  role, or the dynamic Workflow tool (pipeline/parallel/agent); or when long jobs should run as
  background Bash tasks while the parent stays responsive. Covers mechanism choice, spawn contracts,
  read-only vs scoped-edit roles, disjoint write surfaces, model policy (Opus for hard reasoning),
  rendezvous discipline, and conflict-ledger synthesis. Spawn only when asked or when independence
  clearly helps. Not for authoring agent definitions (claude-subagent-creator) or web research
  (deep-research).
license: MIT
---

# Fan-out

Bounded subagent delegation and evidence-first synthesis for Claude Code. You stay the planner and
the single decision-maker; subagents do scoped work and hand back evidence you merge.

## When to fan out (and when not to)

Spawn only if at least one is true:
- The user explicitly asked to delegate / parallelize / spawn subagents.
- The task splits into **2-5 genuinely independent** subtasks (no shared mutable state).
- Independent work would otherwise pollute the main context (large reads, broad searches).

Otherwise keep it local. Do **not** rationalize a spawn for work that is sequential, tiny, or shares
write surface — coordination overhead and edit collisions cost more than they save.

## Mechanism selection

| Situation | Use |
|---|---|
| One bounded subtask, keep main context clean | a single `Task` call |
| 2-5 independent subtasks, need all results together | **batch `Task` calls in one message** (they run concurrently) |
| Dependent / multi-stage / DAG-shaped batch | the dynamic **Workflow** tool (`pipeline`/`parallel`/`agent` stages) |
| Genuinely long-running, parent must stay responsive | **background Bash task** (`run_in_background`) + `TaskGet`/`TaskStop` |

Default to batched `Task` calls. Reach for Workflow only when stages depend on each other or you need
loop/fan-out control flow; reach for background tasks only for true async.

## Spawn contract (every subagent prompt)

```
TASK:    <one concrete outcome>
SCOPE:   <paths / area to look at; what is out of bounds>
MODE:    read-only  |  edit ONLY <named files>
CONTEXT: <self-contained — subagents inherit NO parent transcript>
RETURN:  <exact shape: status, evidence (file:line), commands run, findings + confidence, risks>
```

Subagents start fresh with no conversation history — never reference "the above" or prior turns.

## Roles & model policy

- Pick a custom `.claude/agents/<role>.md` when one matches; else `general-purpose`.
- **Opus** for hard reasoning, review, debugging, planning; inherit (or Sonnet) for read-heavy scans
  and inventories. Pin `model:` in the agent file, or omit to inherit the session model.
- Default subagents to **read-only** (no `Edit`/`Write` in their tool scope) unless they own a
  disjoint, named write surface.

## Rendezvous & async discipline

- Synchronous `Task` batches block until every subagent returns — **read all results** before the
  next substantive step; never act on a partial mental model.
- Background tasks: account for every one via `TaskGet`; `TaskStop` anything stale or superseded.

## Limits & edit safety

- Prefer 2-5 agents; more dilutes synthesis. No nested subagents unless the user asks.
- Parallel edit-workers must own **disjoint files**. Tell each: "you are not alone in this repo — do
  not revert or restage another worker's changes." The parent serializes commits.

## Escalation

If a subagent underfits, tighten its scope and return contract and re-run **only that one** on a
stronger (Opus) role. Never blanket-escalate the whole batch.

## Synthesis & conflict ledger

Merge and dedupe returns; lead with the highest-signal evidence. When subagents disagree, record it
and resolve in the parent:

```
CONFLICT: <claim>
  - agent A: <evidence>
  - agent B: <evidence>
  RESOLUTION: <which wins + why, or the remaining uncertainty>
```

## Verification

Any proposed edit ships with tests/checks actually run, or a concrete command-level verification plan
the parent (or a final verify subagent) executes.

## Failure modes

| Symptom | Fix |
|---|---|
| Subagent overscoped / wandered | tighter SCOPE + RETURN contract |
| Weak / unusable output | specify exact return shape + evidence format |
| Parent acted on partial results | wait for all returns before deciding |
| Forgot a background task | reconcile every task via `TaskGet` |
| Edit collision | disjoint write surfaces; parent serializes commits |
| Role underfit | re-run only that agent on Opus |
| Too many agents | collapse to 2-5 with broader scopes |

See `references/agent-templates.md` for copyable role stubs and a Workflow skeleton.
