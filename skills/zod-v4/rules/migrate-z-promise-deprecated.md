---
title: Avoid z.promise() (deprecated) - await then parse
impact: HIGH
tags: migrate, promises
---

# Avoid z.promise() (deprecated) - await then parse

## Why

In Zod v4, `z.promise(...)` is deprecated. If you suspect a value might be a promise, await it and validate the resolved value.

## Bad

```ts
import { z } from "zod";

const P = z.promise(z.number());
```

## Good

```ts
import { z } from "zod";

const Num = z.number();

async function parseMaybePromise(value: unknown) {
  const resolved = await value;
  return Num.parse(resolved);
}
```

