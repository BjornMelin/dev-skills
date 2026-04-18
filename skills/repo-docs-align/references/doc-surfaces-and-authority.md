# Document Surfaces And Authority

Use this file to choose the smallest correct authority surface for repo-doc alignment work.

## Surface selection

### `AGENTS.md`

Use for durable repo guidance that a coding agent must know in a fresh session:
- required verification commands
- architecture invariants
- canonical file ownership
- stable review/release expectations
- recurring failure modes with durable fixes

Do not use for:
- task status
- branch summaries
- step-by-step project plans
- feature-specific narration

### `README.md`

Use for repo entrypoint guidance:
- what the repo is
- how to set up or navigate it
- where core docs live
- top-level workflows users or contributors need first

Do not overload `README.md` with deep architecture or long task tracking.

### ADRs

Use when one material decision needs durable rationale:
- options considered
- chosen path
- consequences
- superseded decisions

Prefer ADRs over broad design docs when one decision is the real unit of change.

### Specs / requirements / design docs

Use for implementation-shaping detail:
- goals and non-goals
- constraints
- system design
- rollout and rollback
- validation criteria

Prefer updating an existing canonical spec over creating a sibling spec.

### Runbooks / operator guides

Use for operational action:
- detect
- diagnose
- recover
- escalate

Optimize for speed and correctness under pressure.

### Exec artifact / checklist / plan file

Use only when the task needs durable future-session execution context.

This file should be:
- checkable
- current
- scoped to one stream of work
- explicit about validation and remaining tasks

Do not duplicate authority already owned by `AGENTS.md`, ADRs, or stable specs. Link to them instead.

### Docs hubs / status ledgers / execution catalogs

Some repos publish an explicit docs-role map or machine-readable execution surface, for example:
- `docs/README.md`
- `requirements.md` or status ledgers
- release indexes
- prompt catalogs
- machine-readable ledgers or inventories

Treat these as authority-routing inputs. They often answer:
- which doc owns current implementation status
- which docs are active vs superseded
- where execution plans belong
- which filenames or ledgers are contractually stable

If these surfaces exist, read them before creating new docs or relocating work.

## Decision rules

1. Update the current authority doc first.
2. Create a new doc only if no existing authority surface fits.
3. Mark stale docs as superseded or delete them when they no longer own the truth.
4. Keep one canonical source per concern.
5. Prefer repo-local conventions for naming and placement.

## Mapping hints

If the issue is about:
- repo-wide coding agent behavior -> `AGENTS.md`
- how to enter or navigate the repo -> `README.md`
- how docs are partitioned, which docs are active, or where execution work belongs -> docs hub / status ledger / execution catalog
- why a material decision changed -> ADR
- how something should be built or validated -> spec / requirements doc
- how to operate or recover a system -> runbook
- how to continue a bounded workstream next session -> exec artifact
