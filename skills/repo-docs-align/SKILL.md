---
name: repo-docs-align
description: Sync repo docs (AGENTS, README, ADRs, specs, runbooks, doc comments) ↔ code/workflow. Triggers—big change, drift, docs-align/AGENTS prompts, plan, governance. Repo-native tools, any stack.
metadata:
  successor_to: docs-align
---

# Repo Docs Align

Use this skill to turn repo-doc drift into a grounded plan; when asked or clearly right, verified doc-align implementation.

Read before authority or compression calls:

- [references/doc-surfaces-and-authority.md](references/doc-surfaces-and-authority.md)
- [references/adaptive-compression.md](references/adaptive-compression.md)
- [references/subagent-orchestration.md](references/subagent-orchestration.md)

Bundled resources:

- `scripts/new_repo_docs_align_artifact.py` - scaffold hidden artifacts under `.agents/<skill-name>/YYYY-MM/MM-DD/NN/` (`<skill-name>` = installed skill dir name)
- `templates/drift-map.md`
- `templates/reviewed-surfaces.md`
- `templates/exec-plan.md`
- `templates/retrospective.md`

## Core contract

- Start from current repo reality, not prior assumptions.
- Task incomplete until deliverables produced or marked `[blocked]`.
- One canonical doc surface per concern; update in place; avoid duplicate new docs.
- Default: repo-wide doc alignment - branch changes affect doc-owned concern anywhere → inspect surface, bring current.
- Hidden working artifacts OK for analysis, planning, tracking, retros; not canonical repo docs. Artifacts support run; canonical docs stay authority.
- Nontrivial runs: explicit checklist of outputs + reviewed areas. Before finalize: every deliverable + related doc surface covered or `[blocked]`.
- Major-choice scoring (material decisions only):
  - No global fixed weight mix - per decision, pick criteria + weights for that focus (placement, supersession, stack/tool, scope, governance, risk, readers, maintenance; use what fits best).
  - Name dimensions; weights sum to clear whole (e.g. `100%`); score options; record rubric + scores in plan/synthesis.
  - Target `9.0+` on winning path under that rubric when scoring applies.
- User input improves major decision → one question at a time. `request_user_input` when available:
  - 2-3 mutually exclusive options
  - Rubric for this decision (dimensions + weights) + per-option `0.0-10.0` scores (weighted totals OK)
  - Recommended option first

## Tool posture

- `update_plan` for nontrivial runs - workstream explicit + checkable.
- `request_user_input` for major authority, scope, artifact decisions repo evidence cannot settle. No broad free-form batches.
- Parallel read-only retrieval when safe for fast discovery.
- Subagents only when available; bounded exploration, evidence, doc/API verification, grep/path/file audits, focused review, validation triage. Main agent keeps authority, synthesis, default edits local.
- Built-in `web.*` or MCP research when repo/docs evidence thin, stale, or task asks external verification.
- Tool/plugin discovery only when capability not known this session.
- Image/PDF inspection when source of truth is visual (screenshots, rubrics, scans, PDF-only requirements).

## Subagent contract

- Default: exploration + evidence only. No fan-out full doc authorship by default.
- Prefer `1-3` focused subagents; no nested subagents unless user asks.
- Explorer agents read-heavy, evidence-first. Implementation workers only for narrow follow-on after main agent chose authority path.
- Every `spawn_agent` call: set `model` + `reasoning_effort`.
- Main skill: durable cheap-first — small model for bounded exploration, tighten task before escalate, escalate only underfitting subagent. Model ladder + prompt template in `references/subagent-orchestration.md`.
- Every delegated task specifies:
  - narrow task or question
  - allowed scope or surfaces
  - read-only vs may edit
  - main agent wait now vs local work until synthesis gate
  - exact return format
- Default wait: continue local non-overlapping work; wait at synthesis gates before major authority, final recommendations, edits needing delegated evidence.
- Evidence-first returns:
  - key finding or result
  - files + symbols inspected
  - commands or checks run, if any
  - recommended next action
  - unresolved questions or risks
