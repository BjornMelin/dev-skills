# native-lottie Source Ledger

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

- `references/docs-lottie-react-native-readme.md`

## Tailored Reference Files

- `references/native-lottie-asset-lifecycle.md` - Native Lottie asset lifecycle
- `references/dotlottie-native-boundaries.md` - dotLottie native boundaries
- `references/accessibility-performance.md` - Native Lottie accessibility and performance
- `references/designer-handoff-and-feature-support.md` - Native Lottie designer handoff and feature support
- `references/native-playback-control-refs.md` - Native Lottie playback refs and lifecycle

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
