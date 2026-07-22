---
name: multi-model-review
description: Pre-PR multi-model review of a nontrivial diff - parallel opus-4.8 review lane (Agent) + gpt-5.6-sol adversarial review lane (direct codex exec, no shim), then adversarial verification of merged findings. Read-only; the main loop presents/synthesizes and applies fixes only on request. Use before shipping nontrivial changes.
---

# Multi-Model Review

Two independent review lanes run in parallel, then every merged finding is
adversarially verified against the actual code. Replaces the retired
`multi-model-review` workflow: per MODELS.md there is **no Claude shim** - the
main loop (Fable) runs the codex lane itself via direct background Bash.

Inputs: `repo` (absolute path, required), `base` (default `main`), `focus`
(default: correctness, security, edge cases, API contracts, maintainability).

Shared review contract (put in BOTH lane prompts):

> Repo: `<repo>`. Review the diff `git diff <base>...HEAD`; if that diff is
> empty, review staged+unstaged changes instead. Focus: `<focus>`.
> You are READ-ONLY: do not edit any file. Read the changed files fully - not
> just the diff hunks. Report findings with file:line, why each matters, and
> the precise fix. If the change is clean, use verdict "ship" with zero
> findings - do not invent issues.

## Phase 1 - launch both lanes in parallel (same message)

**Opus lane** - `Agent(model: 'opus', effort: 'high', run_in_background: true)`.
Prompt = shared contract + "You are the Claude reviewer lane. Set reviewer to
\"opus-4.8\". Return ONLY a JSON object matching
`~/.claude/skills/multi-model-review/references/findings-schema.json`
(read it first) - no prose around it."

**Codex lane** - direct `codex exec`, no relay agent:
1. Fill `~/.claude/skills/codex-review/references/adversarial-prompt.md`:
   `TARGET_LABEL` = "diff vs <base> in <repo>", `USER_FOCUS` = focus,
   `REVIEW_COLLECTION_GUIDANCE` = "Collect the diff yourself with git
   (git diff <base>...HEAD, falling back to uncommitted changes if empty) and
   read changed files fully.", `REVIEW_INPUT` = "Use git and file reads in the
   repo working directory." Write it to `<scratchpad>/mmr-prompt.md`.
2. Run ONE bare background Bash command (600000ms timeout):

```bash
codex exec -m gpt-5.6-sol -c model_reasoning_effort="medium" -s read-only --cd <repo> --output-schema /home/bjorn/.claude/skills/multi-model-review/references/findings-schema.json --output-last-message <scratchpad>/mmr-codex-findings.json - < <scratchpad>/mmr-prompt.md
```

Effort routing per MODELS.md: `"medium"` (Sol worker) is the default review
tier; pin `"high"` for consequential or cross-cutting diffs. Never xhigh.

3. On completion, read `mmr-codex-findings.json`; set reviewer to
   "gpt-5.6-sol" if absent.

## Phase 2 - lane failure semantics (never skip)

- A lane that errored or returned unusable output = **degraded coverage**:
  say so explicitly in the final report; the other lane's verdict stands alone.
- BOTH lanes failed = **no verdict**. Report the failure and stop - a total
  lane failure must never read as a clean "ship".
- Zero raw findings across live lanes = verdict "ship"; skip Phase 3.

## Phase 3 - adversarial verify

Spawn one verify agent - `Agent(model: 'opus', effort: 'high')` - with the
shared contract, the merged lane JSON, and:

> Adversarially VERIFY each finding against the actual code: open the cited
> file:line, confirm the claim is real on THIS diff (not stale, hypothetical,
> or about pre-existing code), dedupe overlapping findings across lanes (set
> confirmedBy to "both" when lanes agree - that raises confidence), and reject
> false positives with concrete evidence. Order confirmed findings by
> severity. Overall verdict: "blocker" if any confirmed critical/high,
> "changes-recommended" if any confirmed finding remains, else "ship".
> Return ONLY a JSON object matching
> `~/.claude/skills/multi-model-review/references/verified-schema.json`.

For a tiny finding set (≤3), Fable may verify inline instead of spawning.

## Presenting results

- Lead with the overall verdict, then confirmed findings ranked by severity,
  verbatim in substance; list rejected findings with their rejection reasons.
- Flag degraded coverage prominently if a lane failed.
- **Never auto-apply fixes.** The main loop applies fixes only when the user
  asks or an approved implementation task already covers them.
