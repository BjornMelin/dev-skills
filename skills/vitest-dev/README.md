# vitest-dev (Claude Code skill)

This folder is a self-contained skill package.

## Install

1. Download the `vitest-dev.zip` artifact.
2. Unzip it into your Claude Code skills directory.

Common patterns:

- Project-local: `.claude/skills/vitest-dev/`
- User-global: `~/.claude/skills/vitest-dev/`

(Exact paths depend on your Claude Code setup; place the folder where your skills loader expects skill directories.)

## Contents

- `skill.md` – the skill definition and operating procedure
- `checklists/` – QA checklists for test plan + review + CI readiness
- `references/` – distilled Vitest + Next.js guidance and quick references
- `templates/` – ready-to-copy configs + setup files + utilities
- `scripts/` – CI/local helpers (sharding, merging reports, flake detection)
- `examples/` – example tests for TS, React, and Next.js

## License

MIT (see `LICENSE`).
