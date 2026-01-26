# Zod v3 to v4 Migration Guide

## Error Customization (Breaking)

### Unified `error` Parameter

```typescript
// v3
z.string().min(5, { message: "Too short" });
z.string({ invalid_type_error: "Not a string", required_error: "Required" });

// v4
z.string().min(5, { error: "Too short" });
z.string({ error: (iss) => iss.input === undefined ? "Required" : "Not a string" });
```

### Error Map Precedence Reversed

v4: Schema-level > per-parse > global > locale (schema wins)
v3: Global > schema-level (global won)

### Deprecated Error Methods

```typescript
// v3
error.format()   // Deprecated
error.flatten()  // Deprecated
error.formErrors // Dropped
error.addIssue() // Deprecated

// v4
z.treeifyError(error)
z.flattenError(error)
z.prettifyError(error)
```

---

## String Format APIs (Breaking)

Methods moved to top-level functions:

```typescript
// v3
z.string().email()
z.string().uuid()
z.string().url()
z.string().ip()
z.string().cidr()
z.string().ipv4()
z.string().ipv6()

// v4
z.email()
z.uuid()
z.url()
z.ipv4()        // .ip() removed
z.ipv6()
z.cidrv4()      // .cidr() split
z.cidrv6()
z.base64()
z.base64url()
z.jwt()
```

### Stricter UUID Validation

v4 enforces RFC 4122 compliance. Use `z.guid()` for permissive matching.

### Custom Email Regex

```typescript
z.email({ pattern: "gmail" })   // Gmail-specific
z.email({ pattern: "html5" })   // HTML5 pattern
z.email({ pattern: /custom/ })  // Custom regex
```

---

## Number Changes (Breaking)

### No Infinite Values

```typescript
// v4 rejects Infinity and -Infinity
z.number().parse(Infinity);  // Throws
```

### Stricter .safe() and .int()

```typescript
// v3: .safe() accepted floats, .int() accepted any integer
// v4: .safe() only safe integers, .int() only MIN_SAFE to MAX_SAFE

z.number().safe().parse(1.5);           // v4: Throws
z.number().int().parse(9007199254740993); // v4: Throws (> MAX_SAFE_INTEGER)
```

### New Fixed-Width Types

```typescript
z.int32()   // 32-bit signed
z.uint32()  // 32-bit unsigned
z.float32() // 32-bit float
z.int64()   // 64-bit signed
z.uint64()  // 64-bit unsigned
```

---

## Object Changes (Breaking)

### Deprecated Methods

```typescript
// v3
z.object({}).strict()
z.object({}).passthrough()
z.object({}).strip()
z.object({}).merge(other)
z.object({}).deepPartial()
z.object({}).nonstrict()

// v4
z.strictObject({})         // Rejects unknown keys
z.looseObject({})          // Allows unknown keys
z.object({})               // Allows by default
obj.extend(other.shape)    // Instead of .merge()
// .deepPartial() removed - nest .partial() manually
```

### pick/omit with refinements

As of v4.3.x, calling `.pick()` or `.omit()` on schemas with refinements throws.
Rebuild the object from `.shape` instead:

```typescript
const Base = z.object({ id: z.string(), name: z.string() });
const Refined = Base.refine(val => val.id.length > 0);

// ❌ throws in v4.3.x
// Refined.pick({ id: true })

// ✅ rebuild from shape
const Picked = z.object({ id: Base.shape.id });
```

### Stricter key validation in pick/omit

As of v4.3.x, `.pick()` and `.omit()` throw on unknown keys. Ensure keys exist in the schema.

```typescript
const Schema = z.object({ a: z.string() });

// ❌ throws in v4.3.x
// Schema.pick({ missing: true })
```

### extend with refinements

As of v4.3.x, calling `.extend()` on refined schemas throws. Use `.safeExtend()` if you need to add fields without dropping refinements:

```typescript
const Refined = z.object({ a: z.string() }).refine(val => val.a.length > 0);

// ❌ throws in v4.3.x
// Refined.extend({ b: z.number() })

// ✅ preserves refinements
const Extended = Refined.safeExtend({ b: z.number() });
```

### Defaults in Optional Fields

```typescript
const schema = z.object({ a: z.string().default("x").optional() });
schema.parse({});
// v3: {}
// v4: { a: "x" }
```

