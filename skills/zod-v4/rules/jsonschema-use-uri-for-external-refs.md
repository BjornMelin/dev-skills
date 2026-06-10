---
title: Use uri: (id) => ... to generate external $ref URIs for catalogs
impact: LOW
tags: jsonschema, refs
---

# Use uri: (id) => ... to generate external $ref URIs for catalogs

## Why

When exporting a registry/catalog, you may want external, fully-qualified `$ref` URIs instead of relative IDs.

## Good

```ts
import { z } from "zod";

z.toJSONSchema(z.globalRegistry, {
  uri: (id) => `https://example.com/${id}.json`,
});
```
