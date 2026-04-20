# Research Schema

`research-snapshot.json` is the machine-readable evidence file produced by
`scripts/research_upgrade_pack.py`.

The same stage also writes `research-bundle.json`, a raw evidence bundle with
identity resolution, registry metadata, release history traces, Context7 hits,
source evidence, repo-usage outputs, bundled source-map drift checks, and the
machine-readable `web.run` queue.

## Required Keys

- `schema_version`
- `generated_at`
- `family_slug`
- `anchor_package`
- `repo_root`
- `snapshot_filename`
- `bundle_filename`
- `web_findings_filename`
- `research_status`
- `current_version`
- `target_version`
- `target_version_policy`
- `compatibility_rationale`
- `release_range`
- `required_categories`
- `category_status`
- `category_provenance`
- `summary`
- `identity`
- `source_map_seed`
- `target_resolution`
- `recommended_related_packages`
- `official_docs`
- `api_reference`
- `migration_guides`
- `release_history`
- `examples_cookbooks`
- `source_evidence`
- `repo_usage_mapping`
- `web_research_queue`
- `web_research_findings`
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
- `identity.status` should be `high-confidence`, `medium-confidence`, or
  `low-confidence`.
- `target_resolution.selected_status` should distinguish `compatible`,
  `compatible-with-caveats`, and `incompatible`.
- `category_status` should use `ok`, `partial`, `failed`, or `missing`.
- `release_range` should explain the current-to-target range that was actually
  reviewed.
- `web_research_queue` should contain the required and optional `web.run`
  pages that still need official confirmation.
- `web_research_findings` should summarize any confirmed official pages already
  written to `web-research-findings.json`.
- `source_evidence` should include pinned `opensrc path` resolution plus any
  high-signal changelog, migration, or example files found in the fetched
  source tree.
- `repo_usage_mapping` should capture read-only grep or inventory commands that
  prove how the target repo currently uses the package family.
- `recommended_related_packages` should contain evidence-backed peer or
  companion packages that likely belong in the same upgrade wave.
