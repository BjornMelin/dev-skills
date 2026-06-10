# Formatting Errors (Zod v4)

Zod v4 provides utilities to convert a `$ZodError` into formats that are easier to consume.

## `z.prettifyError(error)`

Use for logs, CLI output, and simple API error strings.

```ts
import { z } from "zod";

const schema = z.strictObject({
  username: z.string(),
  favoriteNumbers: z.array(z.number()),
});

const result = schema.safeParse({
  username: 1234,
  favoriteNumbers: [1234, "4567"],
  extraKey: 1234,
});

if (!result.success) {
  console.log(z.prettifyError(result.error));
}
```

## `z.treeifyError(error)`

Use for nested UIs where you want to traverse the error tree.

```ts
import { z } from "zod";

const tree = z.treeifyError(error);

tree.properties?.username?.errors;
tree.properties?.favoriteNumbers?.items?.[1]?.errors;
```

Always use optional chaining to avoid runtime errors while traversing.

## `z.flattenError(error)`

Use for form-shaped payloads (1 level deep field errors).

```ts
import { z } from "zod";

const flat = z.flattenError(error);
// { formErrors: string[]; fieldErrors: Record<string, string[]> }
```

## `z.formatError(error)` is Deprecated

`z.formatError()` exists but is deprecated in favor of `z.treeifyError()`.

If you see code using:

- `z.formatError(err)`
- `err.format()`

Migrate to `z.treeifyError(err)`.

## Where To Go Next

- Rules:
- `rules/error-use-prettify-treeify-flatten.md`
- `rules/error-avoid-z-formaterror.md`
