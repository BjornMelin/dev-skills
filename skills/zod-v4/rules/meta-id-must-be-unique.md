---
title: Metadata registries treat id as unique (duplicates throw)
impact: MEDIUM
tags: meta, registry
---

# Metadata registries treat id as unique (duplicates throw)

## Why

Zod registries treat the `id` property specially: adding multiple schemas with the same `id` throws an error. This applies to all registries, including `z.globalRegistry`.

## Good

Pick stable, unique IDs:

```ts
import { z } from "zod";

const Email = z.email().meta({ id: "email_address" });
const User = z.object({ email: Email }).meta({ id: "user" });
```

## Notes

See: `references/metadata-registries-v4.md`.

