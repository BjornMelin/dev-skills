# UI Audit Schema

`ui_audit.v1` is the shared JSON contract for UI audit evidence emitted or
described by UI-focused skills. It gives future agents, task capsules, and the
TUI one shape for framework-specific audit results without requiring a browser
dashboard or hosted service.

## Producers

Current producers and adapters:

- `skills/streamlit-master-architect/scripts/audit_streamlit_project.py` emits
  `ui_audit.v1` with `--format ui-audit-json`.
- `skills/dash-audit/scripts/dash_ui_audit_adapter.py` converts
  `ui-audit-preflight dash-callback-map` output into `ui_audit.v1`.
- `dash-audit`, `dmc-best-practices`, and `browser-workbench-setup` describe
  manual audit output using this schema when they do not have a native scanner.

## Top-Level Contract

Every payload uses this top-level shape:

```json
{
  "schema": "ui_audit.v1",
  "producer": {
    "skill": "streamlit-master-architect",
    "tool": "audit_streamlit_project.py",
    "version": "2026-05-12"
  },
  "target": {
    "framework": "streamlit",
    "root": "<scan-root>"
  },
  "summary": {
    "status": "pass",
    "counts": {
      "error": 0,
      "warning": 0,
      "info": 0
    },
    "total_findings": 0
  },
  "findings": [],
  "observations": [],
  "metadata": {}
}
```

Required fields:

- `schema`: always `ui_audit.v1`.
- `producer`: skill, tool or workflow, and version/date for the emitting rule
  set.
- `target.framework`: framework or audit surface such as `dash`, `dmc`,
  `streamlit`, or `browser-workbench`.
- `target.root`: `<scan-root>` unless the producer explicitly documents an
  opt-in to expose absolute paths.
- `summary.status`: `fail` when any `error` finding exists, `warning` when any
  `warning` finding exists, otherwise `pass`.
- `summary.counts`: finding counts by severity.
- `findings`: actionable problems or risks.
- `observations`: non-actionable facts that explain coverage, inputs, or audit
  inventory.

## Finding Shape

Findings are review prompts, not automatic edit instructions:

```json
{
  "id": "streamlit.deprecated_api",
  "severity": "error",
  "category": "migration",
  "title": "Deprecated Streamlit API",
  "detail": "st.cache: Deprecated caching API; migrate to st.cache_data / st.cache_resource.",
  "locations": [
    {
      "path": "app.py",
      "line": 12
    }
  ],
  "recommendation": "Migrate deprecated Streamlit APIs to stable equivalents.",
  "docs": [
    "https://docs.streamlit.io/develop/api-reference/caching-and-state"
  ]
}
```

Field conventions:

- `id`: stable, namespaced identifier: `<surface>.<rule>`.
- `severity`: one of `error`, `warning`, or `info`.
- `category`: one of `accessibility`, `layout`, `state`, `performance`,
  `security`, `migration`, `interaction`, `visual`, `testing`, or `workflow`.
- `title`: short human-readable label.
- `detail`: concise explanation suitable for a PR review note.
- `locations`: zero or more path objects. Paths are relative to the target root
  when possible.
- `recommendation`: focused next action when one is known.
- `docs`: authoritative references to verify before changing public behavior.

## Severity Mapping

Use this mapping across framework-specific audits:

- `error`: security-sensitive risk, likely broken production behavior, removed
  API, or issue that blocks a safe release.
- `warning`: likely bug, migration hazard, accessibility regression, or
  performance risk that should be verified and fixed when valid.
- `info`: useful context, coverage note, or lower-risk improvement prompt.

Framework-native terms map into this scale:

- Streamlit `high` -> `error`, `medium` -> `warning`, `low` -> `info`.
- DMC `CRITICAL` or `HIGH` -> `error`, `MEDIUM-HIGH` or `MEDIUM` ->
  `warning`, `LOW` -> `info`.
- Browser workflow setup gaps that block repeatable verification -> `warning`
  or `error` depending on whether a release gate is blocked.

## Privacy

UI audit payloads may contain repository structure and review findings. Default
producers should redact absolute roots as `<scan-root>` and avoid source
snippets. Do not paste full payloads into external search, Firecrawl, Exa, issue
trackers, or PR comments unless the repository owner has approved that
processing. Share only the specific redacted findings needed for review.
