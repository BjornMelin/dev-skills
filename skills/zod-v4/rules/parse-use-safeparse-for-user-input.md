---
title: Use safeParse() for user input and boundary data
impact: CRITICAL
tags: parse, boundaries
---

# Use safeParse() for user input and boundary data

## Why

`parse()` throws. For user input, HTTP requests, env vars, and other boundary data, use `safeParse()` and return structured errors.

## Bad

```ts
const data = Schema.parse(req.body); // throws -> 500 if uncaught
```

## Good

```ts
import { z } from "zod";

const result = Schema.safeParse(req.body);
if (!result.success) {
  return {
    ok: false as const,
    errors: z.flattenError(result.error).fieldErrors,
  };
}

const data = result.data;
```

