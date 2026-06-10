---
title: Use locales for i18n (z.config(en()), dynamic imports, avoid z.locales for bundling)
impact: MEDIUM
tags: error, i18n
---

# Use locales for i18n (z.config(en()), dynamic imports, avoid z.locales for bundling)

## Why

Zod provides many locale error maps. Configure them with `z.config(...)`.

## Good

```ts
import { z } from "zod";
import { en } from "zod/locales";

z.config(en());
```

Lazy-load locales to reduce bundle impact:

```ts
import { z } from "zod";

export async function loadLocale(locale: string) {
  const { default: localeFn } = await import(`zod/v4/locales/${locale}.js`);
  z.config(localeFn());
}
```

## Notes

- `z.locales.*` exists, but may not be tree-shakable in some bundlers. Prefer importing the locale from `zod/locales` when bundling matters.
- See: `references/errors-v4.md`.

