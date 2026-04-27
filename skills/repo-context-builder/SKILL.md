---
name: repo-context-builder
description: Build/refresh REPO_CONTEXT.md + REVIEW_BRIEF from repo. Triggers—new repo, zip for later chats, grounded brief. Not code w/o these artifacts.
---

Create or refresh `REPO_CONTEXT.md` and `REVIEW_BRIEF.md` for the current repository.

Keep the workflow evidence-first, compact, and deterministic. The goal is to leave behind two high-signal documents that future agents and chats can use immediately without re-discovering the repository from scratch.

## Use bundled resources

Read these files before writing output:

- `assets/templates/REPO_CONTEXT.md`
- `assets/templates/REVIEW_BRIEF.md`
- `references/repo-analysis-playbook.md`
- `references/output-checklist.md`

Optional helper:

- `scripts/repo_inventory.py` generates a fast inventory of the repository. Use it when helpful, but do not rely on it blindly. Verify important claims against the actual files.

## Output contract

Produce exactly these files at the repository root unless the user explicitly asks for different paths:

- `REPO_CONTEXT.md`
- `REVIEW_BRIEF.md`

If files already exist, update them in place instead of creating duplicates.

## Required workflow

### 1. Resolve the repository and scope

1. Identify the repository root.
2. Determine whether the repo is a single app, service, library, infra repo, or monorepo.
3. Determine whether the user asked for:
   - a full initial intake,
   - a refresh of stale repo docs,
   - or a task-scoped brief for a specific feature, bug, migration, or review.
4. If the user did not specify a focused task, still create a useful generic `REVIEW_BRIEF.md` that captures the highest-value next work areas, risks, and recommended entry points.

### 2. Collect evidence before writing

Inspect the repo in this order:

1. Root signals:
   - `README*`
   - `AGENTS.md`
   - manifests and lockfiles
   - CI config
   - container and deployment config
   - infra directories
2. Runtime and entrypoints:
   - server entry files
   - CLI entry files
   - app bootstrap files
   - job and worker entry files
3. Quality and test signals:
   - lint, typecheck, test config
   - test directories and representative tests
4. Architecture signals:
   - top-level app, package, service, or module folders
   - shared libraries
   - adapters, API routes, models, schemas, migrations, queues, jobs
5. Operational signals when available:
   - deployment workflows
   - observability setup
   - release or environment docs
6. Git signals when available and useful:
   - active branch
   - recent commits
   - recent changed areas

Use the optional inventory script if it helps accelerate discovery, but confirm important claims by reading the underlying files.

### 3. Fill `REPO_CONTEXT.md`

Use the bundled template and replace every placeholder.

Requirements:

- Summarize the repository’s purpose in one tight paragraph.
- Describe the actual architecture, not a guessed ideal architecture.
- Capture exact setup, run, lint, typecheck, test, build, and deploy commands when they are discoverable.
- For monorepos, include the root plus each materially important package or app.
- Include the most important files and why they matter.
- Record risks, tech debt, and open unknowns explicitly.
- Use file paths as evidence anchors throughout.

### 4. Fill `REVIEW_BRIEF.md`

Use the bundled template and replace every placeholder.

Requirements:

- Translate the current task into a concise engineering brief.
- Capture in-scope and out-of-scope boundaries.
- List the relevant files, packages, and systems.
- Explain the current state with evidence.
- Identify gaps, risks, and unknowns.
- Propose a concrete implementation or review plan.
- Include an exact verification plan with commands.
- End with a ready-to-paste working prompt for the next agent or chat.

If there is no specific task from the user, make the brief about one of these, in priority order:

1. the most valuable missing capability,
2. the highest-risk weakness,
3. or the highest-leverage cleanup / hardening opportunity.

### 5. Validate before finishing

Before you stop, verify all of the following:

- Both files exist in the repository root.
- No placeholder markers remain.
- No section is silently omitted. Use `Not found in repo` or `Unknown` when necessary.
- Commands are copied exactly from the repo when possible, not invented.
- Claims about architecture, deployment, tests, or integrations are backed by file evidence.
- The writing is concise and decision-useful, not bloated.

## Evidence rules

- Prefer repository files over assumptions.
- Prefer executable config over prose docs when they disagree.
- Prefer current manifests and CI definitions over stale README instructions.
- Treat generated code, vendored files, and build output as low-trust signals.
- If something cannot be confirmed, label it explicitly as `Unknown` or `Not found in repo`.

## Writing rules

- Write for future engineers and future agents.
- Be specific. Name files, directories, commands, workflows, services, and boundaries.
- Keep summaries dense and useful.
- Do not dump giant file trees.
- Do not copy large blocks from README files.
- Do not hallucinate commands, environments, services, secrets, or deployment targets.
- Do not mark something as production-ready unless the repo evidence supports that claim.

## Monorepo rules

When the repo contains multiple apps or packages:

- Describe the repo root separately from each important package or app.
- Distinguish shared libraries from deployable services.
- Record package-specific commands when they differ from root commands.
- Call out cross-package dependencies and ownership boundaries when they are inferable.

## Missing-information rules

When evidence is incomplete:

- State exactly what was missing.
- State where you looked.
- State the consequence of the missing information.
- Suggest the fastest way to resolve the unknown.

## Completion standard

This skill is complete only when a future agent could open the repo with just these two files and quickly understand:

- what the repo is,
- how it is structured,
- how to run and verify it,
- what matters most,
- and how to start the next task safely.

## Invocation examples

Explicit invocation is the most reliable way to use this skill.

Example prompts:

- `$repo-context-builder Analyze this repository and create REPO_CONTEXT.md plus REVIEW_BRIEF.md using the bundled templates.`
- `$repo-context-builder Refresh the existing REPO_CONTEXT.md and REVIEW_BRIEF.md after recent changes. Re-check commands, architecture notes, and risks.`
- `$repo-context-builder Analyze this repo and create a task-scoped REVIEW_BRIEF.md for the current authentication bug, plus refresh REPO_CONTEXT.md where needed.`
