# Skill and Subagent Eval Lab

`tools/eval/skill_subagent_eval.py` runs deterministic offline checks for the
full skill catalog, local skill assets, OpenAI agent metadata, subagent
templates, subspawn role contracts, planner presets, and helper-script syntax.
It is deliberately separate from `codex-research eval`, which remains scoped to
research routing, privacy, budgets, evidence, report, and closeout bundle
contracts.

Tracking: #20, #24, and #81.

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

Promote warning findings to failures:

```bash
python3 tools/eval/skill_subagent_eval.py --json --strict
```

## Report Contract

The JSON report uses `skill_eval_report.v1`:

The example below is abridged to one native check; full reports include every
built-in check.

```json
{
  "schema": "skill_eval_report.v1",
  "generated_at": "2026-05-12T05:00:00Z",
  "repo_root": "$REPO",
  "strict": false,
  "ok": true,
  "summary": {
    "checks": 1,
    "passed": 1,
    "warning": 0,
    "failed": 0,
    "timed_out": 0,
    "errors": 0,
    "warnings": 0
  },
  "checks": [
    {
      "id": "openai-agent-metadata",
      "name": "Skill agents/openai.yaml metadata validates",
      "type": "native",
      "severity": "required",
      "runner": "openai_agent_metadata",
      "status": "passed",
      "exit_code": null,
      "duration_ms": 42,
      "findings": [],
      "details": {
        "files": 34,
        "shapes": {
          "interface": 27,
          "direct": 6,
          "legacy": 1
        }
      }
    }
  ]
}
```

Command-backed checks are bounded to 120 seconds each. Timed-out checks return
status `timed_out`, `exit_code: null`, and make the overall report fail. Native
aggregate checks return normalized `findings` and `details`. Warning findings
set a check status to `warning` but keep `ok: true` in default mode; `--strict`
treats warnings as failures. The `repo_root` field is portable and emitted as
`$REPO`. Child Python commands run with an isolated `PYTHONPYCACHEPREFIX` so
compile checks do not write `__pycache__` into the repository. `stdout_tail` and
`stderr_tail` are bounded and replace the absolute repository path with `$REPO`,
making the output suitable for local task-capsule evidence summaries without
committing machine-specific paths.

## Built-In Checks

- `all-skill-frontmatter`
- `tanstack-skill-contracts`
- `readme-catalog-exposure`
- `docs-reference-exposure`
- `skill-local-links`
- `skill-script-syntax`
- `generated-cache-exclusion`
- `dist-package-metadata`
- `openai-agent-metadata`
- `subagent-template-contracts`
- `subspawn-role-contracts`
- `subspawn-research-plan`
- `python-helper-compile`

The lab calls existing owners:

- `tools/skill/quick_validate.py`
- `tools/skill/check_tanstack_skills.py` for TanStack-specific stale-guidance, rule-routing, and authority-ledger contracts
- `skills/subagent-creator/scripts/subagent_creator.py validate`
- `skills/subspawn/scripts/subspawn_plan.py validate-roles`
- `skills/subspawn/scripts/subspawn_plan.py plan`
- `python3 -m compileall`

Native checks enumerate every `skills/*/SKILL.md`, use tracked and untracked non-ignored skill files for
generated-cache and local-link checks, validate supported
`agents/openai.yaml` shapes, and inspect local `.skill` bundles. Ignored local
bundle artifacts are reported as warning findings so stale local build output
does not fail the default offline lab, while `--strict` can still catch them
before publishing bundles.

It does not use network, provider credentials, or live Codex runtime execution.
