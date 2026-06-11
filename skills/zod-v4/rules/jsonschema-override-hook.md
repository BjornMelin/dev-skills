---
title: Use override to adjust generated JSON Schema (and set unrepresentable: any when needed)
impact: MEDIUM
tags: jsonschema, override
---

# Use override to adjust generated JSON Schema (and set unrepresentable: any when needed)

## Why

Sometimes you need project-specific JSON Schema tweaks. Use `override` to mutate `ctx.jsonSchema` for a node.

## Good

```ts
import { z } from "zod";

const S = z.object({ name: z.string() });

z.toJSONSchema(S, {
  override: (ctx) => {
    // Mutate ctx.jsonSchema directly.
    if (ctx.jsonSchema && typeof ctx.jsonSchema === "object") {
      (ctx.jsonSchema as Record<string, unknown>).x_internal = true;
    }
  },
});
```

## Notes

Unrepresentable types throw before `override` runs. If you need to override an unrepresentable type, set:

```ts
z.toJSONSchema(z.date(), {
  unrepresentable: "any",
  override: (ctx) => {
    ctx.jsonSchema = { type: "string", format: "date-time" };
  },
});
```
