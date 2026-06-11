# Codecs (Encode/Decode) (Zod v4)

## Contents

- [parse vs decode vs encode](#parse-vs-decode-vs-encode)
- [z.codec](#zcodecinputschema-outputschema--decode-encode-)
- [Inverting Codecs](#inverting-codecs)
- [Direction Rules](#direction-rules)
- [Stringbool](#stringbool)
- [Where To Go Next](#where-to-go-next)

Zod v4 lets schemas run in two directions:

- Forward: parse/decode (input -> output)
- Backward: encode (output -> input)

Most schemas have identical input/output types, so directions are equivalent.
Codecs (`z.codec`) are the main reason to care: they explicitly map between two different schemas.

## `.parse()` vs `.decode()` vs `.encode()`

- `.parse(value)` accepts `unknown` and returns the output type.
- `.decode(value)` is typically the same runtime behavior as `.parse`, but is intended for typed inputs.
- `.encode(value)` runs the schema "backwards" (important for codecs).

Top-level helpers exist too:

```ts
import { z } from "zod";

z.decode(schema, value);
z.encode(schema, value);
```

## `z.codec(inputSchema, outputSchema, { decode, encode })`

```ts
import { z } from "zod";

export const IsoDatetimeToDate = z.codec(z.iso.datetime(), z.date(), {
  decode: (isoString) => new Date(isoString),
  encode: (date) => date.toISOString(),
});
```

Use a codec at boundaries so the rest of the app can work with rich types:

- HTTP: string -> Date
- DB: string -> bigint (if you keep bigint out of JSON boundaries)
- Queues: JSON string -> object

## Type-Safe Inputs

For codecs, `.decode` and `.encode` are strongly typed:

```ts
IsoDatetimeToDate.decode("2024-01-15T10:30:00.000Z"); // Date
IsoDatetimeToDate.encode(new Date()); // string
```

## Composability

Codecs can be nested inside objects/arrays/pipes like any other schema.

```ts
import { z } from "zod";

const Payload = z.object({ startDate: IsoDatetimeToDate });
Payload.decode({ startDate: "2024-01-15T10:30:00.000Z" }); // { startDate: Date }
```

## Inverting Codecs

Use `z.invertCodec(codec)` when the reverse direction should be a first-class
schema:

```ts
import { z } from "zod";

const StringToDate = z.codec(z.iso.datetime(), z.date(), {
  decode: (isoString) => new Date(isoString),
  encode: (date) => date.toISOString(),
});

const DateToString = z.invertCodec(StringToDate);
```

`z.invertCodec()` only inverts the codec you pass to it. It does not
recursively invert nested codecs inside another schema.

## Direction Rules

- Unidirectional transforms (`.transform(...)` / `z.transform(...)`) are not reversible; encoding will throw.
- Pipes run in reverse order during encoding.
- String mutators like `.trim()` / `.toLowerCase()` also run during encode; they are not reversed.
- Defaults, prefaults, and catches apply only in the forward direction.
- `.default()` short-circuits and must be assignable to the output type.
- `.prefault()` is parsed and must be assignable to the input type.

## Stringbool

`z.stringbool()` is implemented as a codec in Zod 4.4.3. It decodes common
boolish strings to booleans and encodes booleans back to strings:

```ts
import { z } from "zod";

const Bool = z.stringbool();

Bool.decode("false"); // false
Bool.encode(true); // "true"
```

When custom truthy/falsy arrays are provided, encode uses the first configured
truthy/falsy string.

## Where To Go Next

- Rule: use codecs for bidirectional transforms: `rules/codec-use-z-codec-for-bidirectional.md`
- Rule: when to use parse/safeParse vs decode/encode: `rules/codec-prefer-decode-encode-for-typed-boundaries.md`
