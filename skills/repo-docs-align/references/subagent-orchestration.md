# Subagent Orchestration

Use this file when `repo-docs-align` needs subagents for evidence gathering, repo mapping, or verification.

## Purpose

Subagents improve this skill when they stay bounded and evidence-focused. They should reduce search time and surface conflicts, not take over authority decisions or final synthesis.

## Default policy

- Main agent owns:
  - overall plan quality
  - canonical authority decisions
  - final synthesis
  - default doc edits
- Subagents own:
  - exploration
  - grep/path/file audits
  - doc or API verification
  - focused review
  - validation triage
  - narrowly scoped drafting or implementation only when explicitly needed

## Fan-out limits

- Prefer `1-3` focused subagents.
- Avoid broad fan-out.
- Do not spawn nested subagents unless the user explicitly asks.

## Budget-optimized model policy

Default model ladder:
- first choice: `gpt-5.4-mini`
- larger fallback: `gpt-5.3-codex`
- near-instant text-only triage: `gpt-5.3-codex-spark`

### `gpt-5.4-mini`

- default effort: `medium`
- use `medium` for most repo mapping, read-heavy scans, codebase tracing, focused audits, doc/API verification, standard review passes, and normal bounded implementation
- use `high` for ambiguous findings, cross-file reasoning, conflicting evidence, tricky debugging, security-sensitive inspection, or critical logic-path review
- use `low` only for simple deterministic tasks such as path lookup, quick grep confirmation, tiny file checks, surface inventory, or basic doc lookup

Prefer `medium` over `low` by default. Before moving from `medium` to `high`, tighten the task statement, expected output, and verification requirements first.

### `gpt-5.3-codex`

Use only when `gpt-5.4-mini` is clearly underfitting after one tighter pass, or when the task is cross-cutting, ambiguous, or high-risk.

- use `low` for straightforward implementation
- use `medium` for normal non-trivial work
- use `high` only when genuinely necessary

Do not use `xhigh` by default.

### `gpt-5.3-codex-spark`

Use only for near-instant, text-only drafting or triage. Do not use Spark for code changes, final decisions, or high-risk reasoning.

## Escalation rules

Before raising model or effort, first tighten:
- task statement
- output contract
- verification requirements

Escalate only the specific subagent that is underfitting. Do not escalate the whole session because one worker struggles.

## Wait policy

Default posture:
- spawn bounded explorers
- keep the main agent doing non-overlapping local work
- wait at synthesis gates before major authority decisions, final recommendations, or edits that depend on delegated evidence

Only wait immediately when the very next step is blocked on the delegated result.

## Mandatory spawn checklist

Every `spawn_agent` call should include:
- exact task or question
- allowed scope or surfaces
- read-only vs may-edit status
- whether the main agent should wait now or continue until a synthesis gate
- exact return format
- explicit `(model, reasoning_effort)`
- one-line reason when using a non-default model or effort

## Return contract

Require this exact shape or a close equivalent:

```text
Key finding/result:
Files/symbols inspected:
Commands/checks run:
Recommended next action:
Unresolved questions/risks:
```

Reject vague summaries without evidence.

## Prompt template

Use a prompt like:

```text
Task: <one narrow question>
Scope: <allowed files, docs, APIs, or repo surfaces only>
Mode: <read-only | may edit>
Wait policy: <main agent waits now | main agent continues until synthesis gate>
Return format:
- Key finding/result
- Files/symbols inspected
- Commands/checks run
- Recommended next action
- Unresolved questions/risks
Model/effort reason: <only when non-default>
```

## Good fits for this skill

- explorer 1: map `AGENTS.md` and scoped guidance drift
- explorer 2: map README/ADR/spec/runbook ownership and stale surfaces
- explorer 3: verify external docs or APIs when repo evidence is insufficient, using built-in `web.search_query`, `web.open`, `web.find`, `web.click`, and related `web.*` tools as needed

## Bad fits for this skill

- broad multi-agent doc rewrites before the authority path is chosen
- nested fan-out
- vague “analyze the repo” prompts
- handing final decisions to a worker when the main agent is not blocked
