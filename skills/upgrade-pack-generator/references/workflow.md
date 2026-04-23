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
7. Run `scripts/research_upgrade_pack.py` to capture upstream docs, API refs,
   release notes, cookbooks, source evidence, repo-usage mapping, package
   identity, and target-version reasoning in `research-snapshot.json` plus the
   raw `research-bundle.json`.
8. Use Codex `web.run` against the required `web_research_queue` items from
   `research-bundle.json` and write the confirmations to
   `web-research-findings.json`.
9. Re-run `scripts/research_upgrade_pack.py` so `research-snapshot.json`
   includes the official-page confirmations and blocks `complete` when they are
   still missing.
10. Run `scripts/qualify_upgrade_pack.py` to capture read-only docs, source, CLI,
   and repo-local overlay evidence in `qualification-snapshot.json`.
11. Render the pack into `.agents/plans/upgrade/<topic>/`.

The rendered pack should follow this final contract:

- the playbook is the human source of truth
- the playbook is the only file updated during a real implementation run
- operator-mode is a delta card with links back to playbook sections
- trigger-prompt is a thin launcher that tells a fresh Codex session to load
  the playbook first
- `upgrade-pack.yaml`, `research-snapshot.json`, and
  `qualification-snapshot.json` remain the canonical machine-readable sources
- `research-bundle.json` is the raw evidence ledger behind the research
  snapshot
- `web-research-findings.json` is the machine-readable `web.run` evidence file
  for the required official docs and API-reference surfaces

## Guardrails

- Generation is docs/research only.
- Do not implement package upgrades while building the pack.
- The generated `upgrade-pack.yaml` is the canonical source.
- `research-snapshot.json` is the canonical machine-readable research evidence
  file for the rendered pack.
- `research-bundle.json` is the canonical raw evidence bundle for the rendered
  pack.
- `qualification-snapshot.json` is the canonical machine-readable qualification
  evidence file for the rendered pack.
- `operator-mode.md` and `trigger-prompt.md` are rendered derivatives, not
  hand-maintained documents.
- Use one package-manager command family per target repo run.
- In monorepos, prefer workspace-owned family packs over root-centric guesses.
