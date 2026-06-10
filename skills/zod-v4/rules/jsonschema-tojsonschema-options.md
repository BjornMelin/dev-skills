---
title: Use z.toJSONSchema with explicit params (io, target, metadata) when needed
impact: MEDIUM
tags: jsonschema
---

# Use z.toJSONSchema with explicit params (io, target, metadata) when needed

## Why

Defaults are fine for many cases, but you often need to pin:

- `target` (draft vs OpenAPI 3.0)
- `io` (input vs output for transforms/pipes/defaults/coercion)
- `metadata` registry (to extract `$defs` and include meta)

## Example

```ts
import { z } from "zod";

const S = z.coerce.number().pipe(z.number().min(1));

z.toJSONSchema(S, { io: "input" });
z.toJSONSchema(S, { target: "openapi-3.0" });
```

See: `references/json-schema-v4.md`.

