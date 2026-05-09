# Skill and Subagent Eval Lab

`tools/eval/skill_subagent_eval.py` runs deterministic offline checks for skill
metadata, subagent templates, subspawn role contracts, planner presets, and
helper-script syntax. It is deliberately separate from `codex-research eval`,
which remains scoped to research routing, privacy, budgets, evidence, and report
contracts.

Tracking: #20 and #24.

## Command

Run the full offline lab:

```bash
python3 tools/eval/skill_subagent_eval.py --json
```

List checks without running them:

```bash
python3 tools/eval/skill_subagent_eval.py --list --json
```

Run one check by id:

```bash
python3 tools/eval/skill_subagent_eval.py \
  --json \
  --check subagent-template-contracts
```

## Report Contract

The JSON report uses `dev-skills.skill-subagent-eval.v1`:

```json
{
  "schema": "dev-skills.skill-subagent-eval.v1",
  "generated_at": "2026-05-09T05:00:00Z",
  "repo_root": "$REPO",
  "ok": true,
  "checks": [
    {
      "id": "subspawn-role-contracts",
      "name": "Subspawn role contracts validate",
      "command": [
        "python3",
        "skills/subspawn/scripts/subspawn_plan.py",
        "validate-roles",
        "--json"
      ],
      "status": "passed",
      "exit_code": 0,
      "duration_ms": 42,
      "timeout_seconds": 120,
      "stdout_tail": "...",
      "stderr_tail": ""
    }
  ]
}
```

Checks are bounded to 120 seconds each. Timed-out checks return status
`timed_out`, `exit_code: null`, and make the overall report fail. The `repo_root`
field is portable and emitted as `$REPO`. Child Python commands run with an
isolated `PYTHONPYCACHEPREFIX` so compile checks do not write `__pycache__` into
the repository. `stdout_tail` and `stderr_tail` are bounded and replace the
absolute repository path with `$REPO`, making the output suitable for local
task-capsule evidence summaries without committing machine-specific paths.

## Built-In Checks

- `skill-metadata-deep-researcher`
- `skill-metadata-subagent-creator`
- `skill-metadata-subspawn`
- `subagent-template-contracts`
- `subspawn-role-contracts`
- `subspawn-research-plan`
- `python-helper-compile`

The lab calls existing owners:

- `tools/skill/quick_validate.py`
- `skills/subagent-creator/scripts/subagent_creator.py validate`
- `skills/subspawn/scripts/subspawn_plan.py validate-roles`
- `skills/subspawn/scripts/subspawn_plan.py plan`
- `python3 -m compileall`

It does not use network, provider credentials, or live Codex runtime execution.
