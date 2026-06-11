---
title: Remember: z.record(z.enum(…), value) is exhaustive in v4 (use partialRecord for optional keys)
impact: HIGH
tags: schema, records, enums
---

# Remember: z.record(z.enum(...), value) is exhaustive in v4 (use partialRecord for optional keys)

## Why

In Zod v4, if your record key schema is an enum or literal union, Zod will require all keys to be present (matches TypeScript's `Record` behavior).

## Example

```ts
import { z } from "zod";

const Keys = z.enum(["id", "name", "email"]);
const Person = z.record(Keys, z.string());

Person.parse({ id: "1", name: "A" }); // throws (missing email)
```

## Fix

Use `z.partialRecord` when keys are optional:

```ts
import { z } from "zod";

const Keys = z.enum(["id", "name", "email"]);
const Person = z.partialRecord(Keys, z.string());
```
