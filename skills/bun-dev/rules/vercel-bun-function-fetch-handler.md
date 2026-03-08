# vercel-bun-function-fetch-handler

## Why

Vercel’s Bun runtime uses a `fetch`-style handler for Functions. This keeps code aligned with Web-standard `Request`/`Response` APIs.

## Do

- Export a default object with an async `fetch(request: Request)` method.
- Use `Response.json(...)` for JSON responses.

## Don't

- Don’t build a long-lived HTTP server inside a Function.

## Examples

`api/hello.ts`:

```ts
export default {
  async fetch(request: Request) {
    const url = new URL(request.url);
    const name = url.searchParams.get("name") || "World";
    return Response.json({ message: `Hello ${name}!` });
  },
};
```

