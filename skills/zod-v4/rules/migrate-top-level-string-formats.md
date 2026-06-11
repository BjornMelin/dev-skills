---
title: Use top-level string formats (z.email, z.url, z.uuidv4, z.iso.*)
impact: CRITICAL
tags: migrate, strings, formats
---

# Use top-level string formats (z.email, z.url, z.uuidv4, z.iso.*)

## Why

Zod v4 provides top-level format schemas. Older `z.string().email()` style
format methods still exist but are deprecated.

## Bad

```ts
import { z } from "zod";

const Email = z.string().email();
const Url = z.string().url();
const Uuid = z.string().uuid();
```

## Good

```ts
import { z } from "zod";

const Email = z.email();
const Url = z.url();
const HttpUrl = z.httpUrl();
const Uuid = z.uuid();
const UuidV7 = z.uuidv7();
const Host = z.hostname();
const Phone = z.e164();
const Token = z.jwt();

const IsoDate = z.iso.date();
const IsoDatetime = z.iso.datetime();
```

## Notes

Current top-level helpers include `z.email`, `z.url`, `z.httpUrl`,
`z.hostname`, `z.e164`, `z.uuid`, `z.uuidv4`, `z.uuidv6`, `z.uuidv7`,
`z.guid`, `z.ipv4`, `z.ipv6`, `z.cidrv4`, `z.cidrv6`, `z.hex`, `z.hash`,
`z.jwt`, `z.mac`, `z.base64`, `z.base64url`, `z.nanoid`, `z.cuid`, `z.cuid2`,
`z.ulid`, `z.iso.*`, and `z.stringFormat`.
