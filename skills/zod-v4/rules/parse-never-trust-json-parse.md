---
title: Never trust JSON.parse output - validate it
impact: HIGH
tags: parse, json
---

# Never trust JSON.parse output - validate it

## Why

`JSON.parse` returns `any`. Always validate after parsing, especially for external inputs and persisted blobs.

## Bad

```ts
const obj = JSON.parse(raw); // any
doWork(obj.user.id);
```

## Good

```ts
import { z } from "zod";

const Payload = z.object({
  user: z.object({ id: z.string() }),
});

const parsed = Payload.safeParse(JSON.parse(raw));
if (!parsed.success) throw new Error("Invalid payload");

doWork(parsed.data.user.id);
```
