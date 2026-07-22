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

## Model policy

- Terra `medium`: deterministic mapping and inventory.
- Terra `high`: bounded repo, docs, GitHub, and source retrieval.
- Sol `medium`: default review, evidence adjudication, and narrow implementation.
- Sol `high`: planning, architecture, security, root-cause work, and synthesis.
- Terra `max`: independent adversarial validation only.

Do not use routine Sol `xhigh`, `max`, or `ultra`. Keep Luna outside V2 until
native custom-agent support is verified. Prefer role-file pins over per-spawn
overrides; pinned custom roles use `fork_turns = "none"`.

## Escalation rules

Before raising model or effort, first tighten:
- task statement
- output contract
- verification requirements

Escalate only the specific subagent that is underfitting. Do not escalate the whole session because one worker struggles.

## Wait policy

Default posture:
- spawn one bounded batch
- immediately wait for every spawned subagent
- synthesize only after the batch completes

## Mandatory spawn checklist

Every `spawn_agent` call should include:
- exact task or question
- allowed scope or surfaces
- read-only vs may-edit status
- strict wait expectation
- exact return format
- explicit `(model, reasoning_effort)` only when overriding inherited or
  role-pinned defaults
- one-line reason for any model or effort override

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
Wait policy: parent immediately waits for the full batch
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
