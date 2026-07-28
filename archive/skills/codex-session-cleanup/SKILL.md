---
name: codex-session-cleanup
description: "Safely scan, triage, quarantine, restore, and purge Codex session/history/state/memory artifacts for current repo, explicit roots, subrepos, Codex-home, or all sessions."
---

# Codex Session Cleanup

Use this explicit-only skill for Codex artifact hygiene from any directory.
Default to a read-only scan that produces a JSON manifest and Markdown report.
Quarantine is reversible cleanup; purge is irreversible cleanup.

## Safety Contract

- Never delete directly. Quarantine first, purge later.
- Never mutate during `scan`.
- Never clean all sessions unless the user explicitly asks for all sessions.
- Never move, ignore, rewrite, or delete active memory by default. Copy memory
  summaries and the memory registry into quarantine when cleanup runs.
- Never inspect or mutate `auth.json`, `internal_storage/**`, or secret-like
  env files unless the user explicitly requests a separate security-reviewed
  workflow.
- Stop on SQLite integrity failures, JSONL parse errors, checksum mismatch,
  ambiguous thread IDs, active/recent session risk, active jobs/goals, child
  spawn risk, stale manifests, or missing backups.

## First-Pass Workflow

1. Load only the references needed:
   - `references/artifact-model.md` for Codex file and DB layout.
   - `references/classifier-rules.md` for confidence, scope, and memory triage.
   - `references/safety-restore.md` before `apply`, `restore`, or `purge`.
   - `references/report-template.md` when interpreting output.
2. Run a scan first. From a repo, this scans the current Git root and audits
   linked memory without mutating anything:

```bash
python3 "$skill_dir/scripts/codex_session_cleanup.py" scan
```

3. Review the report sections for scope roots, selected threads, medium
   candidates, protected exclusions, memory triage, artifact families, and
   automation eligibility.
4. Use existing read-only subagents only when the report is ambiguous or broad:
   `repo_explorer` for scope sanity, `false_positive_validator` for candidate
   risk, and `history_reviewer` for durable-memory review.
5. Apply only with the exact manifest id and `--execute`; keep memory copy-first:

```bash
python3 "$skill_dir/scripts/codex_session_cleanup.py" apply \
  --manifest /path/to/manifest.json \
  --confirm <manifest-id> \
  --memory-policy copy \
  --execute
```

6. Rerun `scan` after cleanup and report counts, quarantine path, DB integrity,
   checksum status, history/index rows removed, session files quarantined,
   memory files copied/moved, automation blockers, and remaining risk.

## Scopes

- Current repo or directory: `scan`
- Explicit root: `scan --scope root --root /path/to/repo`
- Multiple roots: `scan --scope roots --root /repo/a --root /repo/b`
- Roots file: `scan --scope roots --roots-file /path/to/roots.txt`
- Repositories under cwd: `scan --scope cwd-subrepos --cwd /path/to/parent`
- Codex-home artifacts only: `scan --scope codex-home`
- All Codex sessions/artifacts: `scan --scope all`

Nested repositories use longest-root-wins ownership unless
`--include-parent-overlap` is explicit.

## Artifact Families

Default scan families are `sessions` plus `memory`. Add repeatable
`--artifact-family` values when needed:

- `sessions`: session JSONL, history/index JSONL, state/log SQLite rows.
- `memory`: `memories/MEMORY.md` and linked rollout summaries, copy-first.
- `quarantine`: old quarantine bundles, purge remains manual.
- `logs`: Codex logs and log directories, report-only by default.
- `cache`: cache and temp directories, manual candidate only.
- `generated`: generated images, manual candidate only.
- `skills-agents-config`: skills, agents, AGENTS/config files, report-only.

## Autonomous Apply Policy

Autonomous apply is allowed only for reversible quarantine of high-confidence
current-repo disposable session artifacts when scan evidence and read-only
validator consensus agree false-positive risk is low. Autonomous apply must use
`--memory-policy copy`; active memory files must not be moved, deleted, or
rewritten.

Manual review is required for medium-confidence candidates, broad all-session
sweeps, active/recent sessions, ambiguous IDs, parse or integrity issues,
durable linked memory, destructive memory mutation, and purge.

## Memory Policy

Treat stale, outdated, or conflicting memory as a live-verification queue, not
as proof that the memory should be deleted. Preserve durable workflow knowledge
and mark drift-prone details in the report. Current repo code/docs and live
provider or GitHub state outrank historical memory when they conflict.

Destructive memory modes require an extra manifest-id confirmation:

```bash
python3 .../codex_session_cleanup.py apply ... \
  --memory-policy move \
  --confirm-memory-move <manifest-id> \
  --execute
```

## Implementation Notes

The script is the source of truth for deterministic scanning and mutation. Use
the references for policy and interpretation, not for hand-written cleanup
commands. Prefer adding classifier or artifact-family behavior to the script
over ad hoc shell filters.

Validate after changes:

```bash
python3 -m py_compile "$skill_dir/scripts/codex_session_cleanup.py"
python3 -m unittest discover -s "$skill_dir/tests"
python3 "<skill-creator-dir>/scripts/quick_validate.py" "$skill_dir"
node "<skill-auditor-dir>/scripts/audit-skills-baseline.mjs" "<skills-root>" /tmp/codex-session-cleanup-skill-audit
```
