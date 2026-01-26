# pytest core (pytest 9.x)

This reference focuses on **pytest itself** (not plugins): configuration,
fixtures, markers, parametrization, subtests, and strictness.

## Table of contents

- Configuration (INI vs TOML)
- Markers and selection
- Fixtures and scoping
- Parametrization (and IDs)
- Subtests (pytest 9.0+)
- Warnings, xfail/skip, and strictness

## Configuration (INI vs TOML)

### Prefer `pyproject.toml`

Legacy (INI-compat) config lives in `pyproject.toml` under:

```toml
[tool.pytest.ini_options]
addopts = "-ra -q"
testpaths = ["tests"]
```

pytest 9.0 adds **native TOML** config under `[tool.pytest]` (native TOML types
instead of INI-compat string parsing):

```toml
[tool.pytest]
minversion = "9.0"
addopts = ["-ra", "-q"]
testpaths = ["tests"]
```

Important:
- `[tool.pytest]` and `[tool.pytest.ini_options]` **cannot** be used together.
- If using a separate config file (`pytest.toml` / `.pytest.toml`), use `[pytest]`.

### High-ROI config keys

- `testpaths`: avoid scanning the whole repo during collection.
- `norecursedirs`: exclude heavy dirs (`.git`, `.venv`, `node_modules`, caches).
- `addopts`: set defaults for CI (e.g., `-ra`, `--strict-markers`).
- `markers`: document all custom markers for strict validation.
- `filterwarnings`: enforce deprecations, silence known-noisy libs.

## Markers and selection

### Define markers (then enforce strictness)

In config:

```toml
[tool.pytest.ini_options]
markers = [
  "unit: fast isolated tests",
  "integration: hits DB/filesystem or other services",
  "system: end-to-end tests",
]
```

Then select:
- `-m "unit"` / `-m "not integration"` for suites.
- Combine: `-m "unit and not slow"`.

Strict marker enforcement:
- `strict_markers = true` (or enable strict mode; see below).

## Fixtures and scoping

### Fixture rules of thumb

- Prefer **function-scoped** fixtures by default.
- Increase scope (`module`/`session`) only when:
  - setup is expensive, and
  - the resource is safe to share, and
  - tests stay isolated (no cross-test leakage).

Avoid anti-patterns:
- giant `autouse=True` fixtures that implicitly mutate global state.
- session fixtures that return mutable objects shared across tests.

### Built-in fixtures you should reach for first

- `tmp_path`: per-test temp dir (safe for parallel runs).
- `monkeypatch`: env vars, module attributes, `sys.path`, etc.
- `capsys` / `capfd`: capture stdout/stderr.
- `caplog`: capture log records for assertions.
- `request`: introspection + dynamic fixture access (`request.getfixturevalue`).

## Parametrization (and IDs)

### Prefer `@pytest.mark.parametrize` for static matrices

Use parametrization when the input matrix is known at collection time.

IDs:
- Provide stable ids (debuggable CI).
- pytest 9 adds `strict_parametrization_ids` to **error** on duplicate ids
  instead of auto-disambiguating.

## Subtests (pytest 9.0+)

Subtests are a good fit when the iteration set is only known **at runtime**
(e.g., scanning files on disk, introspecting plugins, dynamic resources).

Pattern:
- Accept the `subtests` fixture (type `pytest.Subtests`).
- Wrap each case in `with subtests.test(...):`.

Design guidance:
- Subtests complement parametrization; don’t replace parametrization for static
  matrices.
- Keep the per-subtest body small; heavy work should live outside the context.

## Warnings, xfail/skip, and strictness

### Strict mode (pytest 9.0+)

`strict = true` enables:
- `strict_config`
- `strict_markers`
- `strict_parametrization_ids`
- `strict_xfail`

You can override individual strictness options explicitly even when strict is on.

Note: strict mode can enable new options in future pytest releases; only turn it
on if you pin/lock pytest or you want that behavior.

### Deprecations in pytest 9

pytest 9 turns `PytestRemovedIn9Warning` into errors by default; update your
suite/plugins or temporarily silence with a warning filter (as a stopgap only).

### `xfail` discipline

Prefer:
- fix the root cause, or
- quarantine with a clearly-scoped marker + follow-up, or
- `xfail` with a link and a narrow condition.

If using strict xfail (`strict_xfail`), an unexpected pass becomes a failure
(useful to ensure quarantines get removed).

