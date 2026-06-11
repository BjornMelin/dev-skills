---
title: Use safeExtend() when extending refined objects or overwriting keys
impact: HIGH
tags: object, refine, safeExtend
---

# Use safeExtend() when extending refined objects or overwriting keys

## Why

On refined object schemas, overwriting keys with `.extend(...)` can throw. Use `.safeExtend(...)` to preserve refinements while ensuring assignability.

## Bad

```ts
import { z } from "zod";

const Base = z.object({ a: z.string() }).refine((v) => v.a.length > 0);

// Overwriting a key on a refined object can throw:
const X = Base.extend({ a: z.string().min(2) });
```

## Good

```ts
import { z } from "zod";

const Base = z.object({ a: z.string() }).refine((v) => v.a.length > 0);

const X = Base.safeExtend({ a: z.string().min(2) });
```
