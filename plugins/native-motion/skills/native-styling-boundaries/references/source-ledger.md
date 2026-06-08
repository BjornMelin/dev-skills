# native-styling-boundaries Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- NativeWind and react-native-css package docs/source metadata.
- Expo Tailwind setup documentation.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-expo-tailwind-setup.md`
- `references/docs-nativewind-installation.md`

## Tailored Reference Files

- `references/nativewind-v4-boundaries.md` - NativeWind setup and version boundaries
- `references/react-native-css-tailwind.md` - React Native CSS and Tailwind compatibility
- `references/class-safety-and-tokens.md` - Native class safety and token policy
- `references/nativewind-metro-babel-pipeline.md` - NativeWind Metro, Babel, and CSS pipeline
- `references/cross-platform-style-differences.md` - Cross-platform style and animation differences

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
