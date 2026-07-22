# Cookbook: Subagent Fanout

Use subagent fanout when independent research lanes can run in parallel and the
user explicitly asked for subagents or delegation.

## Good Fanout Shape

Start with the planner:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py list-presets
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset research \
  --task "Research current Codex subagent docs" \
  --scope "official OpenAI docs and official GitHub repositories only"
```

The planner emits role metadata, copy-ready spawn prompts, and a synthesis
checklist that repeats the strict wait requirement. Use `--json` when you want
to feed the plan into another tool or script.

Example deep research batch:

- `openai_docs_researcher`: official OpenAI docs lane.
- `github_researcher`: GitHub repository/source/release lane.
- `citation_auditor`: audit final claim/source quality.

Example dependency decision batch:

- `context7_researcher`: docs lane.
- `source_validator`: package source lane.
- `github_researcher`: issues/releases lane.

## Parent Responsibilities

Before spawning:

1. Define exact independent lanes.
2. Provide scope and output format.
3. Pick custom roles.
4. State that the parent will wait for all spawned agents.

After spawning:

1. Immediately wait for all spawned agents.
2. Do not inspect files, browse, edit, or continue local analysis while waiting.
3. Synthesize all results.
4. Account for every subagent.

## Prompt Shape

Example planner output for `openai_docs_researcher`:

```text
Task: Research current official OpenAI docs for Codex custom subagents and summarize only confirmed current behavior.
Scope: Official OpenAI docs and official OpenAI GitHub repositories only.
Mode: read-only; do not edit files, stage changes, or commit.
Wait: parent will wait for all spawned agents before substantive next work.
Role: openai_docs_researcher.
Model: custom-agent pinned (`gpt-5.6-terra`).
Reasoning: custom-agent pinned (`high`).
Return format:
- Status (`complete`, `partial`, or `blocked`)
- Official sources read
- Sources hydrated
- Current findings
- Claims with confidence and source IDs
- Deprecated or changed guidance
- Provider limits
- Privacy notes
- Open questions
- Recommended next verification
- Risks/blockers
```

For edit-capable plans, pass `--mode edit` only with disjoint ownership in
`--scope`, then keep each worker's write surface non-overlapping.

## Synthesis Pattern

```text
Subagent results:
- openai_docs_researcher: ...
- github_researcher: ...
- citation_auditor: ...

Merged claims:
1. Claim text. Sources: ...
2. Claim text. Sources: ...

Conflicts:
- Claim: ...
- Evidence A: ...
- Evidence B: ...
- Resolution: ...

Residual risk:
- ...
```

## Anti-Patterns

- Parent works on the same research while subagents run.
- Two agents search the same source for the same question.
- Edit-capable agents own overlapping files.
- A child spawns another child without explicit user instruction.
- Final answer ignores a timed-out or failed subagent.

## Timeout Handling

If a subagent does not finish:

1. Send one status or unblock message if useful.
2. Wait once more with a bounded timeout.
3. Close it if no longer useful.
4. Mark its lane incomplete in synthesis.
