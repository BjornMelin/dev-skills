---
title: Coerce first, then validate via pipe() (avoid validating raw inputs)
impact: HIGH
tags: parse, coerce, pipe
---

# Coerce first, then validate via pipe() (avoid validating raw inputs)

## Why

Coercion changes the type. If you attach constraints directly to the coerce schema, you can get surprising behavior depending on the raw input. Prefer `coerce().pipe(...)` so constraints run on the coerced value.

## Bad

```ts
import { z } from "zod";

const Limit = z.coerce.number().min(1);
```

## Good

```ts
import { z } from "zod";

const Limit = z.coerce.number().pipe(z.number().min(1));
```
