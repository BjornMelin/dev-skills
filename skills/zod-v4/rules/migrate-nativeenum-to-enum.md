---
title: Replace z.nativeEnum() with z.enum()
impact: CRITICAL
tags: migrate, enums
---

# Replace z.nativeEnum() with z.enum()

## Why

In Zod v4, `z.enum(...)` replaces v3's `z.nativeEnum(...)`.

## Bad

```ts
import { z } from "zod";

enum Role {
  Admin = "admin",
  User = "user",
}

const RoleSchema = z.nativeEnum(Role);
```

## Good

```ts
import { z } from "zod";

enum Role {
  Admin = "admin",
  User = "user",
}

const RoleSchema = z.enum(Role);
```

## Notes

For best TypeScript inference, prefer `as const` objects over TS `enum` where feasible.
