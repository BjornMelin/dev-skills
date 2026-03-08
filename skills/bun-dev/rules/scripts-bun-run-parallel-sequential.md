# scripts-bun-run-parallel-sequential

## Why

Running multiple scripts (or workspace scripts) is a common need in monorepos and CI. Bun supports Foreman-style execution with:
- `--parallel` for concurrent runs
- `--sequential` for ordered runs

These flags are available in Bun v1.3.10+.

This reduces the need for extra dependencies like `concurrently` for simple cases.

## Do

- Use `bun run --parallel ...` for concurrent scripts.
- Use `bun run --sequential ...` when order matters.
- Combine with `--workspaces` or `--filter` for monorepos.
- Use `--no-exit-on-error` when you need a “best effort” run.

## Don't

- Don’t introduce additional process supervisors until Bun’s native options are insufficient.

## Examples

Run multiple scripts in one package:

```bash
bun run --parallel lint typecheck
```

Run a script across all workspace packages concurrently:

```bash
bun run --parallel --workspaces test
```

Run across a subset of packages:

```bash
bun run --parallel --filter \"packages/*\" build
```

Continue even if one fails:

```bash
bun run --parallel --no-exit-on-error --workspaces test
```
