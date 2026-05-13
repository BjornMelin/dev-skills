---
name: docs-align
description: Align docs with code. Detect drift and update ADRs, specs, README, AGENTS. Use after implementation or for docs drift reviews. Not for code review fixes or platform architecture.
---

# Docs Align

Use this skill after implementation work or when you suspect documentation drift.

## Workflow

1. Read the repo `AGENTS.md`.
2. From this skill root (directory containing `SKILL.md`), run drift collection (requires `git` on PATH):
   - `python3 scripts/docs_drift.py collect --cwd <repo> --out <json>`
   - Equivalent collect-only shortcut: `python3 scripts/check_docs_drift.py --cwd <repo> --out <json>`
3. `python3 scripts/docs_drift.py compare --input <json> --out <json>`
4. Render the summary:
   - `python3 scripts/docs_drift.py render --input <json> --format md`
5. Use **Alignment policies** below only for doc surfaces that the gap map says are in scope.
6. Update docs only after the gap map is clear.

### Path note

Commands above assume the current working directory is this skill’s root (`skills/docs-align` in this repository). If the working directory is elsewhere, invoke the same files with an absolute path, for example:

`python3 /path/to/docs-align/scripts/docs_drift.py collect --cwd <repo> --out <json>`

## Alignment policies

### ADR

Create or update an ADR when the implementation changed:

- architecture boundaries
- execution model
- durable workflow policy
- major dependencies or infrastructure choices

Do not create ADR churn for small local refactors.

### Spec

Update product or architecture specs when the implementation changed:

- interfaces
- contracts
- verification steps
- operational behavior

Prefer deleting stale spec text over leaving contradictory guidance.

### README

Update the README when the change affects:

- setup
- commands
- environment variables
- high-level architecture or usage expectations

Keep the README high signal; move deep details into docs when needed.

## Use When

- The task is post-implementation doc alignment.
- The user wants README, ADR, spec, or AGENTS updates based on code changes.

## Do Not Use When

- The task is only code review remediation.
- The task is only dependency planning.

## Outputs

- likely impacted docs
- missing/update/delete doc tasks
- a concise docs alignment summary
