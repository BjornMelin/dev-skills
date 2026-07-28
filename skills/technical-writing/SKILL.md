---
name: technical-writing
description: "Internal engineering docs: specs, ADRs, architecture, runbooks, migrations, rollout plans, and maintainer guides."
allowed-tools: Read Write Edit Glob Grep
license: MIT
metadata:
  tags: technical-writing, documentation, specs, architecture, adr, runbooks, migration, developer-docs, docs-as-code
  platforms: Claude, ChatGPT, Gemini
  version: "2.1.0"
  modernization: 2026-04-13
  hardening: 2026-04-17
---

# Technical Writing

Use this skill when the deliverable is **internal technical documentation for builders and operators**.

`technical-writing` is the documentation-cluster anchor for:
- technical specs
- product requirements documents (PRDs)
- architecture docs
- ADRs / decision records
- runbooks and incident procedures
- rollout / rollback / migration guides
- developer-facing implementation or maintenance guides

## When to use this skill
- A team needs a technical spec before implementation starts
- A team needs a PRD for product/feature requirements before design or implementation
- An engineer needs an architecture document or ADR that records trade-offs and decisions
- Ops needs a runbook, rollback guide, or incident response procedure
- A migration or rollout needs a durable written path with validation and rollback notes
- A developer-facing internal guide needs to explain how a system works and how to work on it safely

## When not to use this skill

The lane is *internal* documentation. Say so and hand back when the real job is:

- **Published API, SDK, OpenAPI or developer-portal content** — an external
  reference surface with its own versioning and consumers.
- **End-user onboarding, tutorials, FAQs or help-centre flows** — the audience
  is a customer, not a builder or operator.
- **Release notes, `CHANGELOG.md` or customer-facing migration announcements.**
- **Slides, decks or roadmap presentations.**
- **Product positioning, launch copy or GTM messaging.**
- **Deciding the feature or API itself**, which has to happen before the
  document describing it can be written.

These boundaries are stated as *work*, not as skill names, so they stay true
whichever documentation skills happen to be installed. Name a specific skill
only if one is actually available in the current session.

## Reference map

Load on demand; do not read all of these up front.

| File | Use it for |
| --- | --- |
| `references/document-modes-and-boundaries.md` | Choosing the primary mode and deciding what is out of scope |
| `references/mode-structures.md` | The smallest fitting section layout for the chosen mode |
| `references/prd.md` | PRD-specific structure: personas, stories, acceptance criteria, metrics |
| `references/quality-checklists.md` | The Step 7 quality check in full |
| `references/docs-as-code-and-maintenance.md` | Repo-friendly conventions and long-term doc maintenance |

## Instructions

### Step 1: Classify one primary mode
Normalize the request into one primary mode before drafting.

```yaml
technical_writing_mode:
  primary_mode: prd | spec | architecture | adr | runbook | migration | internal-guide
  audience: engineers | operators | mixed | unknown
  source_of_truth: repo | incident-notes | existing-doc | mixed | unknown
  lifecycle_state: draft | review | rewrite | maintenance
  docs_surface: markdown-repo | docs-site | wiki | unknown
  review_need: decision-signoff | operational-accuracy | handoff-clarity | unknown
```

Use one primary mode per run:
- `prd` → product requirement, personas, stories, acceptance criteria, success metrics, risks
- `spec` → planned change, goals, constraints, design, rollout, rollback, open questions
- `architecture` → system structure, boundaries, interfaces, trade-offs, failure modes
- `adr` → one material decision with options and rationale
- `runbook` → operate, diagnose, recover, escalate
- `migration` → move from old to new safely with validation and rollback
- `internal-guide` → implementation-facing explanation for maintainers

### Step 2: Confirm audience and route-outs
Answer three questions before writing:
1. Who will act on this document?
2. What decision or action should it enable?
3. Which neighboring skills must stay out of scope?

Quick route-out table. Hand the work back with this reason; pick a named skill
from the current session only if one genuinely covers it.

| If the request sounds like... | It is not this lane because... |
|---|---|
| Publish docs for an API, SDK, webhook, or developer portal | The audience is external and the surface is versioned separately |
| Write a tutorial, onboarding guide, or FAQ | The audience is an end user, not a builder or operator |
| Summarize shipped changes or maintain `CHANGELOG.md` | It reports what shipped rather than enabling a decision or action |
| Make slides for a launch, roadmap, or architecture review | The artifact is a presentation, not a reviewable document |
| Write launch or product messaging | It is positioning, not internal technical record |
| Decide the API or feature design before writing docs | The decision has to exist before it can be documented |

