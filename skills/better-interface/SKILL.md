---
name: better-interface
description: "Cross-discipline interface review and build that coordinates better-accessibility, better-layout, better-writing, better-typography, better-colors and better-ui. Use for a holistic UI audit, a full interface review, or a cross-discipline design review of a screen, flow, feature, or product interface, or to build one with the same rules applied forward. Triggers on better-interface, full interface review, holistic UI audit, cross-discipline design review, review the whole interface. Modes: quick, core, full, build."
license: MIT
metadata:
  version: "1.0.0"
---

# Review the interface as one system

A strong interface is not six independent audits stapled together. Review the whole experience, let each `better-*` skill own its domain rules, then consolidate the evidence into one prioritized verdict. The same ownership runs forward: in `build` mode you consult the owners while implementing, then review your own diff against them.

This skill owns orchestration only. Accessibility rules belong to `better-accessibility`; structure to `better-layout`; copy to `better-writing`; type to `better-typography`; color to `better-colors`; visual polish and motion to `better-ui`. Never duplicate or override their rules here.

## Core Principles

### 1. Resolve Scope and Mode First

Infer the screen, flow, feature, or repository scope from the request and current workspace. State the resolved scope in the output. Use `core` when no mode is supplied and the request is a review; use `build` when the request is to implement something. `full` is opt-in — it dispatches a judge lane per domain with candidates and costs accordingly.

| Mode | Coverage | Finding cap |
| --- | --- | --- |
| `quick` | Primary user path and highest-traffic states. Detect across all six domains; judge inline, no lanes. Report only `HIGH` and `MEDIUM` | 5 |
| `core` | Detect across all six domains, then judge **at most two** — the domains whose candidates look most severe. Unjudged domains are reported `Detected only` | 8 |
| `full` | Entire requested scope, including empty, loading, error, and narrow-width states when present. Detect across all six domains and judge **every domain that produced candidates** — no cap | 15 |
| `build` | Implement the requested change, then self-review the diff against every domain in play | n/a |

`core` is the deliberately cheap tier and is the only mode that leaves candidates unjudged.
`full` means fully reviewed: if a domain produced candidates, it gets a judge lane.

If the requested scope is too large to inspect credibly, narrow it to the highest-traffic complete flow and state the boundary. Never imply uninspected surfaces were reviewed.

### 2. Recon Before Judgment

Identify the framework, styling system, component library, design tokens, supported viewports, and available preview or test commands. Follow the project's established Tailwind, plain CSS, CSS-in-JS, token, and component conventions.

Recon output is reusable. Pass it into every lane you dispatch so no lane re-derives the stack, and reuse it unchanged in `build` mode.

### 3. Use Domain Skills as the Sources of Truth

Before reviewing, confirm that all six owning skills below are available. Load and apply every available owner. All four modes sweep all six domains for candidates; they differ only in how many of those domains get a judge lane, per the table in Principle 1. In `full` mode, every domain that produced candidates is judged before consolidation.

Review in this order so foundational failures are not hidden by polish:

1. `better-accessibility`
2. `better-layout`
3. `better-writing`
4. `better-typography`
5. `better-colors`
6. `better-ui`

This skill owns the final response. When a domain skill is loaded through `better-interface`, apply its principles, its `## Severity` calibration, and its references, but ignore its standalone **Review Output Format**. Use the consolidated format, shared severity, and finding cap in this file instead.

If an owning skill is unavailable, mark that domain `Not reviewed`, name the missing skill, and continue with the remaining domains. Do not recreate its rules from memory, substitute a neighboring skill, or claim holistic coverage.

When two skills appear to cover the same issue, assign it to the skill that owns the underlying rule and mention secondary effects in the **Why** cell. Report it once.

### 4. Give Each Domain Its Own Context

Six rule sets plus the project's code crowd one context, and an invoked skill stays resident for the rest of the session. Keep domain rules out of this context whenever the host allows it.

Split the work by task type, not by domain. Every domain decomposes the same way:

| Tier | Nature | Where it runs |
| --- | --- | --- |
| **Inventory** | Enumerate artifacts — interactive elements and their accessible names, rendered color pairs, type declarations, user-facing strings, breakpoints. Every claim carries `path/to/file:line`. No judgment. | The cheapest runtime that can read the tree |
| **Detect** | Apply a written, checkable rule to the inventory. Produces candidates, not findings: no severity, no rewrite. | A dispatched lane, or inline |
| **Judge** | Kill false positives against the source, assign severity, write the Before → After. | An `opus` lane, or this skill inline |
| **Consolidate** | Dedupe by root cause, rank, cap, record restraint, emit the verdict. | Always this skill, inline |

**Dispatching a lane is not cheap. Measure before you assume.** A single detect lane over one
484-line component — reading the component, its four UI primitives, and the installed Radix
source — cost roughly **150k tokens and 14 minutes** at maximum reasoning effort. Six of those
is most of a million tokens. What a lane buys is *depth and a clean context*, not savings:
that run traced into `node_modules` to check what the dialog primitive actually renders, which
an inline pass sharing context with five other rule sets will not do.

