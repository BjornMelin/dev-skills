---
title: z.email({ pattern }) requires a RegExp (not a string)
impact: HIGH
tags: schema, email
---

# z.email({ pattern }) requires a RegExp (not a string)

## Why

`z.email({ pattern })` expects a `RegExp`. Passing a string is a common migration mistake and can lead to runtime errors or ineffective validation.

## Bad

```ts
import { z } from "zod";

const Email = z.email({ pattern: ".*@example.com" });
```

## Good

```ts
import { z } from "zod";

const Email1 = z.email({ pattern: /.*@example\.com$/i });

// Prefer built-in regexes when available:
const Email2 = z.email({ pattern: z.regexes.email });
const Email3 = z.email({ pattern: z.regexes.html5Email });
const Email4 = z.email({ pattern: z.regexes.rfc5322Email });
const Email5 = z.email({ pattern: z.regexes.unicodeEmail });
```
