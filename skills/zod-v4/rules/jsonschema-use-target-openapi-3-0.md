---
title: Set target to openapi-3.0 when generating OpenAPI schema objects
impact: MEDIUM
tags: jsonschema, openapi
---

# Set target to openapi-3.0 when generating OpenAPI schema objects

## Why

OpenAPI schema objects are not identical to JSON Schema drafts. When exporting for OpenAPI, set `target: "openapi-3.0"` explicitly.

## Good

```ts
import { z } from "zod";

const schema = z.object({ name: z.string() });
const openapiSchema = z.toJSONSchema(schema, { target: "openapi-3.0" });
```