- Conflicting delegated findings → surface conflict; resolve in main synthesis before act.

## Workflow

### 1. Normalize the job

Extract:

- task `plan-only`, `plan-then-execute`, or `audit-only`
- user wants durable repo artifact (exec plan/checklist)
- request mentions `AGENTS.md`, README, ADRs, specs, runbooks, requirements docs, code comments
- whether compression/token optimization is in scope

User asked specific response format → preserve exactly.

Repo dirty or branch-specific → anchor on current worktree:

- `git status --short`
- `git diff --name-only`
- changed code + docs → active functionality + authority surfaces
- from anchor: sweep related docs needing updates, corrections, supersession, rewrites, new coverage
- worktree = start signal for related-doc discovery, not outer boundary of docs review
- OK to inspect/plan/edit docs untouched in worktree if same functionality, workflow, contract, authority chain
- sweep until repo-wide docs for affected functionality current; no stale related guidance

### 2. Inventory the repo surfaces

Inspect smallest high-signal set first:

- root `AGENTS.md`
- nearest scoped `AGENTS.md`
- `README.md` + docs indexes
- repo-local docs hub / status authority: `docs/README.md`, `requirements.md`, release indexes, execution catalogs, machine-readable ledgers when present
- ADR/spec/runbook dirs
- requirements, standards, policy docs when present
- recently changed files + nearby comments/docstrings

`rg`/repo-native search map likely impacted docs before edit. Parallelize independent read-only discovery before synthesis. Delegation: lightweight explorer subagents for repo mapping only after evidence targets known.
Many docs → prioritize via docs hubs, status ledgers, execution catalogs, changed functionality; prioritization ≠ coverage limit.

### 3. Route into the right supporting skills and plugins

Prefer repo-native or user-named skills first. Adapt; don’t assume stack.

Examples:

- `$agents-md-maintainer` before finalizing any `AGENTS.md` edit.
- `$technical-writing` when drafting/rewriting ADRs, specs, runbooks, migration docs, internal guides.
- `$caveman-compress` only when surface fits `references/adaptive-compression.md`.
- `$hard-cut` + `$clean-code` when simplifying stale doc structure or removing superseded guidance.
- Stack/platform skills/plugins (`$github`, `$vercel`, `$expo`, `$sentry`, Context7, built-in web search) only when repo context or user request makes them relevant.

Named skill/plugin unavailable → note briefly; closest valid fallback.

### 4. Build a drift map before proposing changes

Compare current docs to:

- implemented behavior
- current scripts/commands
- current architecture + file ownership
- current validation flow
- branch-specific changes that made existing docs stale
- related repo docs describing, constraining, teaching, operating, validating, routing affected functionality

Good delegation:

- one explorer: `AGENTS.md` + scoped guidance drift
- one explorer: ADR/spec/runbook/README ownership mapping
- one explorer: external doc/API verification when repo evidence thin; built-in `web.*` where search needed

Classify each finding:

- `update-in-place`
- `create-canonical-doc`
- `mark-superseded`
- `delete-stale-guidance`
- `leave-unchanged`

No new docs until existing authority doc confirmed not owning concern.
Don’t stop at first matching doc. Follow authority chain across README hubs, AGENTS, requirements, ADRs, specs, runbooks, setup, release docs, prompt catalogs, nearby comments/docstrings until related doc set aligned.
Map every proposed doc/comment change to exact file, path, or code-comment surface. No named target → not grounded yet.

### 5. Choose the canonical authority path

Authority matrix in `references/doc-surfaces-and-authority.md`.

Rules:

- Prefer modifying current canonical doc over creating new.
- New ADR/spec/runbook only when concern materially new + doesn’t fit current authority surface.
- Keep `AGENTS.md` durable repo guidance only — no task logs or branch narration.
- Repo already has docs-role map, status ledger, execution catalog → first-class authority input before inventing placement.
- Long-lived execution context needed → create/update one checkable repo-local exec artifact per existing naming conventions.

