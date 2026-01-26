# Zod v4 Ecosystem Integration Patterns

## React Hook Form

### Basic Integration

```typescript
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

const schema = z.object({
  name: z.string().min(1, { error: 'Required' }),
  email: z.email({ error: 'Invalid email' }),
  age: z.number().min(18),
});

type FormData = z.infer<typeof schema>;

function MyForm() {
  const { register, handleSubmit, formState: { errors } } = useForm<FormData>({
    resolver: zodResolver(schema),
  });

  return (
    <form onSubmit={handleSubmit(data => console.log(data))}>
      <input {...register('name')} />
      {errors.name && <span>{errors.name.message}</span>}

      <input {...register('email')} />
      {errors.email && <span>{errors.email.message}</span>}

      <input type="number" {...register('age', { valueAsNumber: true })} />
      {errors.age && <span>{errors.age.message}</span>}

      <button type="submit">Submit</button>
    </form>
  );
}
```

### Best Practices

- Keep schemas in dedicated `src/schemas/` folder for reuse
- Use `valueAsNumber` for number inputs
- Share schemas between client and server validation

### Compatibility Note

If a dependency is still pinned to Zod v3, use an explicit v3 import:
```typescript
import * as z from 'zod/v3';
```
Prefer `import { z } from 'zod'` for v4 unless you must stay on v3.

---

## tRPC

### Router Definition

```typescript
import { z } from 'zod';
import { router, publicProcedure } from './trpc';

const userSchema = z.object({
  name: z.string().min(1),
  email: z.email(),
});

export const appRouter = router({
  createUser: publicProcedure
    .input(userSchema)
    .mutation(({ input }) => {
      // input is fully typed as { name: string; email: string }
      return createUser(input);
    }),

  getUser: publicProcedure
    .input(z.object({ id: z.string() }))
    .query(({ input }) => {
      return getUserById(input.id);
    }),

  listUsers: publicProcedure
    .input(z.object({
      limit: z.number().min(1).max(100).default(10),
      cursor: z.string().optional(),
    }))
    .query(({ input }) => {
      return listUsers(input.limit, input.cursor);
    }),
});
```

### Output Validation

```typescript
const userOutput = z.object({
  id: z.string(),
  name: z.string(),
  createdAt: z.date(),
});

export const appRouter = router({
  getUser: publicProcedure
    .input(z.object({ id: z.string() }))
    .output(userOutput)
    .query(({ input }) => {
      return getUserById(input.id);
    }),
});
```

---

## Hono

### Validation Middleware

```typescript
import { Hono } from 'hono';
import { zValidator } from '@hono/zod-validator';
import { z } from 'zod';

const app = new Hono();

const createUserSchema = z.object({
  name: z.string().min(1),
  email: z.email(),
});

app.post(
  '/users',
  zValidator('json', createUserSchema),
  (c) => {
    const data = c.req.valid('json');
    // data is typed as { name: string; email: string }
    return c.json({ success: true, user: data });
  }
);

// Query params
const listSchema = z.object({
  page: z.coerce.number().default(1),
  limit: z.coerce.number().max(100).default(20),
});

app.get(
  '/users',
  zValidator('query', listSchema),
  (c) => {
    const { page, limit } = c.req.valid('query');
    return c.json({ page, limit });
  }
);

// URL params
app.get(
  '/users/:id',
  zValidator('param', z.object({ id: z.string() })),
  (c) => {
    const { id } = c.req.valid('param');
    return c.json({ id });
  }
);
```

### Custom Error Handling

```typescript
app.post(
  '/users',
  zValidator('json', schema, (result, c) => {
    if (!result.success) {
      return c.json({
        error: 'Validation failed',
        issues: z.flattenError(result.error),
      }, 400);
    }
  }),
  handler
);
```

---

## Next.js Server Actions

### Basic Pattern

```typescript
'use server';

import { z } from 'zod';

const loginSchema = z.object({
  email: z.email(),
  password: z.string().min(8),
});

export async function loginAction(prevState: any, formData: FormData) {
  const result = loginSchema.safeParse({
    email: formData.get('email'),
    password: formData.get('password'),
  });

  if (!result.success) {
    return {
      success: false,
      errors: z.flattenError(result.error).fieldErrors,
    };
  }

  // Process validated data
  const { email, password } = result.data;
  // ... authenticate user

  return { success: true };
}
```

### With useActionState

```typescript
'use client';

import { useActionState } from 'react';
import { loginAction } from './actions';

export function LoginForm() {
  const [state, formAction, isPending] = useActionState(loginAction, null);

  return (
    <form action={formAction}>
      <input name="email" type="email" />
      {state?.errors?.email && <span>{state.errors.email[0]}</span>}

      <input name="password" type="password" />
      {state?.errors?.password && <span>{state.errors.password[0]}</span>}

      <button disabled={isPending}>
        {isPending ? 'Loading...' : 'Login'}
      </button>
    </form>
  );
}
```

### ZSA (Zod Server Actions)

```typescript
import { createServerAction } from 'zsa';

const createUser = createServerAction()
  .input(z.object({
    name: z.string(),
    email: z.email(),
  }))
  .output(z.object({
    id: z.string(),
    name: z.string(),
  }))
  .handler(async ({ input }) => {
    const user = await db.users.create({ data: input });
    return user;
  });
```

---

## API Routes (Generic Pattern)

### Request Validation

