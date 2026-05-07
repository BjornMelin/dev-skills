# Subspawn Reference

Path:

```text
skills/subspawn/
```

Purpose: bounded subagent delegation and synthesis policy for Codex sessions.

## Planner CLI

Path:

```text
skills/subspawn/scripts/subspawn_plan.py
```

Use the planner before nontrivial fanout to make role selection, scope, wait
policy, and synthesis expectations explicit.

List presets:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py list-presets
```

Generate a strict research fanout plan:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset research \
  --task "Research current Codex subagent docs" \
  --scope "official OpenAI docs and official GitHub repositories only"
```

Generate JSON for another tool:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset dependency \
  --task "Assess whether the dependency upgrade is safe" \
  --scope "package docs, release notes, source, and issue tracker" \
  --json
```

Validate available role names and return-contract headings:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
```

Default presets:

| Preset | Roles |
| --- | --- |
| `research` | `openai_docs_researcher`, `github_researcher`, `citation_auditor` |
| `dependency` | `context7_researcher`, `source_validator`, `github_researcher` |
| `review` | `reviewer`, `false_positive_validator`, `test_runner` |
| `implementation` | `repo_explorer`, `implementation_worker`, `test_runner` |
| `docs` | `docs_researcher`, `docs_auditor`, `citation_auditor` |

Use `--role` to select explicit roles, `--mode edit` only when write surfaces
are disjoint, `--max-agents` to keep the batch bounded, and
`--allow-large-batch` only when the user explicitly requests a larger batch.

## Core Contract

The main Codex session owns:

- planning;
- spawn boundaries;
- waiting;
- synthesis;
- final decisions.

Subagents own bounded evidence gathering or scoped implementation.

## Strict Rendezvous Rule

After the parent spawns a planned batch, it must immediately wait for every
spawned subagent before doing substantive next work.

Allowed while waiting:

- `wait_agent`;
- `send_input` only to unblock or correct a running subagent;
- `resume_agent` or `close_agent` for lifecycle recovery;
- reporting a timeout/tool blocker.

Not allowed while waiting:

- file inspection;
- tests;
- browsing;
- edits;
- local analysis;
- final answer;
- unrelated tool calls.

Exception: the user explicitly asks for asynchronous delegation without waiting.

## When to Spawn

Use subagents when the user explicitly asks for:

- subagents;
- delegation;
- fan-out;
- parallel agent work;
- role/model spawn policy.

Do not spawn to rationalize extra work. Keep work local when delegation adds
coordination overhead without improving evidence or throughput.

## Recommended Fanout

Prefer 1-3 focused subagents.

Good fanout examples:

- docs researcher + source validator + citation auditor;
- GitHub issue archaeology + release/changelog check;
- independent codebase explorer lanes for separate subsystems.

Bad fanout examples:

- multiple agents reading the same files for the same question;
- edit-capable agents with overlapping file ownership;
- nested subagents without explicit user instruction;
- parent doing the same work while children run.

## Mandatory Spawn Contract

Every spawned prompt should include the fields below. Planner output is the
authoritative copy-ready shape; the block below is the conceptual contract it
must preserve.

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
- Evidence
- Files inspected/changed
- Commands run
- Findings
- Risks/blockers
```

## Model and Effort

Default posture:

- inherit model/effort when role files already pin them;
- use `gpt-5.5` for hard review, debugging, security, and planning;
- use `gpt-5.4-mini` for lighter read-heavy scans;
- raise effort only after tightening scope and output expectations.

## Synthesis

Before final answer or next substantive work:

1. wait for all requested subagents;
2. merge overlapping findings;
3. drop duplicates;
4. surface conflicts;
5. resolve disagreements in parent synthesis;
6. account for every subagent as completed, failed, closed, or timed out.

When findings conflict, produce a conflict ledger:

- conflicting claim;
- evidence for each side;
- resolution or remaining uncertainty.

## Failure Recovery

If a wait times out:

1. send one targeted status/unblock message if useful;
2. wait once more with a bounded timeout;
3. synthesize completed results and mark unresolved agents;
4. close stuck agents if their output is no longer useful.

If a role/model override is rejected:

1. retry with a fresh prompt and no full-context fork;
2. omit per-call model/effort if a custom role pins them;
3. fall back to built-in `explorer`, `worker`, or `default` when needed.
