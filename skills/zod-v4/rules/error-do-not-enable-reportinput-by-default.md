---
title: Do not enable reportInput by default (sensitive data risk)
impact: HIGH
tags: error, security
---

# Do not enable reportInput by default (sensitive data risk)

## Why

Including raw inputs in issues can leak secrets/PII into logs and error payloads. Zod keeps it off by default.

## Good

Only enable it for specific, reviewed cases:

```ts
Schema.parse(value, { reportInput: true });
```

