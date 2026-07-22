---
name: codex-review
description: Independent gpt-5.6 diff review via the Codex CLI, normal or steerable adversarial with JSON findings. Use before shipping nontrivial changes.
---

# Codex Review

Independent gpt-5.6 review of local changes through the Codex CLI. Read-only - never
lets Codex modify the repo. Works identically from the main loop, subagents, and
workflow stages (no plugin runtime required).

Reviews run on `gpt-5.6-sol` (MODELS.md: code review = Sol) - pin BOTH the model
(`-m gpt-5.6-sol`) and the effort (`-c model_reasoning_effort="medium"`, or
`"high"` for consequential/cross-cutting diffs) on every command below rather
than inheriting CLI defaults. For `codex review`, `-m` is a GLOBAL flag and must
come before the subcommand. Paths written as `<skill-dir>` mean this skill's
base directory (provided when the skill is invoked).

## Mode selection

- **Normal review** (default): overall code-quality pass on a diff, same quality as
  running `/review` inside Codex.
- **Adversarial review**: when the user (or you) wants a specific decision, risk area,
  or design pressure-tested. Steerable with focus text; returns structured JSON.

## Normal review

```bash
codex -m gpt-5.6-sol -c model_reasoning_effort="medium" review --uncommitted   # staged + unstaged + untracked
codex -m gpt-5.6-sol -c model_reasoning_effort="medium" review --base main     # branch vs base
codex -m gpt-5.6-sol -c model_reasoning_effort="medium" review --commit <sha>  # a single commit
```

- Multi-file/large diffs take minutes: run via Bash `run_in_background: true`; the
  harness notifies on completion (no polling needed).
- Run as ONE bare command - no pipes, no `cd &&` chains (keeps the RTK hook inert).

## Adversarial review

1. Build the prompt from `references/adversarial-prompt.md`: fill `{{TARGET_LABEL}}`
   (e.g. "diff vs main"), `{{USER_FOCUS}}` (focus text or "general adversarial pass"),
   `{{REVIEW_COLLECTION_GUIDANCE}}` (usually: "Collect the diff yourself with git; read
   changed files fully."), and `{{REVIEW_INPUT}}` (either the diff inline, or an
   instruction to run `git diff <base>...HEAD`). Write it to the scratchpad.
2. Run as a single bare command, feeding the prompt file via stdin (`-` reads the
   prompt from stdin; a plain `< file` redirect keeps the command pipe-free):

```bash
codex exec -m gpt-5.6-sol -c model_reasoning_effort="medium" -s read-only --cd <repo> --output-schema <skill-dir>/references/review-output.schema.json --output-last-message <scratchpad>/codex-adversarial-<ts>.json - < <scratchpad>/prompt.md
```

3. Read the output JSON: `verdict` (`approve` | `needs-attention`), `findings[]`
   (severity, title, body, file, line_start/line_end, confidence 0–1, recommendation),
   `next_steps[]`.

## Presenting results

- Present findings ranked by severity, verbatim in substance - do not soften.
- **Never auto-apply fixes from a review.** Present first; fix only when the user asks
  or when you already have an approved implementation task that covers it.
- Note the session can be reopened in Codex with `codex resume` if deeper follow-up
  is wanted.
- If Codex disagrees with a deliberate design choice, report the disagreement and your
  own position - don't silently adopt either side.

## Effort

Always pin effort explicitly - sol's vendor default is `low`. Per MODELS.md:
`"medium"` (Sol worker) is the standard review tier; `"high"` (Sol lead) for
consequential or cross-cutting diffs. **Never xhigh or ultra.** If a Sol-high
review still leaves material doubt, escalate per MODELS.md: Fable resolves the
hard part inline, or one independent `gpt-5.6-terra` `"max"` adversarial pass.

References adapted from openai/codex-plugin-cc (Apache-2.0; see references/NOTICE
and the full license text in references/LICENSE).