```typescript
import { z } from 'zod';

const requestSchema = z.object({
  body: z.object({
    name: z.string(),
    email: z.email(),
  }),
  query: z.object({
    include: z.enum(['posts', 'comments']).optional(),
  }),
  params: z.object({
    id: z.string(),
  }),
});

export async function handler(req: Request) {
  const result = requestSchema.safeParse({
    body: await req.json(),
    query: Object.fromEntries(new URL(req.url).searchParams),
    params: extractParams(req),
  });

  if (!result.success) {
    return Response.json(
      { error: z.prettifyError(result.error) },
      { status: 400 }
    );
  }

  const { body, query, params } = result.data;
  // ... handle request
}
```

### Response Validation

```typescript
const responseSchema = z.object({
  success: z.boolean(),
  data: z.object({
    id: z.string(),
    name: z.string(),
  }).optional(),
  error: z.string().optional(),
});

function createResponse(data: unknown) {
  const result = responseSchema.safeParse(data);
  if (!result.success) {
    console.error('Invalid response shape:', z.prettifyError(result.error));
    return Response.json({ success: false, error: 'Internal error' }, { status: 500 });
  }
  return Response.json(result.data);
}
```

---

## OpenAPI/JSON Schema

### Generating OpenAPI Spec

```typescript
import { z } from 'zod';

const UserSchema = z.object({
  id: z.string(),
  name: z.string(),
  email: z.email(),
}).meta({
  id: 'User',
  title: 'User',
  description: 'A user in the system',
});

const CreateUserSchema = z.object({
  name: z.string().min(1),
  email: z.email(),
}).meta({
  id: 'CreateUser',
  title: 'Create User Request',
});

// Register for JSON Schema generation
z.globalRegistry.add(UserSchema, UserSchema.meta());
z.globalRegistry.add(CreateUserSchema, CreateUserSchema.meta());

// Generate all schemas
const schemas = z.toJSONSchema(z.globalRegistry, {
  target: 'openapi-3.0',
});
```

### With zod-to-openapi

```typescript
import { OpenAPIRegistry, OpenApiGeneratorV3 } from '@asteasolutions/zod-to-openapi';

const registry = new OpenAPIRegistry();

registry.registerPath({
  method: 'post',
  path: '/users',
  request: { body: { content: { 'application/json': { schema: CreateUserSchema } } } },
  responses: {
    200: { content: { 'application/json': { schema: UserSchema } } },
  },
});

const generator = new OpenApiGeneratorV3(registry.definitions);
const openApiDocument = generator.generateDocument({
  info: { title: 'My API', version: '1.0.0' },
});
```

---

## Schema Organization Patterns

### Domain-Based Structure

```
src/
  schemas/
    user.ts       # User-related schemas
    post.ts       # Post-related schemas
    common.ts     # Shared schemas (pagination, errors)
    index.ts      # Re-exports
```

### Example: user.ts

```typescript
import { z } from 'zod';

// Base schemas
export const UserIdSchema = z.string().brand<'UserId'>();
export const EmailSchema = z.email().brand<'Email'>();

// Domain schemas
export const UserSchema = z.object({
  id: UserIdSchema,
  name: z.string().min(1).max(100),
  email: EmailSchema,
  role: z.enum(['admin', 'user', 'guest']),
  createdAt: z.date(),
});

// Input schemas
export const CreateUserSchema = UserSchema.pick({
  name: true,
  email: true,
  role: true,
});

export const UpdateUserSchema = CreateUserSchema.partial();

// Query schemas
export const UserQuerySchema = z.object({
  role: z.enum(['admin', 'user', 'guest']).optional(),
  search: z.string().optional(),
  limit: z.number().min(1).max(100).default(20),
  offset: z.number().min(0).default(0),
});

// Types
export type User = z.infer<typeof UserSchema>;
export type CreateUser = z.infer<typeof CreateUserSchema>;
export type UpdateUser = z.infer<typeof UpdateUserSchema>;
export type UserQuery = z.infer<typeof UserQuerySchema>;
```

### Example: common.ts

```typescript
import { z } from 'zod';

// Pagination
export const PaginationSchema = z.object({
  page: z.coerce.number().min(1).default(1),
  limit: z.coerce.number().min(1).max(100).default(20),
});

export const PaginatedResponseSchema = <T extends z.ZodType>(itemSchema: T) =>
  z.object({
    items: z.array(itemSchema),
    total: z.number(),
    page: z.number(),
    limit: z.number(),
    hasMore: z.boolean(),
  });

// Error responses
export const ApiErrorSchema = z.object({
  success: z.literal(false),
  error: z.object({
    code: z.string(),
    message: z.string(),
    details: z.record(z.string(), z.array(z.string())).optional(),
  }),
});

// Success responses
export const ApiSuccessSchema = <T extends z.ZodType>(dataSchema: T) =>
  z.object({
    success: z.literal(true),
    data: dataSchema,
  });
```

---

## Performance Tips

### Avoid Deep Chaining

```typescript
// Slower: many .extend()/.pick() calls
const schema = base.extend({...}).pick({...}).extend({...});

// Faster: compose smaller schemas
const NameSchema = z.object({ name: z.string() });
const EmailSchema = z.object({ email: z.email() });
const schema = z.object({ ...NameSchema.shape, ...EmailSchema.shape });
```

### Use Zod Mini for Edge/Serverless

```typescript
import { z } from 'zod/mini';  // ~1.9kb gzipped
```

### Memoize Repeated Validations

```typescript
const cache = new Map<string, unknown>();

function validateCached<T>(schema: z.ZodType<T>, input: unknown, key: string): T {
  if (cache.has(key)) return cache.get(key) as T;
  const result = schema.parse(input);
  cache.set(key, result);
  return result;
}
```
