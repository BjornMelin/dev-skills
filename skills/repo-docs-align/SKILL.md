---
name: repo-docs-align
description: Align `AGENTS.md`, README, ADRs, specs, runbooks, doc comments, and other repo docs with the current branch and intended workflow. Use after nontrivial implementation, for docs-drift reviews, or when prompts mention `docs-align`, AGENTS maintenance, canonical docs, exec plans, durable handoff docs, or repo documentation governance. Works across stacks and repo types by routing into repo-native skills, plugins, and validation commands instead of assuming one language or framework.
metadata:
  successor_to: docs-align
---

# Repo Docs Align

Use this skill to turn repo-doc drift into a grounded plan and, when requested or clearly appropriate, a verified doc-alignment implementation.

Read these references before making authority or compression decisions:
- [references/doc-surfaces-and-authority.md](references/doc-surfaces-and-authority.md)
- [references/adaptive-compression.md](references/adaptive-compression.md)
- [references/subagent-orchestration.md](references/subagent-orchestration.md)

Bundled resources:
- `./scripts/new_repo_docs_align_artifact.py` - scaffold hidden working artifacts under `.agents/<skill-name>/YYYY-MM/MM-DD/NN/` where `<skill-name>` is the installed skill directory name
- `./templates/drift-map.md`
- `./templates/reviewed-surfaces.md`
- `./templates/exec-plan.md`
- `./templates/retrospective.md`

## Core contract

- Start from current repo reality, not prior assumptions.
- Treat the task as incomplete until requested deliverables are either produced or marked `[blocked]`.
- Prefer one canonical doc surface per concern. Update in place when possible; avoid duplicate new docs.
- Treat repo-wide documentation alignment as the default goal: if current implementation or branch changes affect a doc-owned concern anywhere in the repo, inspect that surface and bring it current.
- Use hidden working artifacts freely when they help analysis, planning, tracking, or retrospectives, but do not confuse those files with canonical repo documentation. Hidden artifacts support the docs-alignment run; canonical docs remain the authority.
- Keep an explicit checklist of required outputs and reviewed repository areas for nontrivial runs. Before finalizing, confirm that every required deliverable and every related documentation surface is either covered or explicitly marked `[blocked]`.
- Use major-choice scoring only for material decisions. Score with this weighted framework and target `9.0+` for final choices:
  - Solution leverage: `35%`
  - Application value: `30%`
  - Maintenance and cognitive load: `25%`
  - Architectural adaptability: `10%`
- If user input materially improves a major decision, ask one question at a time. Use `request_user_input` when available and include:
  - 2-3 mutually exclusive options
  - explicit `0.0-10.0` weighted scores for each option
  - one recommended option first

## Tool posture

- Use `update_plan` for nontrivial docs-alignment runs so the workstream stays explicit and checkable.
- Use `request_user_input` for major authority, scope, or artifact decisions that repo evidence cannot resolve cleanly. Do not ask broad free-form question batches.
- Use parallel read-only retrieval when safe so repo discovery stays fast.
- Use subagents only when available and only for bounded exploration, evidence gathering, doc/API verification, grep/path/file audits, focused review, or validation triage. Keep the main authority decision, final synthesis, and default edits local.
- Use built-in `web.*` tools or MCP research when current repo/docs evidence is insufficient, stale, or the task explicitly asks for external verification.
- Use tool or plugin discovery only when the needed capability is not already known in the current session.
- Use image or PDF inspection only when the source of truth is visual, such as screenshots, rendered rubrics, scanned docs, or PDF-only requirements.

## Subagent contract

- Default delegation scope: exploration and evidence only. Do not fan out full doc authorship by default.
- Prefer `1-3` focused subagents over broad fan-out. Do not spawn nested subagents unless the user explicitly asks.
- Keep explorer-style agents read-heavy and evidence-first. Use implementation-capable workers only for narrow follow-on tasks after the main agent has already chosen the authority path.
- For every `spawn_agent` call, explicitly set `model` and `reasoning_effort`.
- In the main skill body, prefer a durable cheap-first policy: use a small model first for bounded exploration, tighten the task before escalating, and escalate only the specific underfitting subagent. Use the exact model ladder and prompt template in `references/subagent-orchestration.md`.
- Every delegated task must specify:
  - narrow task or question
  - allowed scope or surfaces
  - whether the agent is read-only or may edit
  - whether the main agent should wait immediately or continue local work until a synthesis gate
  - exact return format
