# web-lottie Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- lottie-web, lottie-react-native, dotLottie web/native package docs and source metadata.
- Expo SDK docs for native package compatibility when applicable.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-dotlottie-web.md`
- `references/docs-lottie-web-load-animation-options.md`
- `references/docs-lottie-web-readme.md`
- `references/implementation-notes.md`

## Tailored Reference Files

- `references/lottie-player-lifecycle.md` - lottie-web player lifecycle
- `references/dotlottie-web-component.md` - dotLottie web component and worker notes
- `references/asset-accessibility-security.md` - Asset accessibility and security review
- `references/authoring-and-export-compatibility.md` - Authoring and export compatibility
- `references/runtime-event-contracts.md` - Runtime events, markers, and playback contracts

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
