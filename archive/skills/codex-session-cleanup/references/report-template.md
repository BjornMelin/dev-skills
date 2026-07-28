# Report Template

Every scan writes JSON and Markdown.

## JSON Manifest Fields

- `manifest_id`
- `generated_at`
- `codex_home`
- `scope`
- `scope_roots`
- `target_root`
- `target_label`
- `artifact_families`
- `rule_profile`
- `include_medium`
- `include_parent_overlap`
- `min_age_hours`
- `candidate_threads`
- `selected_threads`
- `medium_not_selected`
- `excluded_threads`
- `session_files`
- `history`
- `session_index`
- `state_db`
- `logs_db`
- `memory`
- `artifacts`
- `automation`
- `quarantine_hint`

## Markdown Sections

- Summary counts.
- Scope roots.
- Selected high-confidence sessions.
- Medium-confidence candidates not selected.
- Protected/excluded sessions.
- Memory triage.
- Artifact-family report-only candidates.
- Automation policy and blockers.
- Copy-first apply command.
- Restore command shape.

## Closeout Summary

After apply, report:

- quarantine directory;
- session files quarantined and removed;
- history/index lines removed;
- state/log rows removed;
- memory summaries moved/copied;
- memory registry snapshot path;
- SQLite integrity results;
- checksum status;
- backup inventory;
- remaining selected rows/files, if any;
- automation blockers or residual manual-review risk.
