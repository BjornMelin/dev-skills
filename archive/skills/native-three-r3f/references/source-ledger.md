# native-three-r3f Source Ledger

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

- `references/docs-expo-webgpu-three.md`
- `references/docs-r3f-native-api-notes.md`
- `references/docs-r3f-react-native-installation.md`
- `references/docs-three-creating-a-scene.md`
- `references/docs-r3f-introduction.md`
- `references/docs-software-mansion-gpu-animations.md`

## Tailored Reference Files

- `references/r3f-native-installation.md` - R3F native installation and lifecycle
- `references/expo-webgpu-three-boundary.md` - Expo GL/WebGPU/Three boundary notes
- `references/native-gpu-validation.md` - Native GPU validation checklist
- `references/expo-gl-webgpu-decision-tree.md` - Expo GL, WebGPU, and Three decision tree
- `references/native-asset-loader-recipes.md` - Native 3D asset loader recipes

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
