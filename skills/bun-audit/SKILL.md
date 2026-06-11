---
name: bun-audit
description: "Shared Bun audit/remediation router. Use when auditing a repo for Bun-first correctness, explaining findings, listing rules, planning safe fixes, applying low-risk remediations, or validating Bun-related changes."
---

# Bun Audit

Use this skill as a thin router over the native `codex-dev bun` engine. Do not
duplicate rules or remediation logic here; `bun-dev` owns rule text and
references.

## Commands

- `audit`: run `codex-dev --json bun audit --root <repo>`.
- `rules list`: run `codex-dev --json bun rules list`.
- `rules show <rule-id>`: run `codex-dev --json bun rules show <rule-id>`.
- `fixes plan`: run `codex-dev --json bun fixes plan --root <repo>`.
- `fixes apply`: run `codex-dev --json bun fixes apply --root <repo>`.
- `validate plan`: run `codex-dev --json bun validate plan --root <repo>`.
- `validate run`: run `codex-dev --json bun validate run --root <repo> --fail-on warn`.
- `benchmark`: run `codex-dev --json bun benchmark --root <repo>`.

## Operating Rules

- Treat `bun-dev` as the source of truth for rules and reference snapshots.
- Prefer `codex-dev bun ...` over the temporary `bun-platform` compatibility shim.
- Do not run `fixes apply`, `validate run`, or `references sync` in parallel
  against the same repo or skill root.
- Audit cache writes are opt-in with `--write-cache`.
- Safe fixes write rollback artifacts under external dev-skills state.
- If a fix is risky or ambiguous, stop at `fixes plan` and surface the tradeoff.