- Default wait policy: continue local non-overlapping work, then wait at explicit synthesis gates before major authority decisions, final recommendations, or edits that depend on delegated evidence.
- Require evidence-first returns with:
  - key finding or result
  - files and symbols inspected
  - commands or checks run, if any
  - recommended next action
  - unresolved questions or risks
- If delegated findings conflict, surface the conflict explicitly and resolve it in the main-agent synthesis before acting.

## Workflow

### 1. Normalize the job

Extract:
- whether the task is `plan-only`, `plan-then-execute`, or `audit-only`
- whether the user wants a durable repo artifact such as an exec plan/checklist
- whether the request explicitly mentions `AGENTS.md`, README, ADRs, specs, runbooks, requirements docs, or code comments
- whether compression/token optimization is in scope

If the user asked for a very specific response format, preserve it exactly.

If the repo is dirty or the task is branch-specific, anchor the run on the current worktree first:
- inspect `git status --short`
- inspect changed paths with `git diff --name-only`
- use changed code and docs to identify the active functionality and authority surfaces
- from that anchor, sweep all related docs in the repo that may need updates, corrections, supersession, rewrites, or new coverage
- treat the worktree as the starting signal for related-doc discovery, not as the outer boundary of the docs review
- feel free to inspect, plan, and edit docs that are untouched in the worktree when they are part of the same functionality, workflow, contract, or authority chain
- continue the sweep until repo-wide documentation for the affected functionality is current and no stale related guidance remains

### 2. Inventory the repo surfaces

Inspect the smallest high-signal set first:
- root `AGENTS.md`
- nearest scoped `AGENTS.md` files
- `README.md` and docs indexes
- repo-local docs hub or status authority files such as `docs/README.md`, `requirements.md`, release indexes, execution catalogs, or machine-readable ledgers when present
- ADR/spec/runbook directories
- requirements, standards, policy, or similar governing docs when present
- recently changed files and nearby comments/docstrings

Use `rg`/repo-native search to map likely impacted docs before editing. If independent read-only discovery can be parallelized, do that before synthesis. If delegation is available and helpful, use lightweight explorer subagents for repo mapping only after you know what evidence each one should gather.
When the repo has many docs, use docs hubs, status ledgers, execution catalogs, and changed functionality to prioritize the review order, but do not treat that prioritization as a coverage limit.

### 3. Route into the right supporting skills and plugins

Prefer repo-native or user-named skills first. Adapt instead of assuming a stack.

Examples:
- Use `$agents-md-maintainer` before finalizing any `AGENTS.md` edit.
- Use `$technical-writing` when drafting or rewriting ADRs, specs, runbooks, migration docs, or internal guides.
- Use `$caveman-compress` only when the doc surface fits the compression policy in `references/adaptive-compression.md`.
- Use `$hard-cut` and `$clean-code` when simplifying stale doc structure or removing superseded guidance.
- Use stack/platform skills or plugins such as `$github`, `$vercel`, `$expo`, `$sentry`, Context7, or built-in web search only when repo context or the user request makes them relevant.

If a named skill/plugin is unavailable, say so briefly and continue with the closest valid fallback.

### 4. Build a drift map before proposing changes

Compare current docs against:
- implemented behavior
- current scripts/commands
- current architecture and file ownership
- current validation flow
- branch-specific changes that made existing docs stale
- all related documentation in the repo that describes, constrains, teaches, operates, validates, or routes the affected functionality

Good delegation targets here:
- one explorer for `AGENTS.md` and scoped guidance drift
- one explorer for ADR/spec/runbook/README ownership mapping
- one explorer for external doc/API verification when repo evidence is insufficient, using built-in `web.*` tools where web search is needed

Classify each finding:
- `update-in-place`
- `create-canonical-doc`
- `mark-superseded`
- `delete-stale-guidance`
- `leave-unchanged`

Do not propose new docs until you confirm an existing authority doc does not already own the concern.
Do not stop at the first matching doc. Follow the authority chain across README hubs, AGENTS guidance, requirements, ADRs, specs, runbooks, setup docs, release docs, prompt catalogs, and nearby comments/docstrings until the related documentation set is aligned.
Map every proposed doc or comment change to the exact file, path, or code-comment surface it fixes. If you cannot name the target surface, the proposal is not grounded enough yet.

### 5. Choose the canonical authority path

Use the authority matrix in `references/doc-surfaces-and-authority.md`.

