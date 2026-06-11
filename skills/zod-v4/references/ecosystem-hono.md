# Ecosystem: Hono

Use `@hono/zod-validator` to validate request bodies, query params, and path params.

```ts
import { Hono } from "hono";
import { zValidator } from "@hono/zod-validator";
import { z } from "zod";

const app = new Hono();

const CreateUser = z.object({
  name: z.string().min(1),
  email: z.email(),
});

app.post("/users", zValidator("json", CreateUser), (c) => {
  const data = c.req.valid("json");
  return c.json({ ok: true, data });
});
```

For structured validation errors:

```ts
app.post(
  "/users",
  zValidator("json", CreateUser, (result, c) => {
    if (!result.success) {
      return c.json({ errors: z.flattenError(result.error) }, 400);
    }
  }),
  (c) => c.text("ok"),
);
```
