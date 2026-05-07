---
description: Choose the right operating mode for repo modernization: bun-first, pnpm-turbo monorepo, or plan-first review workflow.
---

# Variant Index

Use the base skill by default. Load one of these references first when the repo
shape or user workflow makes the default too broad.

## Available Variants

### `bun-first.md`

Use when:
- Bun is the repo-native package manager
- `bun.lock` is authoritative
- Bun workspaces or Bun runtime/tooling are central to the repo

Bias:
- prefer Bun-native install, update, audit, outdated, and `bun pm` analysis
- prefer Bun docs and release notes as the first package-manager reference

### `pnpm-turbo-monorepo.md`

Use when:
- `pnpm-lock.yaml` is authoritative
- `pnpm-workspace.yaml` or Turbo config defines the repo graph
- task orchestration and workspace boundaries matter as much as dependency bumps

Bias:
- use pnpm for actual dependency changes
- use turbo to validate wave-by-wave and package-by-package impact
- treat workspace boundaries and shared package ripple effects as first-class

### `plan-first.md`

Use when:
- the user expects an explicit plan before edits
- the repo is high-risk or operationally sensitive
- the blast radius is large and staged review matters

Bias:
- findings and upgrade matrix first
- touched-files and risk map before implementation
- execute only after the plan is decision-complete

## Default Choice

If the repo is ambiguous:
1. detect the package manager and workspace graph
2. choose the nearest variant
3. fall back to the base skill if no variant clearly dominates
