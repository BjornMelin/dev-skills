---
title: Avoid intersection() for object merging - prefer extend or object spread
impact: MEDIUM
tags: object, intersection
---

# Avoid intersection() for object merging - prefer extend or object spread

## Why

`z.intersection(A, B)` returns a `ZodIntersection` which lacks common object helpers like `pick` and `omit`. Prefer creating a new object schema.

## Bad

```ts
import { z } from "zod";

const Person = z.object({ name: z.string() });
const Employee = z.object({ role: z.string() });

const Employed = z.intersection(Person, Employee);
```

## Good

```ts
import { z } from "zod";

const Person = z.object({ name: z.string() });
const Employee = z.object({ role: z.string() });

const Employed1 = Person.extend(Employee.shape);
const Employed2 = z.object({ ...Person.shape, ...Employee.shape });
```

