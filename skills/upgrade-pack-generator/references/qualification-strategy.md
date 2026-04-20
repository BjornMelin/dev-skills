# Qualification Strategy

The qualification stage is intentionally separate from bootstrap, enrichment,
and rendering.

## Why It Is Separate

- bootstrap and enrich stay focused on deterministic repo and family structure
- qualification can be rerun independently when docs, CLI behavior, or local
  overlays change
- render stays a pure document-generation step with an optional evidence input

## Qualification Inputs

- `upgrade-pack.yaml`
- the target repo root from `repo_context.repo_root`
- `qualification_plan.doc_urls`
- `qualification_plan.source_specs`
- `qualification_plan.cli_checks`

## Qualification Outputs

- `qualification-snapshot.json`
- rendered summary sections in the playbook and operator-mode files

## Command Rules

- qualification commands must be read-only
- prefer `--help`, graph inspection, package listing, config display, and doctor
  or diagnostics surfaces
- do not run install, deploy, publish, or code-changing commands in the
  qualification stage

## Overlay Rules

- repo-local skill overlays are optional accelerators, not required contracts
- the base family lane must stay correct without overlays
- overlays should only append routing or evidence hints; they must not replace
  the universal family logic
