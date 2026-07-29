# gsap-react Source Ledger

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

- `references/official-source.md` - Official GreenSock React skill source
- `references/react-lifecycle.md` - useGSAP, refs, context, and cleanup
- `references/next-client-boundary.md` - Next.js and SSR/client boundary notes
- `references/contextsafe-event-handlers.md` - contextSafe event handlers and callbacks
- `references/strict-mode-and-route-transitions.md` - React Strict Mode, dependency, and route transition checks

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
