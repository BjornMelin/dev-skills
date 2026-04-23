# Web Research Contract

`upgrade-pack-generator` cannot call `web.run` from Python. The browser
confirmation step is therefore an explicit Codex workflow stage.

## Inputs

- `research-bundle.json`
  - use `web_research_queue`
- `upgrade-pack.yaml`
  - use `research_plan.required_web_confirmation_categories`

## Required Behavior

1. Read `web_research_queue` from `research-bundle.json`.
2. For every queue item where `required_for_complete=true`, use `web.run` to
   confirm the official page.
3. Prefer the seeded official page first. Only broaden the search when the
   seeded page is broken or clearly non-authoritative.
4. Capture concise facts that materially affect the upgrade plan:
   - latest compatible API refs
   - migration constraints
   - newly recommended capabilities
   - deprecations or removals
5. Write those confirmations to `web-research-findings.json`.
6. Re-run `scripts/research_upgrade_pack.py` so `research-snapshot.json`
   reflects the confirmed pages before qualification.

## `web-research-findings.json` Shape

```json
{
  "entries": [
    {
      "category": "official_docs",
      "url": "https://example.dev/docs",
      "confirmed": true,
      "confirmed_at": "2026-04-20T00:00:00Z",
      "facts": [
        "Short machine-readable fact 1.",
        "Short machine-readable fact 2."
      ]
    }
  ]
}
```

## Completion Rule

If a category listed in `required_web_confirmation_categories` does not have a
confirmed entry in `web-research-findings.json`, `research_status` must not be
`complete`.
