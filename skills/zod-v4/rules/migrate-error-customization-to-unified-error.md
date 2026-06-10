---
title: Migrate v3 error customization (invalid_type_error/required_error/errorMap/message) to v4 unified error
impact: CRITICAL
tags: migrate, error
---

# Migrate v3 error customization (invalid_type_error/required_error/errorMap/message) to v4 unified error

## Why

Zod v4 standardizes error customization under a single `error` parameter.
Several v3-era patterns are removed or deprecated and should be migrated.

## Bad (v3 patterns)

```ts
import { z } from "zod";

z.string({ invalid_type_error: "Not a string", required_error: "Required" });
z.string({ errorMap: () => ({ message: "Bad" }) });
z.string().min(5, { message: "Too short" });
```

## Good (v4)

```ts
import { z } from "zod";

z.string({
  error: (iss) => (iss.input === undefined ? "Required" : "Not a string"),
});

z.string().min(5, { error: "Too short" });
```

## Notes

- Prefer schema-level `error` when possible. Per-parse `error` is lower precedence.
- Returning `undefined` yields to the next error map.
- See: `references/errors-v4.md`.

