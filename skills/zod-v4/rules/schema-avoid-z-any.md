---
title: Prefer z.unknown() over z.any() unless you truly need any
impact: HIGH
tags: schema, types
---

# Prefer z.unknown() over z.any() unless you truly need any

## Why

`z.any()` infers `any` which disables type checking. `z.unknown()` forces you to narrow.

## Bad

```ts
import { z } from "zod";

const Payload = z.any();
type Payload = z.infer<typeof Payload>; // any
```

## Good

```ts
import { z } from "zod";

const Payload = z.unknown();
type Payload = z.infer<typeof Payload>; // unknown
```

