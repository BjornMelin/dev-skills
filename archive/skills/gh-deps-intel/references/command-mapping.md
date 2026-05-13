# Command Mapping

## JavaScript/TypeScript

- Bun: `bun outdated` (monorepo: `bun outdated --recursive --filter=*`)
- pnpm: `pnpm outdated -r --format json`
- npm: `npm outdated --json --all`
- yarn classic: `yarn outdated`

## Python

- uv preferred: `uv pip list --outdated --format json --project <repo>`
- fallback: `python3 -m pip list --outdated --format json`

## Rule

Detect package manager/runtime from repo signals first; never hardcode `bun`/`pnpm`/`uv` without detection.
