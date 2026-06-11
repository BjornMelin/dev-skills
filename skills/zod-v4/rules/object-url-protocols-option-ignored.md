---
title: Do not use z.url({ protocols: […] }); use regex protocol/hostname options or z.httpUrl
impact: HIGH
tags: urls, footgun
---

# Do not use z.url({ protocols: [...] }); use regex protocol/hostname options or z.httpUrl

## Why

Some examples online use `z.url({ protocols: [...] })`. In v4, the option to
restrict protocol is `protocol` (singular) and it expects a `RegExp`. The
`hostname` option also expects a `RegExp`.

## Bad

```ts
import { z } from "zod";

const WebUrl = z.url({ protocols: ["http", "https"] });
const AlsoBad = z.url({ protocol: "https" });
```

## Good

```ts
import { z } from "zod";

const AnyUrl = z.url();

const WebUrl = z.url({
  protocol: /^https?$/,
  hostname: z.regexes.domain,
});

const HttpUrl = z.httpUrl(); // convenience for http/https URLs
```

## Notes

Use `normalize: true` only when callers should receive normalized URL strings.
