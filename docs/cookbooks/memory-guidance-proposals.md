# Memory Guidance Proposals

Use this workflow when prior Codex memory suggests a durable repo rule, skill
change, validation note, or operating practice that should be reviewed and
tracked in this repository.

Memory is historical context. It can explain why a direction is worth checking,
but it is not current authority. Current tracked docs, code, tests, GitHub
issues, PR review state, and official upstream sources remain the authority for
changes.

## When to Use This

Create a memory guidance proposal when a prior run contains reusable guidance
that should affect future work, such as:

- a recurring validation command or PR remediation pattern;
- a repo bootstrap rule that belongs in a pack or template;
- a subagent, skill, or docs convention that should be durable;
- a stale or unsafe instruction that should be retired from future guidance.

Do not use this workflow for runtime memory storage design. Durable Codex run
state, SQLite event logs, RAG, and personalization patterns belong in
`skills/codex-sdk/SKILL.md` and its references:

- `skills/codex-sdk/references/state-memory-sqlite.md`
- `skills/codex-sdk/references/rag-and-memory.md`
- `skills/codex-sdk/references/context-personalization.md`

## Proposal Ledger

Use a short ledger in the issue or PR body before editing durable guidance:

| Field | Required Content |
| --- | --- |
| Memory lead | What prior memory suggested, with a short public or sanitized citation when available. |
| Current authority checked | Tracked files, GitHub issue or PR state, tests, docs, or upstream sources inspected now. |
| Decision | Adopt, adapt, reject, or archive. |
| Files changed | Durable repo files that will carry the decision. |
| Validation | Commands rerun in the current checkout. |
| Privacy notes | Confirmation that no private local paths, ignored overlays, secrets, or run ledgers are being committed. |
| Status | Proposed, implemented, superseded, or rejected. |

Keep the ledger compact. Do not paste local `.codex` session paths, run-ledger
paths, or private workstation paths into issues or PRs. If the evidence needs a
longer audit trail, use the research ledger workflow in
`docs/cookbooks/evidence-ledgers.md` and keep run-specific artifacts untracked.
If a reviewer asks for a tracked evidence bundle, include only sanitized,
public-source summaries and keep raw local ledgers, provider dumps, secrets,
cookies, private paths, and private source excerpts out of git.

## Verification Rules

Before adopting memory-derived guidance:

1. Reopen the current tracked authority for the affected surface.
2. Treat prior memory as a lead, not a source of truth.
3. Rerun or refresh drift-prone evidence when it is cheap to verify.
4. Mark any retained unverified historical claim as `UNVERIFIED`.
5. Prefer a current command result, source file, docs page, or hosted GitHub
   record over a prior-run summary.

Stale smoke evidence is not acceptance evidence. If prior memory says a command
passed, rerun the relevant command in the current branch or describe the old
result only as historical context.

## Privacy Rules

Do not commit:

- private local absolute paths from workstation memory;
- ignored local subagent overlays, roles, or private manifests;
- `.codex/research/` run ledgers or reports from local investigations;
- provider dumps, screenshots, or logs that contain private source excerpts;
- secrets, tokens, account IDs, or auth cookies.

If a durable rule needs a path example, use a repo-relative path or a neutral
placeholder. If a local-only overlay motivated the guidance, document the public
shape of the rule without naming the private repo or role.

## Where the Decision Belongs

Choose one durable owner:

| Decision Type | Durable Owner |
| --- | --- |
| Contributor or agent operating rule | `AGENTS.md` |
| User-facing docs navigation | `README.md` and `docs/index.md` |
| Workflow steps | `docs/cookbooks/` |
| Validation commands or manual checks | `docs/runbooks/validation.md` |
| CLI behavior or data contracts | `docs/reference/` or `docs/specs/` |
| Skill behavior | `skills/<skill-name>/SKILL.md` |
| Long skill reference material | `skills/<skill-name>/references/` |

Avoid duplicating the same rule in multiple docs. Link to the canonical owner
from navigation docs instead.

## Review Checklist

Use this checklist in PRs that adopt memory-derived guidance:

- memory lead is labeled as historical context;
- current authority was checked and cited in the PR body;
- stale smoke evidence was rerun or explicitly marked historical;
- private local paths, ignored overlays, provider dumps, and run ledgers are
  absent from the diff;
- related `codex-sdk` references remain the owner for runtime memory/storage
  patterns;
- docs links and changed skill checks pass.

Use the docs gates in `docs/runbooks/validation.md`. Add this skill check only
when the proposal changes `skills/codex-sdk/` or its references:

```bash
python3 tools/skill/quick_validate.py skills/codex-sdk
```
