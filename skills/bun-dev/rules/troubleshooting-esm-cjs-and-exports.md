# troubleshooting-esm-cjs-and-exports

## Why

ESM/CJS boundary issues are a top source of runtime failures when switching runtimes or bundlers: default exports, `require()` vs `import`, and package `exports` conditions.

## Do

- Prefer ESM (`"type": "module"`) for new Bun-first code where feasible.
- When bundling, set an explicit `--target` (`bun` vs `node`) to match production.
- Use `bun build` externalization when a dependency relies on runtime resolution.

## Don't

- Don’t mix `require()` and ESM imports without understanding how your runtime resolves them.
- Don’t assume `node16` TS resolution implies runtime behavior; verify.

## Debugging Moves

- Inspect the failing package’s `package.json` for `exports`.
- Try a minimal reproduction with `bun run` vs `node` to isolate runtime differences.
- If bundling, temporarily mark the dependency as `external` to confirm it’s a bundling issue.

