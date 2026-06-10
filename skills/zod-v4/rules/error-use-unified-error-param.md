---
title: Use the unified error parameter (v4) instead of legacy message/errorMap APIs
impact: HIGH
tags: error, messages
---

# Use the unified error parameter (v4) instead of legacy message/errorMap APIs

## Why

Zod v4 standardizes error customization under a single `error` parameter. This reduces footguns and makes precedence predictable.

## Good

```ts
import { z } from "zod";

const RequiredString = z.string({
  error: (iss) => (iss.input === undefined ? "Required" : "Invalid"),
});

const Password = z.string().min(8, { error: "Too short" });
```

## Notes

Return `undefined` from an `error` function to yield to the next error map in the precedence chain.
See: `references/errors-v4.md`.

