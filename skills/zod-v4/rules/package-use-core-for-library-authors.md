---
title: Use zod/v4/core for schema tooling and library adapters
impact: HIGH
tags: package, core, library-authors
---

# Use zod/v4/core for schema tooling and library adapters

## Why

Application code should usually import from `zod`, and bundle-sensitive app code
can use `zod/mini`. Libraries that accept user-provided Zod schemas or need to
introspect schema internals should target `zod/v4/core` so they work with both
Zod Classic and Zod Mini.

## Bad

```ts
import { z } from "zod";

export function acceptsSchema(schema: z.ZodType) {
  return schema;
}
```

## Good

```ts
import type * as z4 from "zod/v4/core";

export function acceptsSchema(schema: z4.$ZodType) {
  return schema;
}
```

## Notes

- If a generic consumer-facing tool only needs parse/infer behavior, consider
  Standard Schema before building a Zod-specific adapter.
- When switching on first-party schema/check types, include a non-throwing
  default branch so new Zod schema types do not break the library.
- See `references/package-surfaces-v4.md`.
