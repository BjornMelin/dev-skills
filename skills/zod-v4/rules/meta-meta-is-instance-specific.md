---
title: Metadata is instance-specific; most Zod methods clone and drop prior metadata
impact: MEDIUM
tags: meta, gotcha
---

# Metadata is instance-specific; most Zod methods clone and drop prior metadata

## Why

Most Zod methods are immutable and return a new schema instance. Metadata is attached to an instance, so it does not automatically propagate across clones.

## Example

```ts
import { z } from "zod";

const A = z.string().meta({ description: "A cool string" });
const B = A.refine(() => true);

A.meta(); // { description: "A cool string" }
B.meta(); // undefined
```

## Fix

Re-apply metadata to the derived schema:

```ts
const B2 = B.meta({ description: "A cool string" });
```
