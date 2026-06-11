---
title: Use parseAsync/safeParseAsync when schema has async refinements or transforms
impact: CRITICAL
tags: parse, async
---

# Use parseAsync/safeParseAsync when schema has async refinements or transforms

## Why

Async refinements/transforms require async parsing. If you call `.parse()` on a schema containing an async refinement, Zod will throw.

## Bad

```ts
import { z } from "zod";

const UserId = z.string().refine(async (id) => {
  return await db.userExists(id);
});

UserId.parse("abc"); // wrong
```

## Good

```ts
import { z } from "zod";

const UserId = z.string().refine(async (id) => {
  return await db.userExists(id);
});

await UserId.parseAsync("abc");
// or:
const result = await UserId.safeParseAsync("abc");
```
