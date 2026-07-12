# Bun Platform Reference

The Bun platform tooling from `~/repos/cli/skill-tools` now lives in this
repository. The canonical command surface is the native `codex-dev bun` group.
The standalone `bun-platform` binary remains a temporary compatibility shim;
new automation should not depend on it.
Deleting the legacy repository is explicitly gated by
<https://github.com/BjornMelin/dev-skills/issues/105>.

## Crates

- `crates/bun-platform-core`: shared Bun audit, safe-fix, validation, reference
  sync, state-path, and rule/skill integrity logic.
- `crates/bun-platform`: temporary compatibility binary over
  `bun-platform-core`.
- `crates/codex-dev`: canonical native command surface.

## Compatibility binary JSON contract

`bun-platform` JSON output is command-specific and is emitted with
`--format json`:

```json
[
  {
    "rule_id": "pm-no-mixed-lockfiles",
    "severity": "error",
    "file": "package-lock.json"
  }
]
```

Safe-fix planning and apply reports emit `PlannedFix` records. Each record
includes `before` and `after` content when a package.json rewrite is planned or
applied; there is no `--full-content` flag.

## Commands

```bash
bun-platform audit --root . --format json
bun-platform list-rules
bun-platform explain pm-no-mixed-lockfiles
bun-platform plan-fixes --root . --format json
bun-platform apply-safe-fixes --root . --format json
bun-platform validate --root . --fail-on warn
bun-platform benchmark --root . --format json
bun-platform release-sync --status --format json
bun-platform release-sync --dry-run --format json
bun-platform release-sync
bun-platform doctor --format json
```

Common text-mode examples:

```bash
bun-platform audit --root .
bun-platform plan-fixes --root .
bun-platform apply-safe-fixes --root .
bun-platform validate --root . --fail-on warn
```

Use the `codex-dev bun` command group in skills, docs, and new scripts. Keep the
`bun-platform` command only where an existing caller still requires the
temporary compatibility binary.

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

`skills/bun-dev` is the source of vendor reference snapshots and rule snapshots
used by the platform. `skills/bun-audit` is a router/front-end over the Bun
audit workflow rather than the owner of those snapshots.

Reference sync defaults to the installed global agent skill root
`~/.agents/skills/bun-dev/references/...` unless an explicit `--skill-root` is
passed. Running inside this repository does not automatically target the tracked
`skills/bun-dev`; callers that need repo-local snapshots must pass
`--skill-root skills/bun-dev`.

## Task Capsule Import

External JSON reports can be recorded as capsule evidence:

```bash
codex-dev --json evidence append \
  --capsule .codex/tasks/<task> \
  --kind output \
  --summary "Record bun-platform audit output" \
  --tool bun-platform \
  --artifact /tmp/bun-audit.json
```
