---
title: .register() returns the same schema instance (unlike .meta/.describe)
impact: MEDIUM
tags: meta, registry
---

# .register() returns the same schema instance (unlike .meta/.describe)

## Why

`.register(registry, meta)` is special: it does not clone, it returns the original schema instance. This matters when you chain methods and expect metadata to move with the clone.

## Example

```ts
import { z } from "zod";

const r = z.registry<{ description: string }>();
const S = z.string();

const same = S.register(r, { description: "x" });
same === S; // true
```

## Notes

`.meta(...)` and `.describe(...)` return new schema instances.

See: `references/metadata-registries-v4.md`.

