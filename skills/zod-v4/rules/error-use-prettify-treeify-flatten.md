---
title: Use prettifyError/treeifyError/flattenError for error presentation
impact: HIGH
tags: error, formatting
---

# Use prettifyError/treeifyError/flattenError for error presentation

## Why

Zod v4 provides dedicated utilities for common error presentation shapes.

## Good

```ts
import { z } from "zod";

if (!result.success) {
  const pretty = z.prettifyError(result.error); // logs/CLI
  const tree = z.treeifyError(result.error); // nested traversal
  const flat = z.flattenError(result.error); // forms
}
```

See: `references/error-formatting-v4.md`.
