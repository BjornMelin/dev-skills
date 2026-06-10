---
title: Remember: z.object() strips unknown keys by default
impact: HIGH
tags: object, unknown-keys
---

# Remember: z.object() strips unknown keys by default

## Why

In Zod v4, `z.object(shape)` strips unrecognized keys from the output. This affects API behavior if you expected passthrough.

## Example

```ts
import { z } from "zod";

const Dog = z.object({ name: z.string() });

Dog.parse({ name: "Yeller", extraKey: true });
// => { name: "Yeller" }
```

## What To Do Instead

- Reject unknown keys: `z.strictObject(shape)`
- Preserve unknown keys: `z.looseObject(shape)`
