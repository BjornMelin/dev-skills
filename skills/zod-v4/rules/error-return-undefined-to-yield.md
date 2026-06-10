---
title: In error maps, return undefined to yield to the next error map
impact: MEDIUM
tags: error, precedence
---

# In error maps, return undefined to yield to the next error map

## Why

Zod uses a precedence chain for error maps. Returning `undefined` yields to the next error map (per-parse, global, locale, default).

## Good

```ts
import { z } from "zod";

const S = z.number({
  error: (iss) => {
    if (iss.code === "too_big") return `Too big: max is ${iss.maximum}`;
    return undefined; // yield to next error map
  },
});
```

## Notes

See: `rules/error-error-precedence.md` and `references/errors-v4.md`.
