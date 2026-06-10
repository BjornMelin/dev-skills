---
title: Replace error.format/error.flatten/z.formatError with z.treeifyError/z.flattenError
impact: CRITICAL
tags: migrate, errors
---

# Replace error.format/error.flatten/z.formatError with z.treeifyError/z.flattenError

## Why

Zod v4 moved error formatting into top-level utilities:

- `z.treeifyError(error)` for nested structures
- `z.flattenError(error)` for flat form errors
- `z.prettifyError(error)` for a human-readable string

## Bad

```ts
if (!result.success) {
  const formatted = result.error.format();
  const flattened = result.error.flatten();
}
```

```ts
const formatted = z.formatError(error);
```

## Good

```ts
import { z } from "zod";

if (!result.success) {
  const tree = z.treeifyError(result.error);
  const flat = z.flattenError(result.error);
  const pretty = z.prettifyError(result.error);
}
```
