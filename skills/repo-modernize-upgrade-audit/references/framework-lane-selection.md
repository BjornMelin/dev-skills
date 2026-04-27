---
description: Framework lane selection for dependency modernization. Use this to decide when to route into Expo, Convex, Next.js, Turborepo, and Vercel-specific upgrade flows.
---

# Framework Lane Selection

Use this reference when the repo contains one or more major framework lanes and
the upgrade/refactor plan needs explicit routing.

## Purpose

The base modernization skill already includes conditional framework routing.
This file makes that routing operational:

- which framework lanes to activate
- what evidence to gather first
- which tools and skills to prioritize
- what cleanup and verification patterns are framework-specific

## Detection Order

Detect the repo in this order before loading heavier framework-specific flows:

1. package manager and workspace graph
2. app/runtime boundaries
3. framework markers in manifests and config files
4. CI/build scripts that reveal actual execution paths

Common markers:

- Expo:
  - `expo` dependency
  - `app.json` / `app.config.*`
  - `eas.json`
  - Expo Router routes
- Convex:
  - `convex/` directory
  - `convex.config.*`
  - generated Convex artifacts
  - `convex` dependencies/scripts
- Next.js:
  - `next` dependency
  - `next.config.*`
  - App Router or Pages Router files
- Turborepo:
  - `turbo.json`
  - root scripts calling `turbo`
- Vercel:
  - `vercel` dependency
  - Vercel deployment scripts/config patterns
  - platform coupling in docs or CI

## Expo Lane

Activate when Expo or React Native is a primary app/runtime lane.

Use:
- `$expo:upgrading-expo`
- official Expo changelogs and migration docs
- Expo CLI, EAS CLI, `expo-doctor`

Priorities:
- align Expo SDK, React, and React Native correctly
- use Expo-native upgrade flows, not ad hoc semver bumps
- replace deprecated Expo packages with current canonical replacements
- verify native/runtime implications, not just typecheck/build

Cleanup bias:
- remove obsolete Expo config workarounds
- delete packages/config files made redundant by newer Expo behavior
- collapse custom wrappers where Expo now provides the feature directly

Verification bias:
- `expo-doctor`
- repo-native mobile validation
- EAS/build checks if relevant

## Convex Lane

Activate when Convex is the backend or data-contract authority.

Use:
- `$convex-audit`
- official Convex docs and current guidance

Priorities:
- schema as truth
- current validator and function patterns
- current query/index/error-handling conventions
- remove outdated or non-canonical Convex usage

Cleanup bias:
- delete non-canonical wrappers around Convex primitives
- remove outdated query patterns and legacy runtime assumptions
- keep one owner for schema and contract logic

Verification bias:
- repo-native Convex lint/type/test/release checks
- verify function signatures, validators, indexes, and call sites stay aligned

## Next.js Lane

Activate when Next.js is a primary web/runtime lane.

Use:
- `$vercel:nextjs`
- official Next.js versioned upgrade guides
- official codemods and migration docs

Priorities:
- detect current major version and required upgrade steps
- use codemods before manual cleanup when they materially reduce risk
- migrate to current canonical APIs and config patterns
- align React and related type packages correctly

Cleanup bias:
- remove deprecated Next APIs and legacy config flags
- remove stale workaround code that newer Next now handles natively
- keep one canonical rendering/data path per surface

Verification bias:
- build
- route/type generation where applicable
- web runtime validation for changed surfaces

## Turborepo Lane

Activate when Turbo defines execution, caching, and task orchestration.

Use:
- `turbo` CLI
- official Turbo and Vercel docs

Priorities:
- map task graph before upgrading shared packages
- use filtered validation to reduce blast radius during upgrade waves
- preserve coherent cache/task boundaries while simplifying scripts

Cleanup bias:
- delete duplicated orchestration scripts if Turbo already owns the concern
- simplify task wiring after dependency and package cleanup

Verification bias:
- filtered package/app validation first
- full graph validation after targeted fixes

## Vercel Lane

Activate when Vercel tooling or platform behavior is materially part of the
repo’s runtime or delivery model.

Use:
- Vercel plugin/tools
- Vercel CLI where appropriate
- official Vercel docs and release notes

Priorities:
- verify runtime/platform assumptions before config edits
- modernize Vercel-related package and config usage
- remove stale platform workarounds when official support now exists

Cleanup bias:
- delete redundant deployment/config indirection that newer Vercel tooling
  makes unnecessary
- remove dead builders/adapters if current platform lanes supersede them

Verification bias:
- repo-native deployment/build validation
- Vercel-specific checks only where the repo actually depends on them

## Multiple Lanes

If the repo has multiple lanes, do not mix them blindly.

Handle them in this order:
1. package manager and workspace truth
2. shared libraries and cross-cutting contracts
3. backend/data-contract lane
4. web/native app lanes
5. platform/deployment lane

Examples:
- Expo + Convex:
  - settle shared contracts and backend boundaries first
  - then handle Expo SDK and app-facing migrations
- Next.js + Vercel:
  - settle Next upgrade path first
  - then clean up Vercel-specific config/workarounds
- pnpm + Turbo + Next + Convex:
  - start with workspace graph and shared packages
  - then backend contracts
  - then app/runtime migrations

## Rule

Framework-specific skills should sharpen the lane, not replace the core
modernization discipline:

- dependency audit still matters
- hard-cut still applies
- clean-code still applies
- reducing-entropy still applies

Do not let framework-specific migrations become an excuse to preserve obsolete
codepaths or avoid deletions.
