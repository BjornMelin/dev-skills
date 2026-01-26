# Performance and flake reduction playbook

## Table of contents

- Measure first
- Collection speed
- Runtime speed
- xdist strategy selection
- CI sharding
- Flakiness checklist

## Measure first

Establish a baseline before changing anything:

- Whole suite: run the canonical CI command locally once.
- Slowest tests: `python3 -m pytest --durations=20 --durations-min=0.5`

Separate the problem:
- **Collection time** (imports, discovery, plugin overhead)
- **Runtime** (fixtures, I/O, algorithmic cost)

## Collection speed

High-ROI fixes:

- Set `testpaths` to avoid scanning the repo.
- Add `norecursedirs` for non-test directories with lots of files.
- Move heavy imports behind runtime boundaries (lazy import inside functions).
- Disable unneeded plugins (built-in or third-party) for the suite you’re running.

Debugging collection overhead:
- Use Python import timing: `python3 -X importtime -m pytest ...` (noisy but useful).

## Runtime speed

Fixture optimization:
- Keep fixtures small; avoid “do everything” fixtures.
- Use narrow scopes by default; increase scope only when safe.
- Avoid per-test DB schema creation; isolate per-worker, not per-test.

Avoid sleeps:
- Replace `time.sleep()` with polling + a timeout.
- Prefer fake clocks when testing time-based logic.

Mock slow boundaries:
- network calls
- filesystem or external CLI calls
- slow cryptography/compression

## xdist strategy selection

Start with:
- `-n auto --dist load` for broad suites with many independent tests.

Consider:
- `--dist loadfile` when tests in the same file share expensive setup that is
  safe to reuse per worker.
- `--dist loadscope` to keep classes/modules together (helps expensive fixtures).
- `--dist worksteal` when you have a few very slow tests and lots of fast ones.

Common xdist gotchas:
- Each worker performs full collection (collection is multiplied).
- Global resources must be namespaced per worker (DB names, ports, tmp dirs).

## CI sharding

If the suite still exceeds your CI budget:

1. Emit JUnit XML in CI: `--junitxml=reports/junit.xml`
2. Use historical timings to split files across a matrix:
   - This skill’s `scripts/junit_split.py` can shard by file.
3. Inside each shard, optionally use xdist for per-runner parallelism:
   - `--dist loadfile` complements file-level sharding well.

## Flakiness checklist

Flakes usually come from:
- time (real clock, race conditions)
- random (non-seeded generators)
- order dependence (shared global state)
- async concurrency (tasks not awaited, event loop leakage)
- shared external resources (ports, tmp dirs, DBs)

Mitigations:
- Make shared resources per-worker (xdist) and per-test (`tmp_path`) as needed.
- Enforce timeouts (`pytest-timeout`) to prevent hangs.
- Randomize order (`pytest-randomly`) to expose hidden coupling (then fix).
