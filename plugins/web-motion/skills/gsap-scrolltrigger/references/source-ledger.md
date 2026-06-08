# gsap-scrolltrigger Source Ledger

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

- `references/docs-scrolltrigger-current-notes.md`

## Tailored Reference Files

- `references/official-source.md` - Official GreenSock ScrollTrigger skill source
- `references/scene-geometry.md` - Trigger geometry, pin, scrub, and refresh rules
- `references/scroll-validation.md` - Scroll scene validation checklist
- `references/smooth-scroll-and-scroller-proxy.md` - Smooth-scroll and scroller proxy boundary
- `references/responsive-refresh-playbook.md` - Responsive refresh and invalidation playbook

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
