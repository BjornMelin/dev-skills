# Ecosystem: tRPC

## Input Validation

```ts
import { z } from "zod";

export const Input = z.object({ id: z.string() });

// publicProcedure.input(Input).query(({ input }) => ...)
```

## Output Validation

If you validate outputs, define an output schema and keep it close to the procedure:

```ts
import { z } from "zod";

export const UserOutput = z.object({
  id: z.string(),
  email: z.email(),
});

// publicProcedure.output(UserOutput).query(() => ...)
```

## Notes

- Avoid parsing the same payload twice: validate once at the boundary, then treat data as typed internally.
