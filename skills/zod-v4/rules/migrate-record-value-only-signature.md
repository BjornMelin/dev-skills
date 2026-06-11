---
title: Prefer explicit z.record(keySchema, valueSchema)
impact: MEDIUM
tags: migrate, records
---

# Prefer explicit z.record(keySchema, valueSchema)

## Why

Zod 4.4.3 still supports one-argument `z.record(valueSchema)` as compatibility
behavior, but explicit key schemas are clearer and make enum-key exhaustiveness
visible during migrations.

## Bad

```ts
import { z } from "zod";

const Cache = z.record(z.string());
```

## Good

```ts
import { z } from "zod";

const Cache = z.record(z.string(), z.string());
```

## Notes

If you use an enum as the key schema, v4 checks exhaustiveness. If you want
optional enum keys, use `z.partialRecord(...)`. Treat one-argument records as
advisory migration findings, not invalid Zod 4.4.3 code.
