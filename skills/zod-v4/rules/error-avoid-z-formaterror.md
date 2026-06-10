---
title: Avoid z.formatError() (deprecated) - use z.treeifyError()
impact: MEDIUM
tags: error, deprecated
---

# Avoid z.formatError() (deprecated) - use z.treeifyError()

## Why

`z.formatError()` is deprecated in favor of `z.treeifyError()`.

## Bad

```ts
import { z } from "zod";
const formatted = z.formatError(err);
```

## Good

```ts
import { z } from "zod";
const tree = z.treeifyError(err);
```

