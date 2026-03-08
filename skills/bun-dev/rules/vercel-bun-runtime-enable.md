# vercel-bun-runtime-enable

## Why

Vercel Functions and Routing Middleware can run on the **Bun runtime** (Vercel’s Bun runtime is currently Beta per the referenced docs). Enabling it is configuration-driven and should be explicit to avoid accidental runtime changes.

## Do

- Enable Bun runtime by setting `bunVersion` in `vercel.json` (or `vercel.ts`).
- Prefer `bunVersion: "1.x"` unless you have a hard requirement to pin.
  - Validate current support/constraints in `references/ref-vercel-bun-runtime.md`.

## Don't

- Don’t assume that using Bun locally automatically means Vercel is running Bun in production.

## Examples

`vercel.json`:

```json
{
  "$schema": "https://openapi.vercel.sh/vercel.json",
  "bunVersion": "1.x"
}
```

`vercel.ts`:

```ts
import type { VercelConfig } from "@vercel/config/v1";

export const config: VercelConfig = {
  bunVersion: "1.x",
};
```