### 6. Produce the durable exec artifact when needed

Future-session handoff → create/update one canonical plan/checklist with only fitting sections:

- scope + intent
- summary of completed work
- files + surfaces reviewed
- what is done
- remaining tasks + subtasks
- further improvements worth considering
- required research
- decisions made + open decisions
- validation commands + success criteria
- required skills/plugins/tools
- exact files/dirs to load next session
- enforced rules/invariants next session must preserve
- blockers + assumptions

Execution-oriented, not diary.
If the repo already has an execution catalog, prompt ledger, or trigger-prompt system, update that canonical surface, not parallel plan file.

### 6a. Hidden working artifact policy

Non-canonical artifacts from this skill default:

- `.agents/<skill-name>/YYYY-MM/MM-DD/NN/`

When this skill is installed as `repo-docs-align`, that resolves to `.agents/repo-docs-align/YYYY-MM/MM-DD/NN/`.

Use this hidden work area for things like:

- `drift-map.md`
- `reviewed-surfaces.md`
- `exec-plan.md`
- `retrospective.md`
- other temporary/session analysis supporting docs alignment

Rules:

- create directory when needed
- fresh numeric run bucket `01`, `02`, `03` same-day repeats
- ensure repo ignores `.agents/` or min `.agents/<skill-name>/`
- canonical docs, ledgers, specs, ADRs, runbooks, active execution catalogs stay true authority surfaces — not `.agents/<skill-name>/`
- typed filenames vs one giant note when artifacts differ materially
- only create artifacts useful for run; no empty scaffolding

Deterministic scaffolding for hidden work area:

```bash
python3 scripts/new_repo_docs_align_artifact.py \
  --dir <repo-root> \
  --artifacts drift-map,reviewed-surfaces,exec-plan,retrospective
```

Run exact command from installed skill directory. Script resolves bundled templates relative to itself; shorter relative path unambiguous install-wide.

`--artifacts` = only files needed. `--force` = only when intentionally refreshing existing artifact file.

### 7. Implement doc and comment changes when the task calls for execution

After drift map + authority are grounded:

- update canonical docs
- tighten or remove stale guidance
- align nearby code comments/docstrings where useful
- smallest edit set that fully resolves grounded drift; no partially corrected authority chains
- minimal, reviewable diffs

Do not rewrite unrelated docs for imperfection alone.

### 8. Apply adaptive compression only where it improves the repo

Follow `references/adaptive-compression.md`.

Default:

- compress internal operational, agent-facing, workflow, repo-maintenance docs when scan speed + token efficiency improve
- richer prose for public, product, marketing, narrative, teaching docs unless user requests compression

When compressing:

- preserve code, commands, paths, URLs, headings, tables, exact technical terms
- keep document navigable
- don’t cavemanify docs whose value is nuanced explanation or polished prose

### 9. Verify before finalizing

Verify:

- every doc change maps to specific file or confirmed gap
- every unchanged reviewed surface has reason (explicit or implicit) grounded in current repo reality
- authority choices match current repo structure
- requested deliverables complete
- formatting matches surrounding docs
- referenced commands, scripts, paths still exist
- irreversible or external side effects surfaced before execution

Changed `AGENTS.md` → brief `$agents-md-maintainer` pass before closeout.

## Output shape

Default order unless user asked for a different format:

1. drift summary
2. canonical authority decisions
3. exec artifact path or inline plan
4. implemented doc/comment changes
5. verification commands + residual gaps

## Stop rules

- Stop + ask only when major authority decision genuinely ambiguous + repo evidence can’t resolve.
- Missing evidence or uncertain claims → `UNVERIFIED`.
- Retrieval empty or suspiciously narrow → retry one or two different strategies before conclude.
