# References Index

Prefer rules for decisions and references for exact commands or API details.

Verified version pin:

- Bun CLI `1.3.13`
- Bun release `v1.3.13`

Refresh vendor-backed refs:

```bash
codex-dev --json bun references status
codex-dev --json bun references plan
codex-dev --json bun references sync
```

## Bun

- Latest Bun release notes:
  - `ref-bun-release-notes-latest.md`
- Bun capabilities snapshot:
  - `ref-bun-capabilities-latest.md`
- Bun CLI + workflow cheatsheet:
  - `ref-bun-cli-cheatsheet.md`
- Bun built-in APIs cheatsheet:
  - `ref-bun-builtins-cheatsheet.md`
- Bun package-manager fallback notes:
  - `ref-bun-package-manager-fallbacks.md`

## Vercel

- Bun runtime docs:
  - `ref-vercel-bun-runtime.md`

## Fast Lookup

```bash
rg -n "--parallel|--sequential" skills/bun-dev/references/ref-bun-release-notes-latest.md
rg -n "bunVersion|Bun\\.serve|Beta" skills/bun-dev/references/ref-vercel-bun-runtime.md
rg -n "bun (install|add|update|test|build)" skills/bun-dev/references/ref-bun-cli-cheatsheet.md
```
