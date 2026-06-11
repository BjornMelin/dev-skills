---
title: Validate at system boundaries (HTTP, DB, env, queues) and treat internal data as typed
impact: HIGH
tags: parse, boundaries
---

# Validate at system boundaries (HTTP, DB, env, queues) and treat internal data as typed

## Why

The highest leverage is validating once where untrusted data enters your system, then passing typed, validated values through internal code.

## Good default pattern

```ts
import { z } from "zod";

const Input = z.object({ id: z.string() });

export function handler(raw: unknown) {
  const parsed = Input.safeParse(raw);
  if (!parsed.success) return { ok: false as const };

  // parsed.data is now trusted (within this function boundary)
  return doWork(parsed.data.id);
}
```
