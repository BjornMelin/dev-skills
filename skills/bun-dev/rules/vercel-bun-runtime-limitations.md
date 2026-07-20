# vercel-bun-runtime-limitations

## Why

Vercel's Bun runtime (Beta) is optimized for serverless request/response execution. Some
Bun APIs and Vercel build features behave differently than on the Node runtime, so treat
it as a Function runtime, not a long-lived server.

Live source of truth (Beta, changes often): <https://vercel.com/docs/functions/runtimes/bun>

## Do

- Use the Vercel Function handler style (a default export with `fetch`) or a framework
  adapter designed for Vercel (see `vercel-bun-function-fetch-handler`).
- Treat the runtime as request/response oriented; assume no long-lived process.
- For Routing Middleware, keep `export const config = { runtime: 'nodejs' }` in
  `middleware.ts` - middleware does not run on the Bun runtime.
- If you need a persistent `Bun.serve()` server, deploy to a platform that runs Bun as a
  long-lived process instead.

## Don't

- Don't use `Bun.serve()` inside Vercel Functions.
- Don't assume Vercel build features that the Node runtime provides are identical on Bun
  (see matrix).

## Bun vs Node runtime on Vercel (distilled; verify against the live doc)

| Capability | Node runtime | Bun runtime (Beta) |
| --- | --- | --- |
| Node APIs / npm packages | Yes | Yes |
| Fluid compute, Active CPU pricing | Yes | Yes |
| Streaming responses, `waitUntil` | Yes | Yes |
| Runtime logs | Yes | Yes |
| Automatic source maps | Yes | Not yet |
| Bytecode caching | Yes | Not yet |
| `node:http` / `node:https` request metrics | Yes | Limited |
| `Bun.serve()` | n/a | Not supported in Functions |

## Examples

Bad (not supported in Vercel Functions):

```ts
Bun.serve({
  fetch() {
    return new Response("hi");
  },
});
```

Good (Vercel Function fetch handler):

```ts
export default {
  async fetch(req: Request) {
    return new Response("hi");
  },
};
```
