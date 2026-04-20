# Workflow

`upgrade-pack-generator` has two explicit modes.

## Mode 1: Family Authoring

Use this when a dependency family deserves reusable knowledge across many repos.

Steps:

1. Pick the canonical family slug and anchor package.
2. Create or update a YAML override in `references/family-overrides/`.
3. Keep the override family-specific:
   - package-native end state
   - related packages and aliases
   - special research lanes
   - verification proofs
   - naming conventions
4. Do not duplicate generic pack structure already provided by the base
   generator.

## Mode 2: Repo Instantiation

Use this when you want a repo-local pack for a concrete repository.

Steps:

1. Detect repo context with `scripts/detect_repo_context.py`.
2. Bootstrap a starter manifest with `scripts/bootstrap_manifest.py`.
3. Enrich the manifest with `scripts/enrich_manifest.py`.
4. During enrichment, lock the owner surface explicitly:
   - repo root
   - one owning workspace
   - repo root plus related workspaces
5. Refine the manifest further with live repo research and upstream package research.
6. Validate the manifest.
7. Run `scripts/qualify_upgrade_pack.py` to capture read-only docs, source, CLI,
   and repo-local overlay evidence in `qualification-snapshot.json`.
8. Render the pack into `.agents/plans/upgrade/<topic>/`.

## Guardrails

- Generation is docs/research only.
- Do not implement package upgrades while building the pack.
- The generated `upgrade-pack.yaml` is the canonical source.
- `qualification-snapshot.json` is the canonical machine-readable qualification
  evidence file for the rendered pack.
- `operator-mode.md` is a rendered derivative, not a hand-maintained document.
- Use one package-manager command family per target repo run.
- In monorepos, prefer workspace-owned family packs over root-centric guesses.
