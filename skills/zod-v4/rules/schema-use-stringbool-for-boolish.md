---
title: Use z.stringbool() for parsing boolish strings (env vars, query params)
impact: HIGH
tags: schema, env
---

# Use z.stringbool() for parsing boolish strings (env vars, query params)

## Why

`z.coerce.boolean()` uses `Boolean(value)` which treats most non-empty strings as true. For env vars and query params, prefer `z.stringbool()`.

## Bad

```ts
import { z } from "zod";

z.coerce.boolean().parse("false"); // true
```

## Good

```ts
import { z } from "zod";

z.stringbool().parse("false"); // false
z.stringbool().parse("1"); // true
```
