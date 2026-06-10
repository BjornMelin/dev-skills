# Report Specification

## Output Files

- `dependency-upgrade-report.md`
- `dependency-upgrade-report.json`

## JSON Top-Level Keys

- `generated_at`
- `repo_root`
- `mode`
- `compatibility_policy`
- `repo_context`
- `summary`
- `targeted_dependencies`
- `deep_repo_map`
- `dependencies`
- `warnings`
- `command_traces`

## Dependency Object Keys

- `ecosystem`, `name`
- `current_version`, `latest_available`, `target_version`, `target_reason`
- `contexts` (manifest locations/types)
- `release_notes`, `changelog_text`
- `repo_usage` (`summary`, `files`, `hits`)
- `breaking_changes`, `deprecations`, `feature_adoptions`
- `refactor_actions`, `risk_level`, `confidence`
- `source_links`, `fallback_links`

## Markdown Sections

1. Executive Summary
2. Runtime Context
3. Upgrade Matrix
4. Required Refactors
5. Breaking Changes and Deprecations
6. New Features and Improvements
7. Repository Impact Map (when deep repo mapping is enabled)
8. Ordered Implementation Checklist
9. Source Links