Decision rules:
- Prefer modifying the current canonical document over creating a new one.
- Create a new ADR/spec/runbook only when the concern is materially new and does not fit the current authority surface.
- Keep `AGENTS.md` limited to durable repo guidance, never task logs or branch narration.
- If the repo already publishes a docs-role map, status ledger, or execution catalog, treat that as a first-class authority input before inventing new placement.
- If the request needs long-lived execution context, create or update one checkable repo-local exec artifact using existing repo naming conventions.

### 6. Produce the durable exec artifact when needed

When the task asks for a future-session handoff, create or update one canonical plan/checklist artifact that includes only the sections that fit:
- scope and intent
- summary of completed work
- files and surfaces reviewed
- what is already done
- remaining tasks and subtasks
- further improvements worth considering
- required research
- decisions made and open decisions
- validation commands and success criteria
- required skills/plugins/tools
- exact files or directories to load next session
- enforced rules or invariants that the next session must preserve
- blockers and assumptions

Keep it execution-oriented, not a diary.
If the repo already has an execution catalog, prompt ledger, or trigger-prompt system, update that canonical surface instead of creating a parallel plan file.

### 6a. Hidden working artifact policy

For non-canonical working artifacts generated by this skill, default to:
- `.agents/<skill-name>/YYYY-MM/MM-DD/NN/`

When this skill is installed as `repo-docs-align`, that resolves to `.agents/repo-docs-align/YYYY-MM/MM-DD/NN/`.

Use this hidden work area for things like:
- `drift-map.md`
- `reviewed-surfaces.md`
- `exec-plan.md`
- `retrospective.md`
- other temporary or session-oriented analysis files that support docs alignment

Rules:
- create the directory when needed
- use a fresh numeric run bucket such as `01`, `02`, `03` for repeated same-day runs
- ensure the repo ignores `.agents/` or, at minimum, `.agents/<skill-name>/`
- keep canonical docs, ledgers, specs, ADRs, runbooks, and active execution catalogs in their true authority surfaces, not in `.agents/<skill-name>/`
- use typed filenames rather than one giant omnibus note when multiple artifacts are materially different
- only create the artifacts that are actually useful for the run; do not generate empty scaffolding

When you want deterministic scaffolding for this hidden work area, use:

```bash
python3 ./scripts/new_repo_docs_align_artifact.py \
  --dir <repo-root> \
  --artifacts drift-map,reviewed-surfaces,exec-plan,retrospective
```

Run that exact command from the installed skill directory. The script resolves bundled templates relative to itself, so the relative path stays unambiguous wherever the skill is installed.

Use `--artifacts` to request only the files needed for the run. Use `--force` only when intentionally refreshing an existing artifact file.

### 7. Implement doc and comment changes when the task calls for execution

After the drift map and authority decisions are grounded:
- update canonical docs
- tighten or remove stale guidance
- align nearby code comments/docstrings where useful
- make the smallest set of edits that fully resolves the grounded drift; do not leave partially corrected authority chains behind
- keep diffs minimal and reviewable

Do not rewrite unrelated docs just because they are imperfect.

### 8. Apply adaptive compression only where it improves the repo

Follow `references/adaptive-compression.md`.

Default posture:
- compress internal operational, agent-facing, workflow, and repo-maintenance docs when it improves scan speed and token efficiency
- preserve richer prose for public, product, marketing, narrative, or teaching-oriented docs unless the user explicitly requests compression

When compressing:
- preserve code, commands, paths, URLs, headings, tables, and exact technical terms
- keep the document navigable
- do not cavemanify docs whose main value is nuanced explanation or polished prose

### 9. Verify before finalizing

Verify that:
- every proposed doc change maps to a specific file or confirmed gap
- every reviewed doc surface that remains unchanged has a reason, explicit or implicit, grounded in current repo reality
- authority choices match the current repo structure
- requested deliverables are complete
- formatting is consistent with the surrounding docs
- referenced commands, scripts, and paths still exist
- any irreversible or external side effect is surfaced before execution

If you changed `AGENTS.md`, run a brief `$agents-md-maintainer` pass before closeout.

## Output shape

Default order unless the user asked for another format:
1. drift summary
2. canonical authority decisions
3. exec artifact path or inline plan
4. implemented doc/comment changes
5. verification commands and residual gaps

## Stop rules

- Stop and ask only when a major authority decision remains genuinely ambiguous and repo evidence cannot resolve it.
- Label missing evidence or uncertain claims as `UNVERIFIED`.
- If retrieval is empty or suspiciously narrow, retry with one or two different strategies before concluding.
