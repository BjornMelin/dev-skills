---
title: Prefer named import for Zod root; reserve namespace imports for Mini/Core work
impact: LOW
tags: migrate, imports
---

# Prefer named import for Zod root; reserve namespace imports for Mini/Core work

## Why

`import * as z from "zod"` is valid in Zod 4.4.3 and appears in official docs.
This skill prefers `import { z } from "zod"` for app code so examples and diffs
stay consistent. Namespace imports are the normal style for `zod/mini` and
`zod/v4/core`.

## Bad

```ts
import * as z from "zod";
```

## Good

```ts
import { z } from "zod";
```

## Notes

Namespace imports are correct for Zod Mini and Core:

```ts
import * as z from "zod/mini";
import type * as z4 from "zod/v4/core";
```

Treat this as style guidance, not a correctness failure.