So dispatch is a deliberate spend, and the mode ladder is a cost ladder:

- `quick` and `core` are cheap because they **dispatch little or nothing**, not because
  detection is inherently cheap. `quick` runs inline. `core` judges at most two domains and
  must mark the rest `Detected only` in the coverage table — never `Clear`, because an
  unjudged candidate is not the same as no candidate.
- `full` judges every domain that produced candidates and is expensive by design. Reach for it
  on a surface that matters, not as a default sweep.

**Effort buys traversal depth, not polish.** The same lane, same file, measured at both tiers:

| Reasoning effort | Wall clock | Tokens | Candidates |
| --- | --- | --- | --- |
| `high` | ~4 min | ~75k | 2 of 6 |
| `max` | ~14 min | ~153k | 6 of 6 |

The four the fast lane missed were not tail noise. It never entered `node_modules`, so it
missed both findings that required reading what the dialog primitive actually renders —
including a high-confidence one, that focus never returns to the trigger because the ref the
library restores through is unset. It also missed a 16×16 hit area inside a file it had
already read.

So match effort to mode rather than trying to economise inside a mode. `full` dispatches at
maximum effort because tracing into a dependency is the thing that justifies dispatching at
all. `quick` and `core` accept a shallower read by design and can run at a lower tier — a lane
that only reads the component and its direct imports still catches the obvious failures at a
third of the wall clock. What you must not do is run `full` at a fast tier and report it as
full coverage.

Never send interface copy, visual design, motion, or naming decisions to a non-Claude model for judgment: those are taste calls. Detection of mechanical copy defects — terminology drift, inconsistent capitalization, non-verb-first labels, a placeholder restating its label — is a lint and may run anywhere.

If the host cannot run lanes, do the same passes inline in the review order above and say so in **Scope and Coverage**. All six rule sets stay resident on that path, so prefer `quick` or a narrowed scope there.

### 5. Contract Every Lane

Each lane prompt states: the resolved scope, the recon results from Principle 2, the one domain skill it must load, whether it is read-only, and the exact shape it must return. Lanes gather evidence; this skill decides.

There are three distinct payloads, and using the wrong one is the most common way this breaks:

| Stage | Emits | Schema |
| --- | --- | --- |
| Inventory + detect | **Candidates** — `locations`, `observed`, `rule`, optional `measurement`. Explicitly **no severity and no fix**; assigning either is judgment | [candidate-schema.json](references/candidate-schema.json) |
| Judge | **Findings** — the same root causes verified against source, now with `severity` and an actionable `after` | [findings-schema.json](references/findings-schema.json) |
| Consolidate | The merged report: coverage for all six domains, deduped findings, rejected candidates, **verification**, one verdict | [verified-schema.json](references/verified-schema.json) |

Every payload also carries `Domain`, `Evidence` (what was actually inspected), `Rejected` (candidates deliberately not raised, with the reason), `Verification`, and `Blocked` (anything in scope it could not inspect, or `None`). A detect or judge lane emits no verdict and applies no cap — six lanes each emitting `Block` is six reports, not one.

`Rejected` is part of every contract because restraint cannot survive a lane boundary otherwise: a lane that discards its rejected candidates makes Principle 10 impossible to satisfy honestly. `Verification` is carried through consolidation for the same reason — the report mandates that section, so it cannot be dropped at the last hop.

### 6. Lane Failure Is Reported, Never Absorbed

- A lane that errored or returned unusable output is **degraded coverage**. Say so explicitly; the remaining lanes' findings stand alone.
- **`Approve` requires complete coverage.** If any domain is `Degraded`, `Not reviewed`, or `Detected only`, the verdict is `Inconclusive` even when every live lane came back clean — the unreviewed domain is exactly where the unfound problem would be. Name the missing domains and what it would take to close them.
- If every lane failed, there is **no verdict**. Report the failure and stop.
- Zero candidates across *all six* domains, with none degraded, is a real result: report `Approve` with no findings.

### 7. Require Evidence

Every finding cites `path/to/file:line` and shows the current implementation. If the review artifact has no source files, cite the exact screen and component. Do not report a code-level finding from visual appearance alone or a visual finding from source code alone when runtime behavior determines the result.

### 8. Rank by User Impact

Use one shared severity scale, calibrated per domain by that domain skill's own `## Severity` section:

- `HIGH`: blocks a task, misleads the user, hides content or controls, causes data-loss risk, or creates a repeated systemic failure.
- `MEDIUM`: meaningfully harms comprehension, efficiency, adaptability, or consistency.
- `LOW`: isolated polish with limited task impact. Include only in `full` mode.

Within a severity, rank by reach and leverage. A token or shared-component fix outranks the same symptom in one leaf component.

### 9. Consolidate Systemic Findings

One root cause is one finding. List every confirmed location in the same row rather than producing a row per occurrence. Do not pad the report to reach the finding cap; a short review or no findings is a valid result.

