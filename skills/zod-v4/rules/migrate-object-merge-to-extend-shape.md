---
title: Replace object.merge() with extend(other.shape) or object spread
impact: HIGH
tags: migrate, objects, composition
---

# Replace object.merge() with extend(other.shape) or object spread

## Why

Zod v4 prefers building objects from shape or using `.extend(...)`.

## Bad

```ts
import { z } from "zod";

const A = z.object({ a: z.string() });
const B = z.object({ b: z.number() });

const AB = A.merge(B);
```

## Good

```ts
import { z } from "zod";

const A = z.object({ a: z.string() });
const B = z.object({ b: z.number() });

const AB1 = A.extend(B.shape);

const AB2 = z.object({
  ...A.shape,
  ...B.shape,
});
```

## Notes

If you need a different unknown-key strategy, pick the constructor explicitly:

```ts
const StrictAB = z.strictObject({ ...A.shape, ...B.shape });
```
