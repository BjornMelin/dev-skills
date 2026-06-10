# Ecosystem: Next.js Server Actions

Validate at the server boundary, return a form-friendly error shape.

```ts
"use server";

import { z } from "zod";

const Login = z.object({
  email: z.email(),
  password: z.string().min(8),
});

export async function loginAction(formData: FormData) {
  const raw = Object.fromEntries(formData);
  const result = Login.safeParse(raw);

  if (!result.success) {
    return { ok: false as const, errors: z.flattenError(result.error).fieldErrors };
  }

  // result.data is typed and validated
  return { ok: true as const };
}
```