### 10. Make Restraint Visible

Record candidates considered but deliberately rejected. A candidate is rejected when the owning skill permits the current implementation, evidence is insufficient, the project convention is intentional, or the proposed change would add complexity without user benefit.

### 11. Verify What Can Be Verified

Run safe, relevant checks available in the project. Inspect the rendered interface when runtime behavior or visual judgment matters. Report the exact command or interaction and observed result. If a check cannot be run, label it **Not verified** and state what remains; never convert a verification gap into a finding.

### 12. Review Without Mutating by Default

Treat a review request as read-only. Do not edit source code unless the user also asks to implement the findings, or the mode is `build`. When implementation is requested, preserve the consolidated report as the change scope and re-run the relevant verification afterward.

### 13. Build With the Same Owners

`build` mode runs the ownership table forward instead of backward.

1. **Recon** per Principle 2, and identify which domains the change actually touches. A form touches accessibility and writing; a marketing section touches typography, layout, and color. Name them before writing code.
2. **Consult** each owner in play *before* implementing, not after. Their principles are the spec: the type scale, the hit-area floor, the token system, the copy voice. Building first and auditing second produces rework.
3. **Implement** in the project's existing styling system and component library. Never introduce a second styling approach.
4. **Self-review the diff** against every domain you named, using the same severity scale. Fix what you can fix immediately; report what you deliberately leave as `Considered but Rejected` with the reason.
5. **Verify** per Principle 11 and report what you ran.

Output the consolidated report after the change, scoped to the diff rather than the whole surface, and end with the same verdict ladder.

## Common Mistakes

| Mistake | Fix |
| --- | --- |
| Six disconnected domain reports | Consolidate into one ranked findings table |
| Same issue reported by multiple skills | Assign it to the skill that owns the underlying rule |
| Finding with no exact location | Cite `path/to/file:line` and the current implementation |
| Visual claim inferred only from source | Inspect the rendered state or mark it not verified |
| Unlimited low-impact polish | Respect the mode cap; omit `LOW` findings in `quick` |
| Silent gaps in coverage | Show which domains and states were actually inspected |
| Missing owning skill silently treated as covered | Mark the domain `Not reviewed` and name the unavailable skill |
| All six rule sets loaded into one review context | Detect across all six, dispatch judge lanes only where candidates exist |
| A failed lane quietly omitted from the report | Report degraded coverage; a total lane failure is no verdict, never `Approve` |
| Lane output pasted through as the report | Consolidate every lane under this skill's cap and verdict |
| Taste judgment delegated to a non-Claude model | Detection may run anywhere; copy, visual, motion, and naming calls stay here |
| No rejected candidates | Include the required considered-but-rejected table |
| Review silently edits code | Stay read-only unless implementation was requested or the mode is `build` |
| Building first and consulting the domain skills afterward | Consult the owners in play before writing code |
| "Approve" with pending actionable findings | Use `Needs changes` or `Block` |

## Review Output Format

Always use the following sections.

### Scope and Coverage

State the mode, exact scope, stack and styling conventions, how the work was dispatched (inline or lanes), and any review boundary. Then show coverage:

| Domain | Evidence inspected | Result |
| --- | --- | --- |
| Accessibility | Files, components, states, or checks | Findings count or `Clear` |

Include all six domains. `Clear` means inspected with no actionable finding; `Detected only` means the domain was swept for candidates but no judge lane ran under the mode's cap; `Not reviewed` must explain why.

### Findings

Use one table ordered by severity, then reach and leverage:

| # | Severity | Domain | Location | Before | After | Why |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | HIGH | Accessibility | `src/Dialog.tsx:42` | `<button><XIcon /></button>` | Add `aria-label="Close"` and hide the icon from the accessibility tree | The icon-only control has no accessible name |

Each row is one root cause. The **Domain** value is the owning skill without the `better-` prefix. Respect the mode's finding cap. If there are no findings, omit the table and state "No actionable interface findings."

### Considered but Rejected

Include 1–3 candidates in `quick` mode and 2–5 in `core`, `full`, and `build`:

| Location | Candidate | Rejected because |
| --- | --- | --- |
| `src/Card.tsx:28` | Increase the shadow | Existing depth matches the shared surface token; changing one card would reduce consistency |

These are real candidates inspected during the review, not invented filler. If the scope genuinely contains fewer borderline candidates, include the ones that exist and say so.

### Verification

List each check or interaction, the exact command or steps, and the observed result. Separate checks that passed from checks marked **Not verified**.

### Verdict

End with exactly one:

- `Block` — one or more `HIGH` findings remain.
- `Needs changes` — only `MEDIUM` or `LOW` findings remain.
- `Inconclusive` — no actionable findings in what was judged, but at least one domain is `Degraded`, `Not reviewed`, or `Detected only`. Name them.
- `Approve` — no actionable findings remain and every one of the six domains was inspected and judged.
- `No verdict` — every lane failed.