### z.unknown() Optionality

```typescript
z.object({ data: z.unknown() });
// v3: data was optional in inferred type
// v4: data is required (use .optional() explicitly)
```

---

## Default/Prefault (Breaking)

```typescript
// v3: .default() applied before transforms
// v4: .default() applies to output (post-transform)

const schema = z.string().transform(s => s.length).default(5);
// v4: default is number 5, not string

// Use .prefault() for v3-like behavior
z.string().transform(s => s.length).prefault("hello");
// Missing => parses "hello" => 5
```

---

## Record Changes (Breaking)

### Key Schema Required

```typescript
// v3
z.record(z.number())  // Allowed

// v4
z.record(z.string(), z.number())  // Required
```

### Enum Records Are Exhaustive

```typescript
const Keys = z.enum(["a", "b"]);

// v4: Creates { a: number; b: number } (exhaustive)
z.record(Keys, z.number())

// v4: Creates { a?: number; b?: number } (optional)
z.partialRecord(Keys, z.number())
```

---

## Function Changes (Breaking)

```typescript
// v3
z.function()
  .args(z.string(), z.number())
  .returns(z.boolean())

// v4
z.function({
  input: [z.string(), z.number()],
  output: z.boolean()
}).implement((str, num) => str.length > num)

// Async
z.function({ input: [z.string()], output: z.number() })
  .implementAsync(async (str) => str.length)
```

---

## Deprecated/Removed APIs

### z.nativeEnum() Deprecated

```typescript
// v3
enum Color { Red, Green, Blue }
z.nativeEnum(Color)

// v4
z.enum(Color)  // Overloaded z.enum() handles TS enums
```

### z.promise() Deprecated

```typescript
// v3
z.promise(z.string())

// v4 (alternative)
z.custom<Promise<string>>()
```

### Removed Shorthand Types

```typescript
// v3
z.ostring()   // z.string().optional()
z.onumber()   // z.number().optional()
z.oboolean()  // z.boolean().optional()

// v4: Removed, use explicit .optional()
```

### z.literal() Symbol Support Dropped

```typescript
// v3
z.literal(Symbol.for("x"))  // Allowed

// v4
z.literal(Symbol.for("x"))  // Not supported
```

### Static .create() Factories Dropped

```typescript
// v3
z.string.create()  // Existed

// v4
z.string()  // Use directly
```

---

## Refine Changes

### Type Predicates Ignored

```typescript
// v3: Type predicates narrowed types
z.unknown().refine((x): x is string => typeof x === "string")

// v4: Type predicates ignored, use .transform() for narrowing
```

### ctx.path Dropped

```typescript
// v3
.refine((val) => true, { path: ctx.path })

// v4
.refine((val) => true, { path: ["fieldName"] })  // Explicit path
```

---

## New Features in v4

### Multiple Literal Values

```typescript
z.literal([200, 201, 204])  // Union of literals
```

### Template Literals

```typescript
z.templateLiteral(["user_", z.string()])
```

### Stringbool

```typescript
z.stringbool()  // "true"/"false" => boolean
```

### Recursive Objects (No Casting)

```typescript
const Category = z.object({
  name: z.string(),
  get subcategories() { return z.array(Category) }
});
```

### File Schemas

```typescript
z.file().max(5 * 1024 * 1024).mime(["image/png"])
```

### Native JSON Schema

```typescript
z.toJSONSchema(schema, { target: "openapi-3.0" })
```

### Metadata System

```typescript
schema.meta({ title: "User", description: "..." })
```

### Internationalization

```typescript
import { es } from "zod/locales";
z.config(es());
```

### Error Pretty-Printing

```typescript
z.prettifyError(error)
```

### .overwrite() Method

```typescript
z.string().overwrite(s => s.trim())  // Transform without type change
```

---

## Migration Tools

### Automated Codemod

```bash
npx zod-v3-to-v4
```

### Subpath Imports (Incremental Migration)

```typescript
import { z } from "zod";       // v4 (default)
import * as z from "zod/mini"; // Zod Mini
import * as z from "zod/v3";   // v3 fallback only if required
```

### Library Authors

```json
{
  "peerDependencies": {
    "zod": "^3.25.0 || ^4.0.0"
  }
}
```
