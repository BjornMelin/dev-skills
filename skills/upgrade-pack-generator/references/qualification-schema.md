# Qualification Schema

`qualification-snapshot.json` is the machine-readable evidence file produced by
`scripts/qualify_upgrade_pack.py`.

## Required Keys

- `schema_version`
- `generated_at`
- `family_slug`
- `anchor_package`
- `repo_root`
- `snapshot_filename`
- `qualification_status`
- `summary`
- `doc_checks`
- `source_checks`
- `cli_checks`
- `repo_local_skill_overlays`
- `caveats`

## Status Values

- `ready`
- `ready-with-caveats`
- `insufficient-evidence`

## Summary Shape

`summary` should contain integer counters for:

- `doc_checks`
- `doc_failures`
- `source_checks`
- `source_failures`
- `cli_checks`
- `cli_failures`
- `repo_local_overlays`

## Notes

- `doc_checks` records live official-doc metadata for the URLs declared in
  `qualification_plan.doc_urls`.
- `source_checks` records pinned `opensrc path` evidence for the specs declared
  in `qualification_plan.source_specs`.
- `cli_checks` records family-native read-only command results for the commands
  declared in `qualification_plan.cli_checks`.
- `repo_local_skill_overlays` records optional repo-local skill matches detected
  under `.agents/skills/`.
- Keep this file machine-readable. Human summaries belong in the rendered
  playbook/operator docs.
