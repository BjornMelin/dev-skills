---
title: Prefer named import for Zod root in app code
impact: LOW
tags: migrate, imports
---

# Prefer named import for Zod root in app code

## Why

Zod 4.4.3 exports a default `z`, so default imports are package-valid. This
skill still prefers named root imports for consistency with most app examples
and cleaner migration diffs.

## Bad

```ts
import z from "zod";
```

## Good

```ts
import { z } from "zod";
```

## Notes

Treat this as style guidance, not a correctness failure.
