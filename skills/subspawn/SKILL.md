---
name: subspawn
description: Bounded subagent delegation and synthesis. Use for explicit subagent, delegation, fan-out, spawn policy, custom agent routing, wait behavior, and evidence-first synthesis.
---

# Subspawn

Keep subagents bounded. The main agent owns planning, waits for delegated
results, synthesizes evidence, and makes final decisions unless the user says
otherwise.

## Trigger Rules

Use this skill when:

- user asks for subagents, delegation, fan-out, or spawn policy
- task benefits from 1-3 independent bounded subtasks
- model or effort routing for subagents matters
- session needs evidence-first synthesis across delegated results
- custom agent selection or fork behavior matters

Only spawn when the user explicitly asks for subagents or parallel agent work.
Do not use this skill to rationalize extra delegation. Keep work local when
delegation would add coordination cost without improving quality.

## User Overrides

Explicit user instructions override the default delegation heuristics.

If user specifies any below, honor unless unsafe or impossible:

- exact fan-out count
- mandatory delegation or mandatory no-delegation
- specific subagent model
- specific reasoning effort
- explicit wait behavior
- explicit read-only vs edit-capable delegation
- specific custom agent role

When override differs from default:

- keep task bounded anyway
- restate override in spawn contract
- keep synthesis + final decisions in main agent unless user says otherwise

## Pre-Spawn Checklist

Before spawning:

1. State the exact batch you will spawn and why each subtask is independent.
2. Identify any work the main agent must pause until results arrive.
3. Tighten each subtask before increasing model or effort:
   - narrow task statement
   - narrow allowed scope or write surface
   - define exact output contract
   - define verification expectations
4. Default to read-only exploration unless scoped edits materially advance the
   task.

For nontrivial batches, generate a plan before spawning:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset research \
  --task "Research current Codex subagent docs" \
  --scope "official OpenAI docs and GitHub only"
