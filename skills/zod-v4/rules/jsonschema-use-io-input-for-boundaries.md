---
title: Use io: input when you need JSON Schema for boundary inputs (pipes/defaults/coercion)
impact: MEDIUM
tags: jsonschema, io
---

# Use io: input when you need JSON Schema for boundary inputs (pipes/defaults/coercion)

## Why

Some schemas have different input/output types (pipes, defaults, coerced primitives). `z.toJSONSchema` defaults to output; set `io: "input"` when you need to describe the raw boundary input.

## Good

```ts
import { z } from "zod";

const Limit = z.coerce.number().pipe(z.number().min(1));

// Describe boundary input type (string/unknown) instead of output (number).
z.toJSONSchema(Limit, { io: "input" });
```
