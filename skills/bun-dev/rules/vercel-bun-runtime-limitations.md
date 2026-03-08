# vercel-bun-runtime-limitations

## Why

Vercel’s Bun runtime is optimized for serverless execution. Some Bun APIs (notably `Bun.serve`) are not supported in Vercel Functions.

## Do

- Use the Vercel Function handler style (`fetch`) or a framework adapter designed for Vercel.
- Treat the runtime as request/response oriented; avoid assumptions of long-lived servers.
  - If you need a long-lived server, deploy to a platform that supports `Bun.serve()` as a persistent process.

## Don't

- Don’t use `Bun.serve()` inside Vercel Functions.

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
