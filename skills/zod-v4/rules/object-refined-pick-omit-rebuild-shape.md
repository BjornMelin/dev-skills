---
title: Do not call pick()/omit() on refined objects - rebuild from .shape
impact: HIGH
tags: object, refine, pick, omit
---

# Do not call pick()/omit() on refined objects - rebuild from .shape

## Why

In current Zod v4, calling `.pick()` or `.omit()` on a schema that contains refinements throws. Rebuild from the original unrefined shape instead.

## Bad

```ts
import { z } from "zod";

const Base = z.object({ a: z.string(), b: z.string() });
const Refined = Base.refine((v) => v.a === v.b);

const Picked = Refined.pick({ a: true }); // throws
```

## Good

```ts
import { z } from "zod";

const Base = z.object({ a: z.string(), b: z.string() });
const Refined = Base.refine((v) => v.a === v.b);

const Picked = z.object({ a: Base.shape.a });
```
