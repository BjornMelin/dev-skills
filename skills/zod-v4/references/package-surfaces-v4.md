# Package Surfaces, Zod Mini, and Library Authors

## Contents

- [Application Imports](#application-imports)
- [Package Exports](#package-exports)
- [Zod Mini](#zod-mini)
- [Zod Core and Library Authors](#zod-core-and-library-authors)
- [Versioning Guidance](#versioning-guidance)

## Application Imports

For application code on Zod 4.4.3, use the root package by default:

```ts
import { z } from "zod";
```

The root `zod` export is Zod 4 in the 4.x line. Namespace and default imports
also work in the package, but this skill prefers named `z` imports for
consistency and smaller diffs.

Use Zod Mini only when bundle size is a concrete constraint:

```ts
import * as z from "zod/mini";
```

## Package Exports

Zod 4.4.3 exports the package root plus these important subpaths:

- `zod/mini`
- `zod/locales`
- `zod/v3`
- `zod/v4`
- `zod/v4-mini`
- `zod/v4/mini`
- `zod/v4/core`
- `zod/v4/locales`
- `zod/v4/locales/*`
- `zod/package.json`

Use `zod/v3` only for a real pinned-v3 boundary. Do not create a mixed v3/v4
compat layer in app code unless an external dependency requires it.

## Zod Mini

Zod Mini trades method-heavy ergonomics for a smaller surface. Use it when a
bundle budget makes the tradeoff worthwhile. For ordinary server code, internal
tools, and most app code, prefer regular `zod`.

When writing examples for Mini, use namespace imports because Mini's functional
surface is designed around that style:

```ts
import * as z from "zod/mini";
```

## Zod Core and Library Authors

Libraries that accept or introspect user-provided Zod schemas should use
`zod/v4/core`, not `zod`, `zod/v4`, or `zod/mini`. Core contains the shared
base classes and internals for both Classic and Mini.

```ts
import type * as z4 from "zod/v4/core";

export function inspect(schema: z4.$ZodType) {
  const def = schema._zod.def;
  switch (def.type) {
    case "object":
      return "object";
    default:
      return "unknown";
  }
}
```

If the integration only needs a shared validation/inference contract, prefer
Standard Schema over Zod-specific internals.

## Versioning Guidance

For app code, depend on the current major line normally:

```json
{ "dependencies": { "zod": "^4.4.3" } }
```

For libraries that support Zod Core, keep peer ranges broad enough for users to
bring their own compatible Zod 4 core package and test against the latest
available version.
