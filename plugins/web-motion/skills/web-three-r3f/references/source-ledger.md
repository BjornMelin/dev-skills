# web-three-r3f Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Three.js, React Three Fiber, and Drei documentation/source metadata.
- Expo/native GPU docs when the skill targets native runtime.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-drei-gltf-use-gltf.md`
- `references/docs-r3f-canvas.md`
- `references/docs-r3f-pitfalls.md`
- `references/docs-r3f-scaling-performance.md`
- `references/docs-three-disposal.md`
- `references/docs-r3f-introduction.md`
- `references/docs-three-creating-a-scene.md`
- `references/r3f-field-guide.md`

## Tailored Reference Files

- `references/r3f-scene-lifecycle.md` - Canvas/createRoot and scene lifecycle
- `references/asset-loaders-and-fallbacks.md` - GLTF/texture loaders and fallbacks
- `references/three-disposal-performance.md` - Three.js disposal and performance guide
- `references/interaction-and-event-boundaries.md` - R3F interaction and event boundaries
- `references/asset-pipeline-compression.md` - 3D asset pipeline, compression, and loader policy

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