### Step 3: Gather the minimum technical evidence
Do not draft from vibes alone. Pull the smallest credible evidence set first:
- current behavior or architecture notes
- interfaces, schemas, commands, or operational signals
- rollout or operational constraints
- known failure modes and recovery steps
- unresolved questions or trade-offs

If evidence is missing, label assumptions explicitly instead of pretending the document is authoritative.

### Step 4: Choose the smallest fitting structure
Use the mode rules below and only keep the sections that fit the chosen document.

### Step 5: Apply mode-specific writing rules
- **Specs** must separate goals from non-goals.
- **Architecture docs** must explain boundaries and trade-offs, not every code path.
- **ADRs** must capture one decision, not become a full design doc.
- **Runbooks** must optimize for fast action under pressure.
- **Migration guides** must foreground compatibility, validation, and rollback.
- **Internal guides** must explain implementation reality, not customer education or marketing value props.

### Step 6: Keep it docs-as-code friendly
Default to reviewable, repo-friendly writing:
- stable headings
- concise bullet lists where operators scan
- explicit commands, paths, owners, and prerequisites
- dated decisions and status for ADR-like docs
- links to source-of-truth docs instead of duplicated narrative when possible

### Step 7: Run the quality check
Before finalizing, verify:
1. The audience is named or obvious.
2. The document states what decision or action it enables.
3. Assumptions and unknowns are labeled.
4. Commands, interfaces, validation, rollback, or escalation are concrete where relevant.
5. Neighboring documentation skills are not being absorbed.
6. The title and section layout match the chosen mode.

### Step 8: Return a brief or the finished artifact
Preferred summary shape before full drafting:

```markdown
# Technical Writing Brief

## Mode
- Primary mode:
- Why it fits:
- Audience:

## Source material used
- Repo/docs/evidence:
- Assumptions / gaps:

## Draft structure
1. section
2. section
3. section

## Writing notes
- Key decisions / actions enabled:
- Risks / unknowns:
- Route-outs kept out of scope:
```

If the user already asked for the finished artifact, produce the chosen document directly with the matching structure above.

## Examples

### Example 1: Internal design doc before implementation
**Input**
> Write a technical spec for moving our worker queue from Redis lists to Redis streams. Engineers need goals, constraints, rollout, and rollback before coding.

**Good output direction**
- mode: `spec`
- audience: engineers
- include goals, non-goals, constraints, design, rollout, rollback, open questions
- keep API portal publishing out of scope

### Example 2: Architecture decision capture
**Input**
> We chose Postgres logical replication over dual writes. Record the decision and alternatives in an ADR.

**Good output direction**
- mode: `adr`
- capture context, decision, alternatives, consequences, follow-up
- keep the document short and decision-focused

### Example 3: Incident runbook
**Input**
> Write a runbook for when the payments worker backlog spikes and retries start timing out.

**Good output direction**
- mode: `runbook`
- include symptoms, immediate checks, operating steps, escalation, rollback / recovery
- optimize for operator speed, not essay-style explanation

### Example 4: Boundary with API docs
**Input**
> Refresh our public webhook quickstart and auth troubleshooting page for external developers.

**Good output direction**
- decline the request as outside the internal-documentation lane
- explain that the main job is published developer-facing API docs, not internal technical documentation

## Best practices
1. Choose the document mode before writing the body.
2. Keep internal technical docs decision- and action-oriented.
3. Write only the sections the mode needs; do not force every template into every document.
4. Separate internal design / ops docs from API portals, user help, release notes, decks, and GTM copy.
5. Prefer docs-as-code structure: reviewable Markdown, stable headings, and source-linked facts.
6. Label assumptions and unresolved questions explicitly.
7. For runbooks and migrations, make rollback and escalation easy to find.
8. When the request changes audience, route out instead of stretching the internal-docs lane.

## References
- [Diátaxis](https://diataxis.fr/)
- [Write the Docs — Docs as Code](https://www.writethedocs.org/guide/docs-as-code/)
- [Write the Docs — How to write software documentation](https://www.writethedocs.org/guide/writing/beginners-guide-to-docs/)
- [Architectural Decision Records](https://adr.github.io/)
- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
