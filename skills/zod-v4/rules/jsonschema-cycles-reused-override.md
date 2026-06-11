---
title: Configure cycles/reused/override when exporting complex schemas to JSON Schema
impact: MEDIUM
tags: jsonschema
---

# Configure cycles/reused/override when exporting complex schemas to JSON Schema

## Why

Complex schemas often contain:

- cycles (recursive types)
- reused sub-schemas
- project-specific JSON Schema needs (override)

## Good defaults to consider

```ts
import { z } from "zod";

z.toJSONSchema(Schema, {
  cycles: "ref",
  reused: "ref",
});
```

## `override`

Use `override` to adjust the produced JSON Schema for a node.
The function should mutate `ctx.jsonSchema` directly.

If you need to override an unrepresentable type, set `unrepresentable: "any"` so override runs.

See: `references/json-schema-v4.md`.
