---
name: subspawn
description: Bounded subagent delegation + synthesis for Codex. Use for `spawn_agent` planning/exec, 1–3 bounded subtasks, model + effort pick, read-only vs edit scope, wait behavior, conflicting findings, evidence-first synthesis. Trigger -> explicit subagents/delegation/fan-out/model routing/spawn asks, or task needs parallel exploration or narrow implementation ownership.
---

# Subspawn

Keep subagents bounded; main agent owns planning, synthesis, final decisions unless user says otherwise.

## Trigger Rules

Use this skill when:

- user asks for subagents, delegation, fan-out, or spawn policy
- task benefits from 1–3 independent bounded subtasks
- model or effort routing for subagents matters
- session needs evidence-first synthesis across delegated results

Skip skill to rationalize extra delegation. Keep work local when next critical-path step urgent, tightly coupled, or faster direct.

## User Overrides

Explicit user instructions override the default delegation heuristics.

If user specifies any below, honor unless unsafe or impossible:

- exact fan-out count
- mandatory delegation or mandatory no-delegation
- specific subagent model
- specific reasoning effort
- explicit wait behavior
- explicit read-only vs edit-capable delegation

When override differs from default:

- keep task bounded anyway
- restate override in spawn contract
- keep synthesis + final decisions in main agent unless user says otherwise

## Pre-Spawn Checklist

Before spawning:

1. State immediate local task you will keep.
2. Identify bounded sidecar subtasks that can be run in parallel.
3. Tighten each subtask before bumping model or effort:
   - narrow task statement
   - narrow allowed scope or write surface
   - define exact output contract
   - define verification expectations
4. Default read-only exploration unless scoped edits materially advance task.

## Fan-Out Rules

- Prefer 1–3 focused subagents.
- No nested subagents unless user asks.
- Explorer-style: read-heavy, evidence-focused.
- Implementation: narrow, explicit file ownership.
- Wait for all requested subagents before final synthesis.
- Do not block on `wait_agent` reflexively. Continue useful local work unless next step blocked on result.

## Model Policy

Default subagent model policy:

- use `gpt-5.4-mini` first
- prefer `gpt-5.3-codex` as larger fallback
- do not use `gpt-5.4` unless user asks

Always set `model` and `reasoning_effort` explicitly on every `spawn_agent` call.

Non-default model or effort: one-line reason in spawn prompt.

## Effort Routing

For `gpt-5.4-mini`:

- `medium`: default most bounded work—exploration, read-heavy scans, codebase tracing, doc/API verification, focused audits, standard review, normal tool calling, standard bounded implementation
- `high`: ambiguous findings, cross-file reasoning, conflicting evidence, tricky debugging, edge cases, security-sensitive inspection, critical logic
- `xhigh`: rarely necessary, reserved for unusually hard bounded reasoning
- `low`: only simple deterministic tasks—path lookup, quick grep, tiny file checks, surface inventory

Prefer `medium` over `low` by default.

Before moving from `medium` to `high`, tighten:

- task statement
- output contract
- verification requirements

For `gpt-5.3-codex`:

- `low`: straightforward implementation
- `medium`: normal non-trivial work
- `high`: only when genuinely necessary

No default `xhigh`. Only use for unusually hard, bounded reasoning tasks.

## Spark Lane

Use `gpt-5.3-codex-spark` only low-token, low-latency exploration, quick triage, light drafting.

Do not use Spark for:

- broad or ambiguous research
- heavy tool-calling workflows
- code changes
- final decisions
- high-risk reasoning

Spark has a 128K context window and may get stuck in compaction loops on
overscoped tasks. Prefer `gpt-5.4-mini` for heavier research and tool-driven
exploration because it is safer on context-heavy tasks.

## Escalation Rules

Escalate only specific subagents that underfit.

Escalation order:

1. tighten scope + output contract
2. tighten verification requirements
3. raise `gpt-5.4-mini` `medium` → `high` if needed
4. switch only that subagent to `gpt-5.3-codex`

`gpt-5.3-codex` only when:

- `gpt-5.4-mini` clearly underfits after one tighten pass
- task cross-cutting, ambiguous, or high-risk

## Mandatory Spawn Contract

Every `spawn_agent` call must explicitly specify:

1. narrow task or question
2. allowed scope or surfaces
3. whether the agent is read-only or may edit
4. wait for all agents before continuing or not
5. exact return format

Spawn prompt shape:

```text
Task: <one bounded task or question>
Scope: <paths, modules, docs, APIs, or explicit ownership>
Mode: <read-only | may edit <files>>
Wait: <wait for all agents before final synthesis: yes|no>
Model: <explicit model>
Reasoning: <explicit reasoning_effort>
Reason: <only include when using non-default model or effort>
Return format:
- Key finding or result
- Files and symbols touched or inspected
- Commands or checks run, if any
- Recommended next action
- Unresolved questions or risks
```

For edit-capable workers, also say:

- You are not alone in the codebase.
- Do not revert edits made by others.
- Adjust your implementation to accommodate concurrent changes.

## Return Contract

Require concise, evidence-first output:

- key finding or result
- file paths + symbols touched or inspected
- commands or checks run, if any
- recommended next action
- unresolved questions or risks

No vague summaries without evidence.

Prefer:

- file refs over narration
- symbol refs over generic descriptions
- concrete command evidence over assertions

## Verification Rules

For proposed changes, require one of:

- tests or checks run
- concrete command-level verification plan when execution is not appropriate

Subagent proposes edits without tests: require exact checks to run next.

## Synthesis Rules

Before final answer:

1. wait for all requested subagents
2. merge overlapping findings
3. drop duplicate observations
4. surface conflicts explicitly
5. resolve disagreements in main-agent synthesis
6. highest-signal evidence first

When findings conflict, produce a conflict ledger with:

- conflicting claim
- evidence each side
- resolution or remaining uncertainty

## Forward-Testing

Validating skill: include ≥1 probe forcing real delegation, not only refusal path.

Good probe:

- explicitly 2 read-only subagents
- disjoint bounded questions
- final wait-for-all synthesis
- each spawn includes full mandatory spawn contract

## Failure Modes

Common failure modes + fixes:

- Overscoped subtask: shrink task + allowed surface.
- Weak output: restate return contract + evidence requirements.
- Mini underfits: tighten once, escalate only that subagent.
- Spark stalls or compacts: cut context sharply or move task to `gpt-5.4-mini`.
- Too many agents: collapse to 1–3, synthesis local.
- Edit collisions: read-only discovery or disjoint write surfaces.
