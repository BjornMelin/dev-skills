# gsap-performance Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- GreenSock official GSAP AI skills, MIT: https://github.com/greensock/gsap-skills
- GSAP documentation and package license source: https://gsap.com/docs/v3/ and https://gsap.com/standard-license/
- Agent Skills specification and best practices: https://agentskills.io/specification

## Bundled Source Excerpts

- No additional source excerpt files are required for this skill beyond the official baseline and tailored references.

## Tailored Reference Files

- `references/official-source.md` - Official GreenSock performance skill source
- `references/property-cost-matrix.md` - Property cost and rendering matrix
- `references/scroll-performance.md` - ScrollTrigger and scroll workload review
- `references/profiling-playbook.md` - GSAP profiling and evidence playbook
- `references/layer-budget-and-will-change.md` - Layer budget and will-change discipline

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
