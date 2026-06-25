# Custom Search Serialization

Customize parsing/stringifying only when the default JSON-first search format is not sufficient.

## Current API

Use top-level router options:

```tsx
const router = createRouter({
  routeTree,
  parseSearch,
  stringifySearch,
})
```

## Rules

- Do not use stale nested search serializer object examples.
- Keep custom serializers deterministic and reversible.
- Prefer defaults unless URLs must interoperate with existing external contracts.
- Add tests around arrays, nested objects, empty values, and defaults.
