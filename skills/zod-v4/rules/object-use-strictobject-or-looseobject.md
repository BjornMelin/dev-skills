---
title: Pick strictObject vs looseObject explicitly (do not rely on implicit behavior)
impact: HIGH
tags: object, unknown-keys
---

# Pick strictObject vs looseObject explicitly (do not rely on implicit behavior)

## Why

Security and product semantics often depend on unknown-key handling. Make it explicit and visible in code review.

## Good

```ts
import { z } from "zod";

const Strict = z.strictObject({ a: z.string() });
const Loose = z.looseObject({ a: z.string() });
```
