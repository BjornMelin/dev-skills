# Research Schema

`research-snapshot.json` is the machine-readable evidence file produced by
`scripts/research_upgrade_pack.py`.

## Required Keys

- `schema_version`
- `generated_at`
- `family_slug`
- `anchor_package`
- `repo_root`
- `snapshot_filename`
- `research_status`
- `current_version`
- `target_version`
- `target_version_policy`
- `compatibility_rationale`
- `release_range`
- `required_categories`
- `category_status`
- `summary`
- `official_docs`
- `api_reference`
- `migration_guides`
- `release_history`
- `examples_cookbooks`
- `source_evidence`
- `repo_usage_mapping`
- `caveats`

## Status Values

- `complete`
- `partial`
- `insufficient-evidence`

## Required Categories

Every research plan must model these categories:

- `official_docs`
- `api_reference`
- `migration_guides`
- `release_history`
- `examples_cookbooks`
- `source_evidence`
- `repo_usage_mapping`

## Notes

- Keep this file machine-readable. Human summaries belong in the rendered
  playbook and operator docs.
- `category_status` should use `ok`, `partial`, `failed`, or `missing`.
- `release_range` should explain the current-to-target range that was actually
  reviewed.
- `source_evidence` should include pinned `opensrc path` resolution plus any
  high-signal changelog, migration, or example files found in the fetched
  source tree.
- `repo_usage_mapping` should capture read-only grep or inventory commands that
  prove how the target repo currently uses the package family.
