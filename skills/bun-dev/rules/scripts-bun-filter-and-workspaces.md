# scripts-bun-filter-and-workspaces

## Why

Monorepos need a reliable way to run scripts across packages without `cd` gymnastics or external orchestration tools. Bun supports:
- `--workspaces` to run in all workspace packages
- `--filter` to run in matching packages

## Do

- Use `--workspaces` for “run everywhere”.
- Use `--filter` (or `bun --filter <pattern> <script>`) for subsets.
- Use `--elide-lines 0` to show full output if you need complete logs.

## Don't

- Don’t shell out to multiple terminals for routine multi-package runs.

## Examples

Run `dev` in all workspace packages:

```bash
bun --filter \"*\" dev
```

Run `build` sequentially across all packages:

```bash
bun run --sequential --workspaces build
```

Show full logs:

```bash
bun run --parallel --elide-lines 0 --filter \"packages/*\" test
```

