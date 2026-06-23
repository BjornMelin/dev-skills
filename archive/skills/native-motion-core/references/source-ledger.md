# native-motion-core Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Expo SDK docs, React Native Reanimated, React Native Worklets, and Software Mansion source notes.
- React Native accessibility and performance documentation.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-expo-reanimated.md`
- `references/docs-reanimated-4-migration-testing.md`
- `references/docs-reanimated-compatibility.md`
- `references/docs-reanimated-getting-started.md`
- `references/docs-reanimated-performance.md`
- `references/docs-expo-animation.md`
- `references/docs-reanimated-accessibility.md`
- `references/docs-rn-animated-source-notes.md`
- `references/software-mansion-animations-animation-functions.md`
- `references/software-mansion-animations-animations-performance.md`
- `references/software-mansion-animations-animations.md`
- `references/software-mansion-animations-canvas-animations.md`
- `references/software-mansion-animations-canvas-atlas.md`
- `references/software-mansion-animations-gpu-animations.md`
- `references/software-mansion-animations-index.md`
- `references/software-mansion-animations-layout-animations.md`
- `references/software-mansion-animations-scroll-and-events.md`
- `references/software-mansion-animations-svg-animations.md`

## Tailored Reference Files

- `references/reanimated-worklets-core.md` - Reanimated 4 and Worklets core workflow
- `references/expo-sdk-compatibility.md` - Expo SDK compatibility matrix
- `references/layout-scroll-gesture-patterns.md` - Layout, scroll, and gesture patterns
- `references/threading-runonjs-scheduleonrn.md` - Reanimated threading and RN callback boundaries
- `references/gesture-handler-integration.md` - Gesture Handler and Reanimated integration

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
