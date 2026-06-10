---
title: Use z.config({ customError }) for global error customization
impact: MEDIUM
tags: error, config
---

# Use z.config({ customError }) for global error customization

## Why

If you need consistent error messages across an app (or want to attach i18n behavior), use a global error map via `z.config({ customError })`.

Global customization has lower precedence than schema-level and per-parse error messages.

## Good

```ts
import { z } from "zod";

z.config({
  customError: (iss) => {
    if (iss.code === "invalid_type") return `Expected ${iss.expected}`;
    return undefined; // yield to locale/default
  },
});
```

## Notes

- Return `undefined` to yield to the next error map in the precedence chain.
- Do not throw from error map functions.
- See: `references/errors-v4.md`.

