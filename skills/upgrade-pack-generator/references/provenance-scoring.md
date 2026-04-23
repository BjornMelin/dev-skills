# Provenance Scoring

`research_upgrade_pack.py` now emits per-category provenance in
`category_provenance`.

## Factors

- `officiality`
  - how authoritative the source looks
- `freshness`
  - how recently the bundled source-map entry was verified
- `directness`
  - whether the category has explicit `web.run` confirmation
- `package_specificity`
  - whether the source is package-specific instead of generic ecosystem prose

## Score

The current weighted score is:

- officiality: `35%`
- freshness: `20%`
- directness: `25%`
- package specificity: `20%`

This score is a trust signal, not a final gate by itself. Readiness still
depends on:

- required category coverage
- identity confidence
- compatible target resolution
- required `web.run` confirmations