```

Use `--role` for explicit custom role selection, `--mode edit` only with
disjoint write surfaces, and `--json` when another tool needs to consume the
plan. Run `validate-roles` after changing subagent templates.
Packaged standalone `subspawn` installs include local preset template copies in
`templates/agents/`; full repo checkouts prefer the sibling deep-researcher and
subagent-creator templates first.

## Fan-Out Rules

- Prefer 1-3 focused subagents; use 4-6 only for genuinely independent read,
  test, or audit lanes with one writer at most.
- No nested subagents unless user asks.
- Explorer-style: read-heavy, evidence-focused. Prefer a custom
  `repo_explorer` role when installed; keep Codex built-in `explorer` as a
  fallback and avoid custom names that accidentally shadow built-ins.
- Implementation: narrow, explicit file ownership.
- Prefer custom roles when they exist and match the task; fall back to built-in
  `explorer`, `worker`, or `default`.
- State selected roles in the spawn prompt.

## Rendezvous Rule

After spawning a planned batch, immediately wait for every spawned subagent in
that batch before doing substantive next work.

While subagents are running, allowed actions are limited to:

- `wait_agent` or equivalent wait/status operations
- `send_input` only to unblock or correct a running subagent
- `resume_agent` or `close_agent` for lifecycle recovery
- reporting a wait timeout, tool error, or explicit blocker to the user

Do not inspect files, run tests, browse, edit, continue local analysis, start
new unrelated tool calls, or produce the final answer until the spawned batch
has completed and its results have been synthesized. The only exception is an
explicit user instruction to run asynchronous delegation without waiting.

When the runtime exposes explicit close operations, close completed subagent
threads after synthesis so long-lived sessions do not accumulate stale agent or
MCP resources.

## Model Policy

Follow the active Codex tool schema and the custom agent files available in the
session.

- Omit per-call `model` and `reasoning_effort` when inheriting the parent model
  or when a custom agent file already pins them.
- Use `gpt-5.6-terra` at `medium` for mechanical inventory and at `high` for
  bounded retrieval and source gathering.
- Use `gpt-5.6-sol` at `medium` for default judgment and implementation; use
  Sol `high` for planning, architecture, security, root-cause work, and lead
  synthesis.
- Use Terra `max` only for independent adversarial validation.
- Do not use Sol `xhigh`, `max`, or `ultra` for routine delegated work. Sol
  `max` is a root-only emergency escalation after tighter scope and Sol `high`
  still underfit.
- Do not route Luna in V2 until native custom-agent support is verified.

Only set `model` or `reasoning_effort` directly when the user asks or the
current tool schema supports that override and it is necessary. Include a
one-line reason in the spawn prompt when overriding.

## Effort Routing

Use `medium` as the default for most agents.

- Use `high` when an agent must trace complex logic, check assumptions, handle
  edge cases, investigate security-sensitive code, or resolve conflicting
  evidence.
- Use Terra `high` for bounded read-heavy source gathering.
- Use Terra `max` only for an independent adversarial lane, never the primary
  planner or implementer.

Before increasing effort, tighten:

- task statement
- output contract
- verification requirements

## Fork And Context Policy

Use the current `spawn_agent` schema exactly.

Runtime compatibility matrix:

| Surface | Typical fields | Safe custom-role posture |
| --- | --- | --- |
| legacy tool | `agent_type`, `model`, `reasoning_effort`, `fork_context` | Keep prompt self-contained. Avoid combining full `fork_context` with role/model overrides if rejected. |
| multi-agent v2 | `agent_type`, `model`, `reasoning_effort`, `task_name`, `fork_turns` | Set stable `task_name`; use `fork_turns: "none"` for custom role/model/effort overrides unless inherited history is required. |
| custom agent file | standalone TOML role under `~/.codex/agents` or `.codex/agents` | Prefer omitting per-call model/effort when the role file pins them. |

If the schema exposes `fork_turns` and `task_name`:

- Always provide a stable snake_case `task_name`.
- For custom agents, built-in `agent_type`, or direct model/reasoning overrides,
  set `fork_turns: "none"` unless the user explicitly requests inherited
  history. Current Codex multi-agent v2 rejects full-history forks with role,
  model, or reasoning overrides.
- Use `fork_turns: "all"` only when the child must inherit full parent context
  and no role/model/reasoning override is required.
- Use a positive integer string for partial history when a small amount of
  recent context is enough.

If the schema exposes legacy `fork_context`:

- Do not combine full-context fork behavior with explicit role/model/reasoning
  overrides if the tool reports that combination is unsupported.
- Prefer fresh, bounded prompts with the required context embedded.

## Escalation Rules

Escalate only specific subagents that underfit.

Escalation order:

1. tighten scope + output contract
2. tighten verification requirements
3. raise effort on only that subagent if needed
4. switch only that subagent to a stronger model when available

## Mandatory Spawn Contract

Every spawned subagent prompt must explicitly specify:

1. narrow task or question
2. allowed scope or surfaces
3. whether the agent is read-only or may edit
4. strict wait expectation
5. exact return format

Spawn prompt shape:

```text
Task: one bounded task or question
Scope: paths, modules, docs, APIs, or explicit ownership
Mode: read-only, or may edit named files only
Wait: parent will wait for all spawned agents before substantive next work
Role: selected custom or built-in agent role
Model: inherited, custom-agent pinned, or explicit override with reason
Reasoning: inherited, custom-agent pinned, or explicit override with reason
Return format:
- Status
- Evidence or role-specific source headings
- Files inspected/changed, queries run, or sources hydrated
- Commands run or provider calls
- Findings or claims with confidence and source IDs
- Risks/blockers
```

When a custom role template provides a narrower return contract, use the
template-specific headings emitted by the planner.

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

## Wait Timeout Rules

Use long bounded waits for the full batch rather than short polling loops. If a
wait times out:

1. make one status or follow-up attempt for the running agents
2. wait once more with a bounded timeout
3. synthesize completed results and clearly mark unresolved agents, or close
   stuck agents if their output is no longer useful

Do not forget open subagents. Final output must account for every spawned
subagent as completed, timed out, closed, or failed.

## Synthesis Rules

Before any substantive next work or final answer:

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

Validating this skill requires at least one probe forcing real delegation, not
only a refusal path.

Good probe:

- explicitly 2 read-only subagents
- disjoint bounded questions
- strict wait-for-all synthesis
- each spawn includes full mandatory spawn contract

Planning-only smoke:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset review \
  --task "Review this PR for correctness and test gaps" \
  --scope "current PR diff only"
```

## Failure Modes

Common failure modes + fixes:

- Overscoped subtask: shrink task + allowed surface.
- Weak output: restate return contract + evidence requirements.
- Parent overlaps work: stop, wait for the batch, then synthesize.
- Parent forgot a subagent: wait or close it, then account for it in synthesis.
- Role/model override rejected: retry with fresh context or compatible fork
  fields for the active tool schema.
- Lightweight model underfits: tighten once, then escalate only that subagent.
- Too many agents: collapse to 1–3, synthesis local.
- Edit collisions: read-only discovery or disjoint write surfaces.
