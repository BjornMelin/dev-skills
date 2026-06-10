# Bun Platform Reference

The Bun platform tooling from `~/repos/cli/skill-tools` now lives in this
repository. The canonical surface is the native `codex-dev bun` command group,
with `bun-platform` retained as a temporary compatibility shim for one release.
Deleting the legacy repository is explicitly gated by
<https://github.com/BjornMelin/dev-skills/issues/105>.

## Crates

- `crates/bun-platform-core`: shared Bun audit, safe-fix, validation, reference
  sync, state-path, and rule/skill integrity logic.
- `crates/bun-platform`: compatibility binary that calls
  `bun-platform-core`.
- `crates/codex-dev`: native command surface for future automation.

## JSON Contract

All native commands support the global `codex-dev --json` envelope:

```json
{
  "schema": "codex-dev.output.v1",
  "ok": true,
  "command": "bun audit",
  "result": {
    "schema": "codex-dev.bun-audit.v1"
  }
}
```

Fix reports default to hashes and a diff. Full before/after content is emitted
only with `--full-content`.

## Commands

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

Compatibility shim:

```bash
bun-platform audit --root .
bun-platform plan-fixes --root .
bun-platform apply-safe-fixes --root .
bun-platform validate --root . --fail-on warn
```

Prefer `codex-dev bun ...` in skills, docs, and future scripts.

## State And Config

Repository config is still `bun-platform.config.json`. Supported keys:

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

Audit cache writes are disabled by default. Use `--write-cache` only when a
caller explicitly wants reusable scan-cache entries. Safe fixes always write
rollback artifacts under external state before mutating files.

## Skill Integration

`skills/bun-dev` owns rules and vendor reference snapshots. `skills/bun-audit`
is only a router over `codex-dev bun`. Reference sync defaults to the tracked
`skills/bun-dev` when run inside this repository; pass `--skill-root` for an
installed global skill.

## Task Capsule Import

External JSON reports can be recorded as capsule evidence:

```bash
codex-dev --json tool import \
  --capsule .codex/tasks/<task> \
  --tool bun-platform \
  --report /tmp/bun-audit.json \
  --kind output
```
