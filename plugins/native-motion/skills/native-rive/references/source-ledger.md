# native-rive Source Ledger

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

- `references/docs-rive-react-native.md`
- `references/docs-rive-state-machine.md`

## Tailored Reference Files

- `references/rive-native-state-machines.md` - Native Rive state-machine contract
- `references/rive-native-asset-loading.md` - Rive native asset loading and lifecycle
- `references/nitro-platform-validation.md` - Nitro/platform validation notes
- `references/rive-file-caching-and-assets.md` - Native Rive file caching and asset loading
- `references/state-machine-input-protocol.md` - Native Rive state-machine input protocol

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
