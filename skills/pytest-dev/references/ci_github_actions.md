# GitHub Actions: pytest throughput patterns

## Table of contents

- Baseline CI command shape
- Dependency caching (uv)
- Test sharding (matrix)
- Artifacts (JUnit, coverage)
- Fail-fast and cancellation

## Baseline CI command shape

Prefer explicit module execution:

```bash
python3 -m pytest -q --junitxml=reports/junit.xml
```

Add coverage only when required (coverage can add overhead):

```bash
python3 -m pytest -q --cov --cov-report=xml:coverage.xml
```

## Dependency caching (uv)

If your project uses `uv`, prefer `astral-sh/setup-uv` with caching enabled.
Keep the cache key tied to your lockfile (e.g., `uv.lock`) for correctness.

Key ideas:
- cache the `uv` download/build cache
- avoid re-resolving dependencies on every run

## Test sharding (matrix)

Sharding gives *horizontal* scaling (multiple runners). Combine with xdist for
*vertical* scaling (multi-core per runner).

Baseline approach:
1. Run the full suite once and persist `reports/junit.xml` as an artifact.
2. Next runs use that JUnit as historical timing input for sharding.

This skill ships `scripts/junit_split.py` which prints the list of files for a
given shard index:

```bash
python3 .codex/skills/pytest-dev/scripts/junit_split.py \
  --junitxml reports/junit.xml \
  --glob 'tests/**/*.py' \
  --groups 4 \
  --index 0
```

Then run just those files:

```bash
python3 .codex/skills/pytest-dev/scripts/run_pytest_filelist.py shard_0.txt \
  -- -q --junitxml=reports/junit.xml
```

Practical tip: shard by file, then use xdist within the shard:
- `python3 -m pytest -n auto --dist loadfile ...`

## Artifacts (JUnit, coverage)

Always upload artifacts when debugging flakes/perf:
- `reports/junit.xml`
- `coverage.xml` / `htmlcov/` (if enabled)

JUnit is also used by many “annotate test failures” Actions.

## Fail-fast and cancellation

CI resource best practices:
- Use GitHub Actions `concurrency` to cancel obsolete runs on the same branch.
- For sharded jobs, set `fail-fast: false` if you want all shards’ failures in
  one run; otherwise keep it `true` for faster feedback.
