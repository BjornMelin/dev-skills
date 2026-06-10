---
title: Handle unrepresentable Zod types when generating JSON Schema
impact: MEDIUM
tags: jsonschema, footgun
---

# Handle unrepresentable Zod types when generating JSON Schema

## Why

Some Zod types cannot be represented in JSON Schema (e.g. `z.date()`, `z.bigint()`, `z.undefined()`, `z.map()`, `z.set()`, `z.transform()`, `z.custom()`).

By default, `z.toJSONSchema` throws when it hits these.

## Options

- Prefer removing/avoiding unrepresentable types in schemas you need to export as JSON Schema.
- If you need best-effort output, set `unrepresentable: "any"` (unrepresentable nodes become `{}`).

```ts
import { z } from "zod";

z.toJSONSchema(z.date(), { unrepresentable: "any" }); // {}
```
