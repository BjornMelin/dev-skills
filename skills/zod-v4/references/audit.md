# Audit Script (Zod v4) - Guide

The bundled audit script is a fast, report-only scanner for Zod v3/v4 deprecated patterns and common v4 footguns.

## Run

From the repo you want to scan:

```bash
bun "$skill_dir/scripts/zod-audit.ts" --root . --format text
```

Useful flags:

- `--format text|json|md`
- `--fail-on error|warn|info` (exit code 1 if any findings at/above threshold)
- `--include-exts ts,tsx,js,jsx`
- `--exclude-dirs node_modules,.next,dist`
- `--explain <ruleId>` (print the linked rule doc)

## What It Flags (High Level)

- v3-only APIs: `z.nativeEnum`, `z.promise`
- Deprecated v4 usage: `z.formatError`, `err.format()`, `err.flatten()`
- Common migration diffs: `z.record(valueSchema)`, `obj.merge(...)`, `z.string().email()`
- Known v4 footguns: `z.url({ protocols: [...] })` (ignored), `z.email({ pattern: "..." })` (pattern must be RegExp)
- Object constructor differences: `.strict()`/`.passthrough()`/`.strip()`

## How To Fix

Each finding includes a `ruleId` that maps to a file in `rules/`.
Open that rule file and apply the replacement pattern.

If you want a quick explanation for a particular finding:

```bash
bun "$skill_dir/scripts/zod-audit.ts" --explain migrate-top-level-string-formats
```
