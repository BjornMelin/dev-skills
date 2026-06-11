---
title: Use z.codec for bidirectional transforms across boundaries
impact: MEDIUM
tags: codec, encode, decode
---

# Use z.codec for bidirectional transforms across boundaries

## Why

Use codecs when you need a reversible mapping between a wire format and an internal representation (e.g. ISO string <-> Date).

## Bad

```ts
import { z } from "zod";

// Transform is decode-only and cannot be reversed.
const IsoToDate = z.iso.datetime().transform((s) => new Date(s));
```

## Good

```ts
import { z } from "zod";

const IsoToDate = z.codec(z.iso.datetime(), z.date(), {
  decode: (s) => new Date(s),
  encode: (d) => d.toISOString(),
});
```

## Notes

See: `references/codecs-v4.md`.
