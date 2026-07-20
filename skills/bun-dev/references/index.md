# References Index

Prefer rules for decisions, references for exact commands and API detail.

Verified version pin:

- Bun CLI `1.3.14`
- Bun release `v1.3.14`

Refresh vendor-backed refs:

```bash
codex-dev --json bun references status
codex-dev --json bun references plan
codex-dev --json bun references sync
```

## Bun

- Bun release notes snapshot:
  - `ref-bun-release-notes-latest.md`
- Bun capability map:
  - `ref-bun-capabilities-latest.md`
- Bun CLI reference:
  - `ref-bun-cli-cheatsheet.md`
- Bun runtime APIs reference:
  - `ref-bun-builtins-cheatsheet.md`
- Package-manager fallback lanes:
  - `ref-bun-package-manager-fallbacks.md`

## Vercel

- Bun runtime docs:
  - `ref-vercel-bun-runtime.md`

## Fast Lookup

```bash
rg -n "Bun.WebView|markdown\\.ansi|Bun\\.cron|availableParallelism|stripANSI" ~/.agents/skills/bun-dev/references/ref-bun-capabilities-latest.md
rg -n "bun (install|add|update|audit|outdated|pm|build|test|run)" ~/.agents/skills/bun-dev/references/ref-bun-cli-cheatsheet.md
rg -n "Node runtime|pnpm|npm|Yarn|package manager only" ~/.agents/skills/bun-dev/references/ref-bun-package-manager-fallbacks.md
rg -n "bunVersion|Bun\\.serve|runtime" ~/.agents/skills/bun-dev/references/ref-vercel-bun-runtime.md
```
