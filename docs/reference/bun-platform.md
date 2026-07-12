# Bun platform reference

The Bun platform tooling from `~/repos/cli/skill-tools` now lives in this
repository. Use the native `codex-dev bun` group for audits, fixes, validation,
reference sync, and diagnostics. Deleting the legacy repository remains gated
by [issue #105](https://github.com/BjornMelin/dev-skills/issues/105).
Previously installed `bun-platform` executables are unsupported workstation
residue; removing them remains an explicit local-deletion approval step.

## Crates

- `crates/bun-platform-core`: shared Bun audit, safe-fix, validation, reference
  sync, state-path, and skill-integrity logic.
- `crates/codex-dev`: canonical command surface.

## JSON contract

Place the global `--json` flag before `bun`. Native results use a versioned
result schema inside the standard `codex-dev.output.v1` envelope:

```json
{
  "schema": "codex-dev.output.v1",
  "ok": true,
  "command": "bun audit",
  "result": {
    "schema": "codex-dev.bun-audit.v1",
    "finding_count": 0,
    "findings": []
  }
}
```

Safe-fix reports include hashes and diffs by default. Pass `--full-content` to
include complete before-and-after content.

## Commands

Use these commands from an installed `codex-dev` binary:

```bash
codex-dev --json bun audit --root .
codex-dev --json bun rules list
codex-dev --json bun rules show pm-no-mixed-lockfiles
codex-dev --json bun fixes plan --root .
codex-dev --json bun fixes apply --root .
codex-dev --json bun validate plan --root .
codex-dev --json bun validate run --root . --fail-on warn
codex-dev --json bun benchmark --root .
codex-dev --json bun references status
codex-dev --json bun references plan
codex-dev --json bun references sync
codex-dev --json bun doctor
```

## State and config

Repository config remains `bun-platform.config.json`. Supported keys:

- `disabledRules`
- `severityOverrides`
- `adapters`
- `includePaths`
- `excludeDirs`
- `baseline`
- `maxFiles`
- `maxBytes`
- `validationCommands`
- `writeCache`

External paths use the dev-skills namespace:

- config: `${XDG_CONFIG_HOME:-~/.config}/dev-skills/bun-platform`
- state: `${XDG_STATE_HOME:-~/.local/state}/dev-skills/bun-platform`
- cache: `${XDG_CACHE_HOME:-~/.cache}/dev-skills/bun-platform`

Audit cache writes are disabled by default. Pass `--write-cache` when you need
reusable scan-cache entries. Safe fixes write rollback artifacts under external
state before changing files.
Benchmark runs always suppress cache writes, even when configuration requests
them, so measured timings exclude persistent-cache side effects. The benchmark
command intentionally does not accept `--write-cache`.

## Skill integration

`skills/bun-dev` owns vendor references and rule snapshots. `skills/bun-audit`
routes audit work to the native command surface.

Inside this repository, reference commands discover the tracked
`skills/bun-dev` directory. Outside the repository, they use
`BUN_PLATFORM_SKILL_ROOT` or `~/.agents/skills/bun-dev`. Pass `--skill-root` to
override discovery.

## Task capsule import

Record a Bun audit report as task-capsule evidence:

```bash
codex-dev --json bun audit --root . > /tmp/bun-audit.json
codex-dev --json tool import \
  --capsule .codex/tasks/task_id \
  --tool codex-dev-bun \
  --report /tmp/bun-audit.json \
  --kind output
```
