# web-rive Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Rive web/native runtime package docs and source metadata.
- Rive state machine documentation.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-rive-react.md`
- `references/docs-rive-state-machine.md`
- `references/docs-rive-web-js.md`
- `references/implementation-notes.md`

## Tailored Reference Files

- `references/rive-react-runtime.md` - Rive React/runtime lifecycle
- `references/state-machine-inputs.md` - State machine input contract guide
- `references/asset-security-and-fallbacks.md` - Rive asset security, accessibility, and fallback review
- `references/layout-fit-and-resize.md` - Rive layout, fit, resize, and DPR behavior
- `references/data-binding-and-events.md` - Rive data binding and event contracts

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
