# Subagent Workflow Recipes

Use this reference when creating repeatable workflows around the bundled Codex
custom agents. Keep runtime orchestration in `$subspawn`; keep reusable role
files in this skill.

## PR Review Recipe

Purpose: high-signal review with false-positive filtering.

1. Eligibility:
   - confirm the PR/diff is current
   - skip closed, draft, trivial, or already-reviewed work when applicable
2. Context:
   - `guidance_mapper`: find relevant `AGENTS.md`, `CLAUDE.md`, README, and
     scoped instructions for changed paths
   - `repo_explorer`: summarize the changed surface and likely owner modules
3. Parallel review:
   - `shallow_bug_reviewer`: obvious changed-line correctness bugs
   - `security_reviewer`: assigned security-sensitive surfaces only
   - `history_reviewer`: git history and previous intent for changed files
   - `docs_auditor`: docs or comment guidance only when docs are in scope
4. Validation:
   - `false_positive_validator`: score each candidate finding from 0-100
   - discard findings below 80 unless the parent chose another threshold
5. Synthesis:
   - main agent owns final findings and any GitHub comments
   - cite exact files, lines, commands, docs, or source URLs

Do not ask review agents to run broad tests. Use `test_runner` only after the
main agent chooses a focused verification target.

## Audit Recipe

Purpose: broad but non-overlapping risk scan.

1. Scope the audit surface and stop conditions in the parent prompt.
2. Spawn only independent lanes:
   - `security_reviewer`
   - `runtime_bug_reviewer`
   - `dependency_researcher`
   - `performance_reviewer`
   - `docs_auditor`
3. Require each lane to state what it did not inspect.
4. Main agent deduplicates findings and chooses the remediation order.

## Ops Recipe

Purpose: make release or CI status actionable.

1. `ci_triager`: failing job, first meaningful error, likely owner files.
2. `env_validator`: required env vars, examples, and unsafe defaults.
3. `release_validator`: changelog, version, tags, packages, publish gates.
4. `test_runner`: run only the smallest verification command chosen by the
   parent.

## Dispatch Notes

- Prefer custom roles when installed and a role clearly matches.
- Fall back to Codex built-ins when a custom role does not exist.
- Use `$subspawn` strict rendezvous: spawn a planned batch, wait for all
  spawned agents, synthesize, then continue.
- Keep template prompts bounded; put workflow coordination in the parent.
