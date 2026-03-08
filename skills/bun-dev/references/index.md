# References Index

References are snapshots of vendor docs (Markdown-only, main page content). Prefer opening `rules/<rule-id>.md` first.

Refresh snapshots:

```bash
bun ~/.agents/skills/bun-dev/scripts/update-bun-release-notes.ts
bun ~/.agents/skills/bun-dev/scripts/update-vercel-bun-docs.ts
```

## Bun

- Bun v1.3.10 release notes:
  - `ref-bun-release-notes-bun-v1.3.10.md`
- Bun CLI + workflow cheatsheet (skill-authored):
  - `ref-bun-cli-cheatsheet.md`
- Bun built-in APIs cheatsheet (skill-authored):
  - `ref-bun-builtins-cheatsheet.md`

## Vercel

- Bun runtime docs:
  - `ref-vercel-bun-runtime.md`

## Fast Lookup

```bash
rg -n \"--parallel|--sequential\" ~/.agents/skills/bun-dev/references/ref-bun-release-notes-bun-v1.3.10.md
rg -n \"bunVersion|Bun\\.serve|Beta\" ~/.agents/skills/bun-dev/references/ref-vercel-bun-runtime.md
rg -n \"bun (install|add|update|test|build)\" ~/.agents/skills/bun-dev/references/ref-bun-cli-cheatsheet.md
```
