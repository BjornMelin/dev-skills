---
title: Know error precedence (schema > per-parse > global > locale)
impact: HIGH
tags: error, precedence
---

# Know error precedence (schema > per-parse > global > locale)

## Why

If multiple customizations are present, the highest-precedence message wins.

## Precedence (highest to lowest)

1. Schema-level error
2. Per-parse error map
3. Global error map (`z.config({ customError })`)
4. Locale error map (`z.config(en())` or `z.config(z.locales.en())`)

## Reference

See: `references/errors-v4.md`.

