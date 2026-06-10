---
title: Prefer .meta() over .describe() in v4
impact: LOW
tags: meta
---

# Prefer .meta() over .describe() in v4

## Why

`.describe()` exists for compatibility with Zod v3, but `.meta()` is the recommended approach in v4.

## Good

```ts
import { z } from "zod";

const Email = z.email().meta({ description: "An email address" });
```

