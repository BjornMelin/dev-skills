---
title: Use .meta({ id, title, description, examples }) to enrich JSON Schema output and enable $defs
impact: MEDIUM
tags: jsonschema, meta
---

# Use .meta({ id, title, description, examples }) to enrich JSON Schema output and enable $defs

## Why

Metadata from registries is copied into JSON Schema output. IDs can also affect `$defs` extraction behavior when using registries.

## Good

```ts
import { z } from "zod";

const User = z
  .object({
    email: z.email().meta({ description: "User email" }),
  })
  .meta({ id: "user", title: "User" });

z.toJSONSchema(User);
```

