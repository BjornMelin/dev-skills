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

`Rejected` is part of every contract because restraint cannot survive a lane boundary otherwise: a lane that discards its rejected candidates makes Principle 11 impossible to satisfy honestly. `Verification` is carried through consolidation for the same reason — the report mandates that section, so it cannot be dropped at the last hop.

**Evidence must be quotable, not merely asserted.** "The primitive does not set `aria-modal`" is a
claim; the grep or the quoted line that shows it is evidence. When a candidate rests on what a
dependency does, name the resolved version — a lockfile can install two copies of the same
package, and "the inspected implementation" is meaningless when there are two.

### 6. What Each Lane May Consult

The authority for a review is **the code as installed**, not the latest documentation. A
project pinned to an older version is correctly reviewed against that version; a finding
sourced from current docs is wrong if the pinned release behaves differently. So access is
asymmetric by stage:

| Stage | May consult | Must not |
| --- | --- | --- |
| Inventory, Detect | The repository and its installed dependencies, including reading inside `node_modules`. The lockfile decides which version is authoritative | External docs. Local source already settles these questions, and fetching adds latency to the stage that can least afford it |
| Judge | The above, plus package source and version-pinned API references to resolve a candidate the source alone cannot settle — is this deprecated in *this* version, is this the intended API | Substituting a docs claim for source evidence when the source is present and readable |
| `build` | Everything, and it should. Writing new code is where currency matters most: current APIs, current Baseline support, whether a pattern is still recommended | Introducing a feature the project's targets do not support without saying so |

When a lane does consult an external source, it cites the source and the version it applies to,
in the same `Verification` entry as the check. A judged finding that rests on documentation
rather than on the code in front of it must say so, so the reader can weigh it.

### 7. Lane Failure Is Reported, Never Absorbed

- A lane that errored or returned unusable output is **degraded coverage**. Say so explicitly; the remaining lanes' findings stand alone.
- **`Approve` requires complete coverage.** If any domain is `Degraded`, `Not reviewed`, or `Detected only`, the verdict is `Inconclusive` even when every live lane came back clean — the unreviewed domain is exactly where the unfound problem would be. Name the missing domains and what it would take to close them.
- If every lane failed, there is **no verdict**. Report the failure and stop.
- Zero candidates across *all six* domains, with none degraded, is a real result: report `Approve` with no findings.

### 8. Require Evidence

Every finding cites `path/to/file:line` and shows the current implementation. If the review artifact has no source files, cite the exact screen and component. Do not report a code-level finding from visual appearance alone or a visual finding from source code alone when runtime behavior determines the result.

### 9. Rank by User Impact

Use one shared severity scale, calibrated per domain by that domain skill's own `## Severity` section:

- `HIGH`: blocks a task, misleads the user, hides content or controls, causes data-loss risk, or creates a repeated systemic failure.
- `MEDIUM`: meaningfully harms comprehension, efficiency, adaptability, or consistency.
- `LOW`: isolated polish with limited task impact. Include only in `full` mode.

Within a severity, rank by reach and leverage. A token or shared-component fix outranks the same symptom in one leaf component.

### 10. Consolidate Systemic Findings

One root cause is one finding. List every confirmed location in the same row rather than producing a row per occurrence. Do not pad the report to reach the finding cap; a short review or no findings is a valid result.

### 11. Make Restraint Visible

Record candidates considered but deliberately rejected. A candidate is rejected when the owning skill permits the current implementation, evidence is insufficient, the project convention is intentional, or the proposed change would add complexity without user benefit.

### 12. Verify What Can Be Verified

Run safe, relevant checks available in the project. Inspect the rendered interface when runtime behavior or visual judgment matters. Report the exact command or interaction and observed result. If a check cannot be run, label it **Not verified** and state what remains; never convert a verification gap into a finding.

### 13. Review Without Mutating by Default

Treat a review request as read-only. Do not edit source code unless the user also asks to implement the findings, or the mode is `build`. When implementation is requested, preserve the consolidated report as the change scope and re-run the relevant verification afterward.

### 14. Build With the Same Owners

`build` mode runs the ownership table forward instead of backward. Review asks "what is wrong
with this"; build asks "what would make this right the first time". Same owners, same severity
scale, opposite direction.

#### 1. Recon, then name the domains in play

Principle 2's recon output is the input here, unchanged: framework, styling system, component
library, tokens, viewports, available checks. Then decide which domains the change actually
touches and say so before writing code.

Most changes touch three or four, not six. A form touches accessibility (labels, errors, focus
order) and writing (labels, error text), and layout only if it introduces structure. A
marketing section touches typography, layout, and color, and rarely accessibility beyond
headings and contrast. An icon button touches accessibility (accessible name, hit area) and
`better-ui` (stroke weight, states). Naming them is what makes step 4 checkable — an unnamed
domain is one you will not review.

#### 2. Consult the owners before implementing

Load each named owner and extract the constraints that bind *this* change **before** the first
line of code. Their principles are the spec, not a rubric to be graded against afterwards:
the type scale and its steps, the hit-area floor and which exception applies, the token names
for the roles you need, the copy voice and capitalisation policy, the reduced-motion rule.

Write those constraints down as the acceptance criteria for the change. Building first and
auditing second produces rework, and it biases the audit — you will not find a problem in code
you just wrote unless you decided in advance what "correct" meant.

#### 3. Implement inside the project's system

Use the existing styling system, component library, and tokens. Never introduce a second
styling approach, a second icon set, or a raw value that duplicates a mapped token.

Prefer extending a shared component over styling a leaf: a fix at the token or component level
applies everywhere the defect exists, which is the same reach-and-leverage rule Principle 9
uses for ranking findings.

This is the one mode that consults current documentation freely (Principle 6). New code should
use current APIs, and a pattern that was best practice three years ago often is not — but
check the feature against the project's stated browser targets before relying on it, and say
so when a choice depends on that.

#### 4. Self-review the diff

Review `git diff` against every domain named in step 1, using the same severity scale and the
same evidence standard. Scope is the diff, not the surrounding file: a pre-existing defect you
did not touch is out of scope, and reporting it as if you introduced it obscures what changed.

Fix anything you can fix immediately. Anything you deliberately leave becomes a
`Considered but Rejected` row with the reason — "the existing component has this defect
throughout and fixing it belongs in its own change" is a legitimate entry, and a far more
honest one than silence.

Self-review has a known weakness: you are checking your own work against criteria you chose.
The acceptance criteria from step 2 are the mitigation, because they were written before the
code existed. If a domain in play has no criteria from step 2, that is the gap to close first.

#### 5. Verify, then report

Run the checks per Principle 12 and report the exact commands and results. A build that
type-checks but was never rendered is `Not verified` on anything visual, and should say so.

Output the consolidated report scoped to the diff, with all named domains in the coverage
table and the same verdict ladder. `Approve` still requires complete coverage of the domains
you named — a build whose accessibility criteria were never checked is `Inconclusive`, not
`Approve`.

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
