---
title: Treat z.fromJSONSchema as experimental (pin versions and test hard)
impact: MEDIUM
tags: jsonschema
---

# Treat z.fromJSONSchema as experimental (pin versions and test hard)

## Why

Zod v4 documents `z.fromJSONSchema()` as experimental and not part of the stable API contract.

## Guidance

- Avoid using it in core product paths unless you can pin Zod versions.
- Add tests that validate behavior against representative schemas.

