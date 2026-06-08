# typegpu Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- TypeGPU documentation and MIT package source.
- MDN/WebGPU browser runtime documentation.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/matrices.md`
- `references/pipelines.md`
- `references/setup.md`
- `references/shaders.md`
- `references/textures.md`
- `references/types.md`
- `references/advanced.md`
- `references/noise.md`
- `references/sdf.md`

## Tailored Reference Files

- `references/typegpu-codex-playbook.md` - Codex workflow for TypeGPU tasks
- `references/shader-resource-boundaries.md` - Shader/resource ownership rules
- `references/webgpu-runtime-validation.md` - Browser WebGPU validation and fallbacks
- `references/browser-capability-and-adapter-selection.md` - Browser capability and adapter selection
- `references/compute-vs-render-pipeline-design.md` - Compute versus render pipeline design

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
