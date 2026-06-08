# native-controls-transitions Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Expo Router, Expo UI, react-native-screens, and React Navigation docs.
- Expo SDK compatibility docs.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-expo-router-stack.md`
- `references/docs-expo-ui.md`
- `references/docs-react-native-screens.md`
- `references/docs-react-navigation-native-stack.md`
- `references/docs-expo-animations.md`
- `references/docs-expo-controls.md`
- `references/docs-expo-ui-jetpack-compose.md`
- `references/docs-expo-ui-swift-ui.md`

## Tailored Reference Files

- `references/expo-router-and-screens-transitions.md` - Expo Router and screens transition guide
- `references/expo-ui-control-boundaries.md` - Expo UI native control boundaries
- `references/native-navigation-validation.md` - Navigation transition validation
- `references/platform-transition-option-map.md` - Platform transition option map
- `references/expo-ui-worklets-state.md` - Expo UI worklets and native state boundaries

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
