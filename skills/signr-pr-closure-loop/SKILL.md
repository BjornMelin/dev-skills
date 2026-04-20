---
name: signr-pr-closure-loop
description: Signr PR remediation + closure operator for GitHub PRs—review-thread fixes, CI repair, Expo/EAS or Vercel/Turborepo debug, repo docs alignment, scoped conventional commits, push, post-push babysit until merge-ready. Use when Signr PR blocked by review comments, CodeRabbit, failing checks, EAS/Vercel workflows, or docs drift; e.g. “fix PR 148 end-to-end”.
---

# Signr PR Closure Loop

Repo-specific orchestrator for Signr PR closure.

Start from live GitHub + repo state. Do not trust stale prompt claims about failing checks, pending comments, or review decisions.

Read only what you need:

- `references/tool-routing.md`
- `references/closure-contract.md`

## Core Contract

- Read root + scoped `AGENTS.md` first.
- Resolve target PR, current head SHA, changed files, required checks, review state, unresolved hosted threads/comments, mergeability before editing.
- Treat live hosted state as truth. Skip stale findings already fixed on current head.
- Prefer existing skills, plugins, CLIs, repo-native commands over rebuilding side systems.
- Apply hard-cut, reducing-entropy, clean-code defaults when changing behavior: one canonical path, delete obsolete code, avoid fallback layers.
- Use official current docs, release notes, changelogs, CLI help when platform behavior may have drifted.
- Update impacted authority docs same pass before final closeout.
- Commit scoped conventional commits, push, monitor until PR merge-ready or real blocker needs user help.
- Do not merge PR. Stop at ready-to-merge + concise summary.

## Workflow

### 1. Normalize The Target

1. Resolve PR from explicit input or current branch.
2. Snapshot:
   - PR metadata + changed files
   - current checks on head SHA
   - reviews, review threads/comments, mergeability
   - local worktree cleanliness
3. If worktree has unrelated dirty changes, stop + ask user.

### 2. Build The Blocker Map

Split work into blocker classes:

- unresolved review feedback
- failing or missing hosted checks
- docs drift from branch changes
- mergeability or branch-state blockers

Use hosted unresolved-thread state as closure truth. Do not rely on `reviewDecision` alone.

### 3. Route Each Blocker To The Right Lane

Use `references/tool-routing.md`.

Default routing:

- review feedback + review bundles -> `$gh-pr-review-fix` plus `review-pack`
- failing GitHub Actions checks -> `$github:gh-fix-ci`
- Expo/EAS workflows, builds, or validation -> `$expo:expo-cicd-workflows`,
  `$expo:upgrading-expo`, Expo MCP when available, EAS CLI
- Vercel or Turborepo issues -> `$vercel` plugin capabilities,
  `$vercel:turborepo`, Vercel CLI, Turbo CLI
- Bun, package-manager, or runtime policy issues -> `$bun-dev`
- repo-wide docs drift -> `$repo-docs-align` if installed, else `$docs-align`,
  else direct docs sweep
- post-push monitoring -> `$babysit-pr`
- scoped commits -> `$commit`

If named skill unavailable, continue with closest repo-native fallback.

### 4. Implement Minimal Canonical Fixes

1. Fix highest-signal blocker cluster first.
2. Reproduce CI-specific failures locally with repo-native command matching hosted lane.
3. Delete obsolete or parallel code paths made unnecessary by fix.
4. Update tests, fixtures, snapshots to canonical shape only.
5. Avoid broad refactors not tied to active blocker.

### 5. Verify Before Publishing

1. Run smallest relevant local gates first.
2. Run branch-relevant surface checks from repo guidance + failing workflow.
3. Before finalizing fix set, run `bun run validate:local:agent`.
4. If docs or architecture contracts changed, run `bun run docs:arch:validate`.
5. Record exact commands + outcomes for final summary.

### 6. Align Repo Docs

Sweep all impacted authority surfaces, not just touched files:

- `AGENTS.md`
- `README.md` + nested READMEs
- requirements, ADRs, specs, runbooks, setup, release, operator docs
- other affected markdown guidance

Update in place. Prefer existing canonical docs over creating new ones.

### 7. Commit, Push, And Clean Up Hosted State

1. Group changes into scoped conventional commits.
2. Push PR head branch.
3. Resolve or reply to hosted review items only after relevant fix is on GitHub.
4. Refresh hosted review state immediately after push.
5. Re-check unresolved hosted feedback until zero for issues you addressed.

### 8. Babysit Until Ready To Merge

After each push:

1. refresh checks, reviews, unresolved hosted feedback
2. handle new CI failures or review comments
3. loop until ready-to-merge criteria in `references/closure-contract.md` satisfied

Stop only for:

- ready-to-merge state
- real blocker requiring user help

## Signr-Specific Rules

- Use `bun` or `bunx` only for repo tasks.
- Prefer `bun run validate:local:agent` as pre-final stop rule.
- For Expo/EAS work, prefer repo scripts such as:
  - `bun run workflows:validate`
  - `bun run ci:validate:eas`
  - `cd apps/mobile && bun run eas -- <command>`
- For Vercel work, use `bunx vercel <command>` from `apps/web`.
- For Turborepo work, prefer repo scripts first, then `bunx turbo` for direct inspection when necessary.
- Keep docs hubs + authority surfaces aligned with current repo guidance.

## Boundaries

- Do not assume prompt failure story still current; verify live state first.
- Do not stop at local green if hosted review threads or CI still open.
- Do not use `reviewDecision` as sole stop signal.
- Do not preserve dead fallbacks or legacy parallel paths.
- Do not finish after single push; keep loop until closure criteria met or run blocked.
- Do not merge on user's behalf.

## Example Prompts

- `Use $signr-pr-closure-loop to fix PR #148 end-to-end, clear remaining hosted review feedback, and stop only when it is ready to merge.`
- `Use $signr-pr-closure-loop on current Signr branch to repair failing Expo + Vercel checks, align docs, and babysit PR after push.`
