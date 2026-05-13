# Workflow

## Canonical Flow

1. Detect repository shape and runtimes.
2. Extract dependency manifests (`package.json`, `pyproject.toml`).
3. Run manager-appropriate outdated checks.
4. Resolve package metadata and GitHub repo mapping.
5. Select target version via runtime-pinned compatibility policy.
6. Pull GitHub releases/changelog content for current->target window.
7. Analyze breaking/deprecation/feature signals.
8. Emit Markdown + JSON reports.

## Decision Tree

- Need full upgrade intelligence: run `gh_deps_intel.py full`.
- Need complete plan for one package only: run `gh_deps_intel.py package --dependency <name>`.
- Need only inventory/outdated: run `scan`.
- Need only release-window research for one repo: run `gh_release_diff.py`.
- Need compare notes between refs: run `gh_compare_notes.py`.

## Fast vs Safe

- Safe mode default for reliability and secondary-rate-limit avoidance.
- Fast mode only when explicitly requested and acceptable to retry/fallback.

## Single-Dependency Upgrade Workflow

1. Run package mode with explicit dependency selector:
- `python scripts/gh_deps_intel.py package --repo . --out reports --dependency workflow --mode safe`
2. Review:
- `reports/dependency-upgrade-report.md`
- `reports/dependency-upgrade-report.json`
3. Use `Repository Impact Map` section to execute refactors file-by-file.
