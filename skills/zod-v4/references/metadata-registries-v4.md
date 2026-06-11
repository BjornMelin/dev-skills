# Metadata and Registries (Zod v4)

Zod v4 stores metadata in registries. This is used for JSON Schema generation and general schema cataloging.

## Custom Registries

Create a strongly-typed registry:

```ts
import { z } from "zod";

const myRegistry = z.registry<{ description: string }>();

const S = z.string();
myRegistry.add(S, { description: "A cool schema!" });

myRegistry.has(S); // true
myRegistry.get(S); // { description: "A cool schema!" }
myRegistry.remove(S);
myRegistry.clear();
```

### Special Handling for `id`

Registries treat `id` specially: registering two schemas with the same `id` throws.

Core rule coverage for registries and metadata:

- `rules/meta-id-must-be-unique.md`
- `rules/meta-register-returns-same-instance.md`
- `rules/meta-use-meta-and-globalregistry.md`
- `rules/meta-meta-is-instance-specific.md`
- `rules/meta-prefer-meta-over-describe.md`
- `rules/meta-declaration-merging-globalmeta.md`

## `z.globalRegistry`

Zod exposes a global registry intended for JSON Schema metadata.

The built-in global meta shape includes `id`, `title`, `description`, `deprecated`, and supports additional keys.

Register metadata explicitly:

```ts
import { z } from "zod";

const Email = z.email().register(z.globalRegistry, {
  id: "email_address",
  title: "Email address",
  description: "Your email address",
  examples: ["first.last@example.com"],
});
```

This reference intentionally does not restate the `.meta()` / `.describe()` / `.register()` basics.
Jump to the rules above for the atomic behaviors and gotchas.

## JSON Schema Integration

- `z.toJSONSchema(schema)` will include global registry metadata from `z.globalRegistry`.
- You can pass a custom registry via the `metadata` option to `z.toJSONSchema`.
- Passing a registry into `z.toJSONSchema` can generate a schema catalog with `$defs`.

## Where To Go Next

- Rules:
- `rules/meta-use-meta-and-globalregistry.md`
- `rules/meta-meta-is-instance-specific.md`
- `rules/meta-id-must-be-unique.md`
- `rules/meta-register-returns-same-instance.md`
- `rules/meta-prefer-meta-over-describe.md`
- `rules/meta-declaration-merging-globalmeta.md`
- JSON Schema options: `references/json-schema-v4.md`
