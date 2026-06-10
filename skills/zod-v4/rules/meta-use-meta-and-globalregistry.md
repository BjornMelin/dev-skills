---
title: Use .meta() (or register(globalRegistry)) to attach JSON Schema metadata
impact: MEDIUM
tags: meta, jsonschema
---

# Use .meta() (or register(globalRegistry)) to attach JSON Schema metadata

## Why

Metadata in Zod v4 lives in registries and flows into `z.toJSONSchema(...)`.

## Good

```ts
import { z } from "zod";

const Email = z.email().meta({
  id: "email_address",
  title: "Email address",
  description: "Your email address",
  examples: ["first.last@example.com"],
});
```

## Notes

`.register(z.globalRegistry, meta)` also works, but `.meta()` is the most direct.
See: `references/metadata-registries-v4.md`.

