# Subspawn Reference

Path:

```text
skills/subspawn/
```

Purpose: bounded subagent delegation and synthesis policy for Codex sessions.

Template ownership and duplicate-role handling are defined in
[Subagent Templates](subagent-templates.md). This reference covers the
delegation policy and planner CLI.

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
Prefer 1-3 roles; use 4-6 only for independent read, test, or audit lanes with
one writer at most.
In a full repository checkout, the planner loads the deeper research and
subagent template directories first. In a packaged standalone `subspawn` skill,
it falls back to the local `skills/subspawn/templates/agents/` copies so preset
plans remain usable without sibling skills.

If `validate-roles` prints "duplicate role templates ignored" for known
research, creator-pack, or subspawn fallback roles, treat that as an audit trail,
not a failure. Investigate it as drift only when the duplicate falls outside the
authority model or changes the canonical role contract.

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
- Evidence or role-specific source headings
- Files inspected/changed, queries run, or sources hydrated
- Commands run or provider calls
- Findings or claims with confidence and source IDs
- Risks/blockers
```

Template roles may emit narrower return headings from their TOML
`developer_instructions`; built-in roles use the generic minimum shown above.

## Model and Effort

Default posture:

- inherit model/effort when role files already pin them;
- use Terra `medium` for mechanical reads and Terra `high` for bounded source
  gathering;
- use Sol `medium` for default judgment and implementation, and Sol `high` for
  planning, architecture, security, root-cause work, and synthesis;
- reserve Terra `max` for independent adversarial validation;
- do not use routine Sol `xhigh`, `max`, or `ultra`, and keep Luna outside V2
  until native custom-agent support is verified;
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

## codex-dev Capsule Bridge

`subspawn` remains the planning and delegation-policy owner. `codex-dev`
records local capsule evidence only; it must not spawn, wait on, retry, or
semantically interpret agent output on its own. It may derive mechanical batch
status and `orchestration_run.v1` completion diagnostics from recorded agent
statuses so the capsule can be scanned quickly.

Record a planned batch after generating planner JSON:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset review \
  --task "pre-PR branch review" \
  --scope "current branch diff" \
  --json > /tmp/pre-pr-review-plan.json

cargo run -q -p codex-dev -- --json subagents record-plan \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --source /tmp/pre-pr-review-plan.json \
  --command "python3 skills/subspawn/scripts/subspawn_plan.py plan --preset review --json"
```

For operator-facing verification, prefer the equivalent `orchestration`
commands. They still write only `subagents.json` and `evidence.jsonl`, but they
return a stable `orchestration_run.v1` report with expected roles, runtime agent
IDs, wait status, stale evidence warnings, and completion coverage.

Then record agent outcomes as the parent session verifies them:

```bash
cargo run -q -p codex-dev -- --json subagents record-outcome \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --role reviewer \
  --status completed \
  --summary "no blocking findings" \
  --disposition accepted \
  --human-verified \
  --source-id reviewer:1 \
  --artifact review-notes.md
```

Finally record the parent synthesis:

```bash
cargo run -q -p codex-dev -- --json subagents record-synthesis \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --status completed \
  --summary "review batch clean after follow-up fixes" \
  --human-verified \
  --source-id synthesis:pre-pr-review \
  --artifact review-summary.md
```

The bridge stores role names, duplicate-template warnings, registry issues,
stable prompt IDs, SHA-256 prompt hashes, human-verified dispositions, and
short summaries. Registry issues are preserved on the batch and emitted as
`registry_issue` warnings in `orchestration_run.v1`. Completed synthesis requires
every planned role to have a
terminal status, `human_verified: true`, and a final disposition of `accepted`,
`rejected`, `mixed`, or `informational`; `pending` is not final. The bridge does
not store raw prompt text in `subagents.json` and does not store raw long
transcripts by default.
