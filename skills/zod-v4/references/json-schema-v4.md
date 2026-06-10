# JSON Schema (Zod v4)

## Contents

- [toJSONSchema](#ztojsonschemaschema-params)
- [Parameters](#parameters-you-should-know)
- [Object Modes](#object-modes)
- [Unrepresentable Types](#unrepresentable-types)
- [fromJSONSchema](#zfromjsonschemajsonschema-experimental)
- [Where To Go Next](#where-to-go-next)

Zod v4 has first-party JSON Schema conversion via:

- `z.toJSONSchema(...)` (stable)
- `z.fromJSONSchema(...)` (experimental)

## `z.toJSONSchema(schema, params?)`

```ts
import { z } from "zod";

const S = z.object({
  name: z.string(),
  age: z.number(),
});

const jsonSchema = z.toJSONSchema(S);
```

### Parameters You Should Know

- `io`: Some schemas have different input/output types (pipes, defaults, coercion).
- Default is output; set `io: "input"` for input type.
- `target`: Defaults to draft 2020-12. Docs list `"draft-4"`, `"draft-7"`, `"draft-2020-12"`, and `"openapi-3.0"`.
- `metadata`: Registry to look up metadata. If schemas have `id`, they can be extracted into `$defs`.
- `unrepresentable`: `"throw"` (default) or `"any"` (convert unrepresentable nodes to `{}`).
- `cycles`: `"ref"` (default) or `"throw"`.
- `reused`: `"inline"` (default) or `"ref"` (extract reused schemas into `$defs`).
- `uri`: Map `id` to external ref URIs.
- `override`: Custom override hook to adjust the produced JSON Schema for nodes.

## Object Modes

`z.object()` emits different JSON Schema depending on `io`:

- output mode: `additionalProperties: false`
- input mode: omits `additionalProperties`
- `z.strictObject()`: always rejects additional properties
- `z.looseObject()`: allows additional properties

Use `io: "input"` for request bodies, form values, environment inputs, and
other boundary payloads where the input type matters more than parsed output.

### Unrepresentable Types

Some Zod types cannot be represented as JSON Schema (examples include `z.bigint()`, `z.date()`, `z.map()`, `z.set()`, `z.transform()`, `z.custom()`, `z.undefined()`).

Default behavior is to throw:

```ts
import { z } from "zod";
z.toJSONSchema(z.date()); // throws
```

If you need a "best effort" schema, use:

```ts
z.toJSONSchema(z.date(), { unrepresentable: "any" }); // {}
```

### Cycles (`cycles`)

Cycles are broken by `$ref` by default:

```ts
import { z } from "zod";

const User = z.object({
  name: z.string(),
  get friend() {
    return User;
  },
});

z.toJSONSchema(User); // will contain a $ref
```

### Reused Schemas (`reused`)

By default reused schemas are inlined. Use `reused: "ref"` to extract them into `$defs`.

### Registries and Catalog Output

Passing a registry (including `z.globalRegistry`) can produce a schema catalog with `$defs`.

## `z.fromJSONSchema(jsonSchema)` (Experimental)

`z.fromJSONSchema` exists, but is explicitly experimental and may change. Use it carefully and pin versions if you depend on it.

## Where To Go Next

- Metadata deep dive: `references/metadata-registries-v4.md`
- Rules:
- `rules/jsonschema-tojsonschema-options.md`
- `rules/jsonschema-use-target-openapi-3-0.md`
- `rules/jsonschema-use-io-input-for-boundaries.md`
- `rules/jsonschema-use-metadata-registry-and-ids.md`
- `rules/jsonschema-use-uri-for-external-refs.md`
- `rules/jsonschema-override-hook.md`
- `rules/jsonschema-handle-unrepresentable.md`
- `rules/jsonschema-cycles-reused-override.md`
- `rules/jsonschema-fromjsonschema-experimental.md`
