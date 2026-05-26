---
name: autoreview
description: Run a Codex-only structured code review closeout for local, branch, or commit diffs. Use when the user asks for autoreview, Codex review, structured closeout review, final review before commit/ship, or review after non-trivial code edits.
---

# Auto Review

Run the bundled Codex structured review helper as a closeout check. Treat the
result as advisory: verify every finding against the real code before changing
files, and reject speculative or over-broad findings.

## Contract

- Use only the bundled helper; do not run nested review commands.
- The helper uses `codex exec` instead of `codex review` so it can enforce structured JSON output and deterministic pass/fail behavior.
- Review the intended diff target, not a clean checkout by accident.
- Keep going until the final helper run reports no accepted/actionable findings.
- If a review-triggered fix changes code, rerun focused tests and rerun the helper.
- Report security findings only for concrete, actionable risks introduced or exposed by the change.
- Do not push just to review. Push only when the user requested push, ship, or PR update.

## Pick Target

Dirty local work:

```bash
skills/autoreview/scripts/autoreview --mode local
```

Branch work:

```bash
skills/autoreview/scripts/autoreview --mode branch --base origin/main
```

Committed single change:

```bash
skills/autoreview/scripts/autoreview --mode commit --commit HEAD
```

Use `--mode local` only when the patch is actually unstaged, staged, or
untracked in the current checkout. For committed, pushed, or PR work, review the
commit or branch diff instead.

## Options

- `--model <model>`: pass a Codex model override; omit it to inherit the configured Codex model.
- `--reasoning-effort low|medium|high|xhigh`: pass Codex model reasoning effort; omit it to inherit the configured/model default.
- `--web-search`: opt into Codex web search for dependency/API/security research; default is off to match Codex review behavior.
- `--prompt` / `--prompt-file`: add task-specific review instructions.
- `--dataset <file>`: include extra evidence in the review bundle.
- `--parallel-tests "<command>"`: run focused tests while Codex reviews the frozen bundle.
- `--output <file>` / `--json-output <file>`: persist human or structured output.
- `--dry-run`: print target selection without invoking Codex.

Format first if formatting can change line locations. If tests or review cause
edits, rerun the affected tests and rerun autoreview until the helper exits
cleanly.

## Helper Behavior

The helper:

- chooses dirty local changes first in `--mode auto`
- otherwise uses `origin/main` for non-main branch review
- uses `codex exec` with read-only sandboxing and structured JSON output
- inherits current Codex model selection by default; keep Codex config on the latest best-fit review model and use `--model` only for deliberate overrides
- writes only to stdout unless `--output` or `--json-output` is set
- prints `review still running: codex elapsed=<seconds>s pid=<pid>` while waiting
- prints `autoreview clean: no accepted/actionable findings reported` on a clean result
- exits nonzero when accepted/actionable findings are present

## Final Report

Include:

- review command used
- tests/proof run
- findings accepted/rejected, briefly why
- the clean result from the final helper run, or why a remaining finding was consciously rejected
