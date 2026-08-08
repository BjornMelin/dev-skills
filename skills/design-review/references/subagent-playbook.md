# Design Review Subagent Playbook

Use bounded fan-out only when the diff is broad enough to benefit from
independent review lanes or the user explicitly requests delegation. The parent
agent owns scope, synthesis, corrections, re-verification, and the final
verdict.

## Host-Agnostic Pattern

Use the host's native read-only agent mechanism:

- Claude Code may load project roles from `.claude/agents`.
- Codex may load roles from `.codex/agents`.
- OpenCode may use task subagents with lane-specific prompts.

Role files are optional. The durable pattern is: bounded scope, non-mutating
review, shared evidence contract, complete wait, then parent-owned synthesis.
Do not require a repository to install one host's fleet format.

## When to Fan Out

Fan out when at least one applies:

- multiple user-facing platforms or routes changed;
- the branch combines visual, motion, accessibility, and performance risk;
- a flagship redesign needs independent taste and runtime challenges;
- native and web evidence must be gathered separately;
- the user requests an adversarial or multi-agent review.

Stay single-agent for a small component, copy-only change, or one narrow defect.
Delegation overhead is not evidence.

## Suggested Read-Only Lanes

Choose only applicable lanes, normally one to three at a time:

| Lane | Assignment |
| --- | --- |
| Taste and product | Find template aesthetics, weak hierarchy, refused vocabulary, unclear One Thing, and neglected back-of-fence states. |
| Accessibility and metadata | Verify WCAG 2.2 AA risks, keyboard/focus/forms/contrast, reduced motion, and page metadata. |
| Motion and performance | Inspect animation ownership, curves, SSR/reduced-motion, runtime smoothness, bundles, and Core Web Vitals. |
| Runtime evidence | Exercise exact flows in a browser, simulator, or device; capture screenshots, console/network state, and measurements without editing. |
| Entropy and architecture | Trace duplicate components/tokens/styles, fallback paths, and stale ownership. |
| Data or native specialist | Use only when the diff touches that boundary. |

Do not spawn every lane by default. Avoid multiple lanes that merely restate the
same visual judgment.

## Prompt Contract

Every lane prompt must include:

1. **Objective:** one sentence naming the review question.
2. **Exact scope:** changed files, routes, components, and diff range.
3. **Exclusions:** unrelated dirty work and lanes owned elsewhere.
4. **Authorities:** repository design docs, token/theme sources, component
   registry, and the generic gates that apply when doctrine is absent.
5. **Evidence rule:** a finding requires Contract + Runtime + Correction and a
   `file:line` citation; unsupported claims are `UNVERIFIED`.
6. **Runtime allowance:** which browser/device/measurement tools may be used.
7. **Mutation rule:** read-only unless the parent explicitly assigns a
   non-overlapping fix scope after synthesis.
8. **Return schema:** Status, Coverage, Evidence, Findings, Rejected candidates,
   Commands, and Risks/blockers.
9. **No nesting:** the lane must not spawn more agents or broaden scope.

Example prompt skeleton:

```text
Review <scope> for <lane>. Remain read-only.
Authorities: <paths and generic gates>.
Exclude: <unrelated paths and other lanes>.
For each finding provide severity, Contract, Runtime, Correction, and file:line.
Mark anything unproven UNVERIFIED. Return Status, Coverage, Evidence, Findings,
Considered-but-rejected, Commands, and Risks. Do not spawn subagents.
```

## Concurrency Rules

- Parallelize only independent read-only lanes.
- Never allow concurrent writers in one checkout.
- If fix workers are used later, assign mutually exclusive files and keep one
  owner for shared tokens, generated output, and docs.
- Give every lane the same frozen diff/scope snapshot when possible.
- If the parent changes the worktree while lanes run, invalidate stale runtime
  or line-number evidence and rerun the affected lane.

## Wait Rules

- Launch an independent batch together, then wait for every spawned lane.
- Do not synthesize, edit, or launch a fix lane while evidence lanes remain
  outstanding unless the host guarantees the snapshot cannot change.
- Treat a timed-out or failed lane as degraded coverage, not an empty finding
  set.
- Do not let one confident lane substitute for a missing independent lane.

## Parent Synthesis Checklist

1. Confirm every expected lane returned or mark it degraded.
2. Reject findings missing Contract, Runtime, Correction, or `file:line`.
3. Deduplicate by root cause and affected owner, not wording.
4. Resolve conflicts against repository doctrine and runtime evidence.
5. Rank hard failures, then task clarity/a11y, theme and state quality, motion
   and performance, entropy, and docs.
6. Record one to three plausible candidates that were considered and rejected.
7. In implementation mode, patch the canonical owner and delete superseded
   paths; do not distribute overlapping fixes.
8. Re-run affected runtime proof after all edits.
9. Emit exactly one verdict: Approve, Needs changes, Block, or Inconclusive.

Subagents provide adversarial evidence. They never own the final product
decision or approval claim.
