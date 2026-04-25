---
name: convex-audit
description: Audit Convex—schema, security, runtime edges, migrations, function risk. Triggers—review, contract, remediate. Not greenfield spec (convex-feature-spec).
---

# Convex Audit

Use this skill for read-first Convex audits that produce a clear remediation plan before implementation.

## Workflow

1. Read the repo `AGENTS.md`.
2. Run `/home/bjorn/.codex/skill-support/bin/convex-scan inventory --cwd <repo> --out <json>`.
3. Run `/home/bjorn/.codex/skill-support/bin/convex-scan surface --cwd <repo> --out <json>`.
4. Run `/home/bjorn/.codex/skill-support/bin/convex-scan gaps --inventory <json> --out <json>`.
5. Read only the references needed for the active findings:
   - `references/security.md`
   - `references/schema.md`
   - `references/runtime-boundaries.md`
   - `references/migrations.md`
6. Validate non-trivial recommendations against current docs before finalizing.
7. Output a remediation plan with file targets, risk level, and verification steps.

## Use When

- The user asks for a Convex audit, security pass, schema review, or backend remediation plan.
- The repo has Convex and the main task is to assess existing architecture or implementation quality.

## Do Not Use When

- The task is a new feature specification with multiple design options.
- The task is only a dependency upgrade or docs sync.

## Outputs

- A concise audit summary.
- Ranked findings.
- An implementation-ready remediation checklist.

## Resources

- Inventory helpers via `/home/bjorn/.codex/skill-support/bin/convex-scan`
- `references/security.md`
- `references/schema.md`
- `references/runtime-boundaries.md`
- `references/migrations.md`

