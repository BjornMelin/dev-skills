# native-skia Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- React Native Skia package docs/source metadata and Expo SDK docs.
- Software Mansion animation/canvas skill notes where license-gated.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-react-native-skia-api-notes.md`
- `references/docs-react-native-skia-installation.md`
- `references/docs-software-mansion-canvas-animations.md`
- `references/docs-software-mansion-canvas-atlas.md`

## Tailored Reference Files

- `references/skia-canvas-patterns.md` - Skia canvas animation patterns
- `references/skia-performance-lifecycle.md` - Skia performance and lifecycle
- `references/skia-web-expo-boundaries.md` - Skia web and Expo boundaries
- `references/skia-reanimated-interoperability.md` - Skia and Reanimated interoperability
- `references/image-font-shader-resource-cache.md` - Skia image, font, shader, and resource cache policy

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
