---
name: signr-pr-closure-loop
description: End-to-end Signr PR remediation and closure operator for GitHub pull requests that need review-thread fixes, CI repair, Expo/EAS or Vercel/Turborepo debugging, repo-wide docs alignment, scoped conventional commits, push, and post-push babysitting until the PR is truly ready to merge. Use when a Signr PR is blocked by review comments, CodeRabbit findings, failing checks, EAS/Vercel workflow issues, or docs drift, especially for requests like “fix PR #148 end-to-end”.
---

# Signr PR Closure Loop

Use this skill as the repo-specific orchestrator for Signr pull-request closure
work.

Start from live GitHub and repo state. Do not trust stale prompt claims about
failing checks, pending comments, or review decisions.

Read only what you need:

- `references/tool-routing.md`
- `references/closure-contract.md`

## Core Contract

- Read root and scoped `AGENTS.md` first.
- Resolve the target PR, current head SHA, changed files, required checks,
  review state, unresolved hosted threads/comments, and mergeability before
  editing.
- Treat live hosted state as truth. Skip stale findings already fixed on the
  current head.
- Prefer existing skills, plugins, CLIs, and repo-native commands over
  rebuilding side systems.
- Apply hard-cut, reducing-entropy, and clean-code defaults when changing
  behavior: keep one canonical path, delete obsolete code, and avoid fallback
  layers.
- Use official current docs, release notes, changelogs, and CLI help when
  platform behavior could have drifted.
- Update impacted authority docs in the same pass before final closeout.
- Commit in scoped conventional commits, push, then keep monitoring until the
  PR is ready to merge or a real blocker requires user help.
- Do not merge the PR. Stop at ready-to-merge plus a concise summary.

## Workflow

### 1. Normalize The Target

1. Resolve the PR from explicit input or the current branch.
2. Snapshot:
   - PR metadata and changed files
   - current checks on the head SHA
   - reviews, review threads/comments, and mergeability
   - local worktree cleanliness
3. If the worktree contains unrelated dirty changes, stop and ask the user.

### 2. Build The Blocker Map

Split the work into distinct blocker classes:

- unresolved review feedback
- failing or missing hosted checks
- docs drift created by branch changes
- mergeability or branch-state blockers

Use hosted unresolved-thread state as closure truth. Do not rely on
`reviewDecision` alone.

### 3. Route Each Blocker To The Right Lane

Use `references/tool-routing.md`.

Default routing:

- review feedback and review bundles -> `$gh-pr-review-fix` plus `review-pack`
- failing GitHub Actions checks -> `$github:gh-fix-ci`
- Expo/EAS workflows, builds, or validation -> `$expo:expo-cicd-workflows`,
  `$expo:upgrading-expo`, Expo MCP when available, and EAS CLI
- Vercel or Turborepo issues -> `$vercel` plugin capabilities,
  `$vercel:turborepo`, Vercel CLI, Turbo CLI
- Bun, package-manager, or runtime policy issues -> `$bun-dev`
- repo-wide docs drift -> `$repo-docs-align` if installed, else `$docs-align`,
  else do a direct docs sweep
- post-push monitoring -> `$babysit-pr`
- scoped commits -> `$commit`

If a named skill is unavailable, continue with the closest repo-native fallback.

### 4. Implement Minimal Canonical Fixes

1. Fix the highest-signal blocker cluster first.
2. Reproduce CI-specific failures locally with the repo-native command that
   matches the hosted lane.
3. Delete obsolete or parallel code paths made unnecessary by the fix.
4. Update tests, fixtures, and snapshots to the canonical shape only.
5. Avoid broad refactors not tied to an active blocker.

### 5. Verify Before Publishing

1. Run the smallest relevant local gates first.
2. Run branch-relevant surface checks from repo guidance and the failing
   workflow.
3. Before finalizing a fix set, run `bun run validate:local:agent`.
4. If docs or architecture contracts changed, run `bun run docs:arch:validate`.
5. Record exact commands and outcomes for the final summary.

### 6. Align Repo Docs

Sweep all impacted authority surfaces, not just touched files:

- `AGENTS.md`
- `README.md` and nested READMEs
- requirements, ADRs, specs, runbooks, setup, release, and operator docs
- other affected markdown guidance

Update in place. Prefer existing canonical docs over creating new ones.

### 7. Commit, Push, And Clean Up Hosted State

1. Group changes into scoped conventional commits.
2. Push the PR head branch.
3. Resolve or reply to hosted review items only after the relevant fix is on
   GitHub.
4. Refresh hosted review state immediately after push.
5. Re-check unresolved hosted feedback until it reaches zero for the issues you
   addressed.

### 8. Babysit Until Ready To Merge

After each push:

1. refresh checks, reviews, and unresolved hosted feedback
2. handle any new CI failures or review comments
3. loop until the ready-to-merge criteria in
   `references/closure-contract.md` are satisfied

Stop only for:

- ready-to-merge state
- a real blocker that requires user help

## Signr-Specific Rules

- Use `bun` or `bunx` only for repo tasks.
- Prefer `bun run validate:local:agent` as the pre-final stop rule.
- For Expo/EAS work, prefer repo scripts such as:
  - `bun run workflows:validate`
  - `bun run ci:validate:eas`
  - `cd apps/mobile && bun run eas -- <command>`
- For Vercel work, use `bunx vercel <command>` from `apps/web`.
- For Turborepo work, prefer repo scripts first, then `bunx turbo` for direct
  inspection when necessary.
- Keep docs hubs and authority surfaces aligned with current repo guidance.

## Boundaries

- Do not assume the prompt's failure story is still current; verify live state
  first.
- Do not stop at local green if hosted review threads or CI are still open.
- Do not use `reviewDecision` as the sole stop signal.
- Do not preserve dead fallbacks or legacy parallel paths.
- Do not finish after a single push; keep the loop alive until closure criteria
  are met or the run is blocked.
- Do not merge on the user's behalf.

## Example Prompts

- `Use $signr-pr-closure-loop to fix PR #148 end-to-end, clear remaining hosted review feedback, and stop only when it is ready to merge.`
- `Use $signr-pr-closure-loop on the current Signr branch to repair failing Expo and Vercel checks, align docs, and babysit the PR after push.`
