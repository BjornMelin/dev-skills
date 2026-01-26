# Plugins and helper libraries (selection guide)

Prefer **a small, curated plugin set**. Each plugin adds hooks, potential
incompatibilities, and runtime overhead.

## Table of contents

- Core set (most projects)
- Parallelism and sharding
- Coverage
- Async
- HTTP/network mocking
- Flake reduction
- Performance tooling

## Core set (most projects)

- `pytest-mock`: ergonomic `unittest.mock` usage (`mocker` fixture).
- `pytest-cov`: coverage integration (`--cov`, reports, fail-under).

## Parallelism and sharding

- `pytest-xdist`: parallelize on one machine (`-n auto`) and choose a
  distribution strategy (`--dist`).

Sharding across CI machines:
- Prefer a purpose-built sharding plugin (e.g., `pytest-split`) when you can.
- Otherwise shard by **test files** using historical timings (see this skill’s
  `scripts/junit_split.py`).

## Coverage

- `pytest-cov`:
  - avoid “coverage theater”: focus on meaningful paths and invariants
  - for parallel runs, ensure you combine coverage data correctly (depends on
    your runner/sharding strategy)

## Async

- `pytest-asyncio`:
  - centralize event-loop policy in config/fixtures
  - keep async tests explicit; don’t mix sync/async implicitly via autouse

## HTTP/network mocking

Choose one per stack:

- `pytest-httpx` for `httpx`
- `responses` for `requests` (not pytest-specific but widely used)
- `respx` for `httpx` (alternative)
- `vcrpy` when you intentionally record/replay HTTP (use sparingly; can hide
  bugs and break determinism if recordings drift)

## Flake reduction

- `pytest-timeout`: hard cap on per-test runtime (prevents deadlocks hanging CI).
- `pytest-rerunfailures`: last resort for quarantining flakes while fixing root
  causes (keep reruns small and time-bounded).
- `pytest-randomly`: randomize order and seed RNG to expose hidden coupling.

## Performance tooling

- `pytest-benchmark`: microbenchmarks with a stable fixture and comparisons.
- `pytest-profiling` / `pytest-monitor`: deeper profiling when `--durations`
  isn’t enough.

