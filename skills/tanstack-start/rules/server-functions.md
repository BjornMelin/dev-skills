# Server Functions

Use `createServerFn` for same-origin, type-safe server RPCs called from loaders, components, hooks, and event handlers.

## Current pattern

```tsx
import { createServerFn } from '@tanstack/react-start'
import { z } from 'zod'

const createApplication = createServerFn({ method: 'POST' })
  .validator(z.object({ applicationId: z.string().min(1) }))
  .handler(async ({ data, context }) => {
    await requireUser(context)
    return saveApplication(data.applicationId)
  })
```

## Rules

- Use `.validator(...)`; `.inputValidator` is legacy/deprecated.
- Use `GET` for reads and `POST` for mutations.
- Keep secrets, DB clients, filesystem, and private SDK calls inside server functions or server routes.
- Return serializable values.
- Validate authorization in the handler or attached middleware, not only in route guards.
