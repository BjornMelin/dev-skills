---
title: Use declaration merging to extend GlobalMeta typing
impact: LOW
tags: meta, typescript
---

# Use declaration merging to extend GlobalMeta typing

## Why

`z.globalRegistry` supports additional metadata keys, and you can type them safely via declaration merging.

## Good

```ts
declare module "zod" {
  interface GlobalMeta {
    examples?: unknown[];
  }
}

export {};
```

## Notes

See: `references/metadata-registries-v4.md`.
