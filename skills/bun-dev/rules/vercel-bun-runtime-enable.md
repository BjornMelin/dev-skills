# vercel-bun-runtime-enable

## Why

Vercel Functions can run on the Bun runtime (Beta). Enabling it is configuration-driven
and should be explicit to avoid accidental runtime changes. Local Bun does not imply
production Bun.

Live source of truth (Beta, changes often): <https://vercel.com/docs/functions/runtimes/bun>

## Do

- Enable the Bun runtime by setting `bunVersion` in `vercel.json` (or `vercel.ts`).
- Use `bunVersion: "1.x"` (currently the only supported value).
- For Next.js on the Bun runtime, run the framework through Bun in your scripts (see
  `vercel-nextjs-bun-runtime-scripts`).
- Keep Routing Middleware on Node: set `export const config = { runtime: 'nodejs' }` in
  `middleware.ts` (see `vercel-bun-runtime-limitations`).

## Don't

- Don't assume using Bun locally means Vercel runs Bun in production.
- Don't pin an exact Bun patch; `"1.x"` is the supported spelling.

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
