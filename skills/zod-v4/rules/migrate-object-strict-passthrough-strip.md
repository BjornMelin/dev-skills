---
title: Replace object().strict/passthrough/strip with strictObject/looseObject/default
impact: CRITICAL
tags: migrate, objects
---

# Replace object().strict/passthrough/strip with strictObject/looseObject/default

## Why

Zod v4 prefers explicit object constructors:

- `z.object(shape)` strips unknown keys by default
- `z.strictObject(shape)` rejects unknown keys
- `z.looseObject(shape)` preserves unknown keys

## Bad

```ts
import { z } from "zod";

const A = z.object({ a: z.string() }).strict();
const B = z.object({ a: z.string() }).passthrough();
const C = z.object({ a: z.string() }).strip();
```

## Good

```ts
import { z } from "zod";

const A = z.strictObject({ a: z.string() });
const B = z.looseObject({ a: z.string() });
const C = z.object({ a: z.string() }); // strips by default
```

