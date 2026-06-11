# Schema Surface Highlights

## Contents

- [String Formats](#string-formats)
- [Template Literals](#template-literals)
- [Records](#records)
- [Unions and XOR](#unions-and-xor)
- [Refinements and when](#refinements-and-when)
- [Defaults, Prefaults, and Catch](#defaults-prefaults-and-catch)

## String Formats

Prefer top-level string format schemas in Zod 4.4.3:

```ts
import { z } from "zod";

z.email();
z.url();
z.httpUrl();
z.hostname();
z.e164();
z.uuid();
z.uuidv4();
z.uuidv6();
z.uuidv7();
z.guid();
z.ipv4();
z.ipv6();
z.cidrv4();
z.cidrv6();
z.mac();
z.hex();
z.hash("sha256");
z.jwt();
z.base64();
z.base64url();
z.nanoid();
z.cuid();
z.cuid2();
z.ulid();
z.iso.date();
z.iso.time();
z.iso.datetime();
z.iso.duration();
```

Use `z.stringFormat(name, validatorOrRegex)` for custom named string formats.

## Template Literals

Use `z.templateLiteral([…])` for template-literal-shaped strings instead of
ad hoc regexes when the parts map naturally to schemas:

```ts
import { z } from "zod";

const CssLength = z.templateLiteral([z.number(), z.enum(["px", "em", "rem"])]);
```

## Records

Prefer explicit key schemas:

```ts
import { z } from "zod";

z.record(z.string(), z.number());
z.record(z.number(), z.string()); // validates numeric string keys
```

Enum and literal key schemas are exhaustive in v4:

```ts
const Keys = z.enum(["id", "name"]);
z.record(Keys, z.string()); // requires both keys
z.partialRecord(Keys, z.string()); // optional keys
```

Use `z.looseRecord()` when non-matching keys should pass through.

## Unions and XOR

Use discriminated unions when a stable discriminator exists. Use `z.xor([…])`
when exactly one branch must match:

```ts
import { z } from "zod";

const Payment = z.xor([
  z.object({ cardToken: z.string() }),
  z.object({ bankAccountId: z.string() }),
]);
```

## Refinements and when

Use `when` for refinements that should still run when unrelated fields fail
validation. Keep the `when` predicate narrow and based on the fields needed by
the refinement.

## Defaults, Prefaults, and Catch

- `.default(value)` short-circuits on `undefined`; the default must be
  assignable to the output type.
- `.prefault(value)` parses the fallback; the prefault must be assignable to
  the input type.
- `.catch(valueOrFn)` supplies a fallback after validation failure.
- During encode, defaults, prefaults, and catches are not applied.
