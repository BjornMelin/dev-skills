---
title: Prefer decode/encode for typed boundaries; use safeParse for untyped user input
impact: LOW
tags: codec, parse
---

# Prefer decode/encode for typed boundaries; use safeParse for untyped user input

## Why

- `.parse(...)` is great at untyped boundaries but throws.
- `.safeParse(...)` is the default at untyped boundaries where you want structured errors.
- `.decode(...)` / `.encode(...)` are useful when the boundary input is already strongly typed in TS (especially with codecs).

## Good

```ts
import { z } from "zod";

const IsoDatetimeToDate = z.codec(z.iso.datetime(), z.date(), {
  decode: (s) => new Date(s),
  encode: (d) => d.toISOString(),
});

// Untyped boundary (unknown): safeParse
const r = IsoDatetimeToDate.safeParse("2024-01-01T00:00:00.000Z");
if (r.success) r.data;

// Typed boundary: decode/encode
IsoDatetimeToDate.decode("2024-01-01T00:00:00.000Z");
IsoDatetimeToDate.encode(new Date());
```

