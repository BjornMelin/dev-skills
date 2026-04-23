# Research Bundle Schema

`research-bundle.json` is the raw evidence artifact produced by
`scripts/research_upgrade_pack.py`.

## Required Keys

- `schema_version`
- `generated_at`
- `family_slug`
- `anchor_package`
- `repo_root`
- `snapshot_filename`
- `bundle_filename`
- `web_findings_filename`
- `identity`
- `source_map_seed`
- `target_resolution`
- `registry`
- `github`
- `ctx7`
- `collectors`
- `repo_runtime`
- `discovered_sources`
- `source_evidence`
- `repo_usage_mapping`
- `category_entries`
- `category_status`
- `category_provenance`
- `web_research_queue`
- `web_research_findings`
- `caveats`

## Notes

- Keep this file machine-readable. It is the raw evidence ledger, not a human
  narrative.
- `identity` should include the canonical package identity, resolved docs and
  repository surfaces, confidence, evidence, conflicts, and unresolved
  surfaces.
- `target_resolution` should include the selected target version, compatibility
  status, candidate versions reviewed, peer and engine checks, and any
  evidence-backed related packages.
- `registry` should summarize the npm metadata call used to seed generic
  discovery.
- `github` should summarize resolved repository metadata and recent release
  tags when available.
- `ctx7` should capture library resolution plus any category-level docs queries
  used for API-reference, migration, or example discovery.
- `source_map_seed` should capture the bundled source-map entry, freshness, and
  drift findings used to seed generic package research.
- `collectors` should summarize optional adapters or local tools detected at
  runtime.
- `category_entries` should mirror the category material that rolled up into
  `research-snapshot.json`.
- `web_research_queue` should be the machine-readable queue Codex uses with
  `web.run` for official docs and API references.
