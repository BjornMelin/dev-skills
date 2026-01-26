# Zod v4 API Reference

## Primitives

```typescript
z.string()
z.number()
z.bigint()
z.boolean()
z.symbol()
z.undefined()
z.null()
z.void()
z.any()
z.unknown()
z.never()
```

### Coercion

```typescript
z.coerce.string()   // "42" from 42
z.coerce.number()   // 42 from "42"
z.coerce.boolean()
z.coerce.bigint()
z.coerce.date()
```

## String Formats (Top-Level in v4)

```typescript
// Email
z.email()
z.email({ error: "Invalid email" })

// UUIDs
z.uuid()       // Any version
z.uuidv4()
z.uuidv6()
z.uuidv7()
z.guid()       // Permissive matching

// URLs
z.url()
z.url({ protocols: ["https"] })

// Date/Time (ISO formats)
z.iso.datetime()
z.iso.datetime({ offset: true })
z.iso.datetime({ precision: 3 })
z.iso.date()   // YYYY-MM-DD
z.iso.time()   // HH:MM:SS

// Network
z.ipv4()
z.ipv6()
z.cidrv4()
z.cidrv6()
z.mac()

// Other
z.jwt()
z.hash("sha256")
z.base64()
z.base64url()
```

## String Validations

```typescript
z.string().min(5)
z.string().max(10)
z.string().length(8)
z.string().regex(/^\d+$/)
z.string().startsWith("pre")
z.string().endsWith("fix")
z.string().includes("mid")
z.string().uppercase()
z.string().lowercase()

// Transforms
z.string().toLowerCase()
z.string().toUpperCase()
z.string().trim()
```

## Numbers

```typescript
z.number()
z.number().gt(5)
z.number().gte(5)
z.number().lt(10)
z.number().lte(10)
z.number().positive()
z.number().negative()
z.number().multipleOf(5)
z.number().finite()
z.number().safe()

// Integers
z.int()        // Safe integer
z.int32()      // 32-bit signed
```

## Objects

```typescript
// Standard (allows unknown keys by default)
z.object({ name: z.string(), age: z.number() })

// Strict (rejects unknown keys)
z.strictObject({ name: z.string() })

// Loose (explicitly allows unknown keys)
z.looseObject({ name: z.string() })
```

### Object Methods

```typescript
const User = z.object({ name: z.string(), age: z.number() });

User.shape.name           // Access field schema
User.keyof()              // z.enum(["name", "age"])
User.extend({ email: z.string() })
User.safeExtend({ email: z.string() }) // preserves refinements
User.pick({ name: true })
User.omit({ age: true })
User.partial()            // All optional
User.required()           // All required
User.catchall(z.string()) // Unknown keys must be strings
```

## Arrays and Tuples

```typescript
// Arrays
z.array(z.string())
z.array(z.number()).min(1)
z.array(z.number()).max(10)
z.array(z.number()).length(5)
z.array(z.number()).nonempty()

// Tuples
z.tuple([z.string(), z.number()])
z.tuple([z.string()], z.number())  // Variadic: [string, ...number[]]
```

## Unions and Intersections

```typescript
// Union (OR)
z.union([z.string(), z.number()])

// Discriminated Union (efficient)
z.discriminatedUnion("status", [
  z.object({ status: z.literal("success"), data: z.string() }),
  z.object({ status: z.literal("error"), error: z.string() })
])

// Intersection (AND)
z.intersection(
  z.object({ name: z.string() }),
  z.object({ age: z.number() })
)
```

## Enums

```typescript
// Array enum
const Role = z.enum(["admin", "user", "guest"]);
Role.exclude(["guest"])
Role.extract(["admin"])

// TypeScript enum (v4 - use z.enum directly)
enum Color { Red, Green, Blue }
z.enum(Color)  // Not z.nativeEnum() in v4
```

## Records, Maps, Sets

```typescript
// Record (always specify key schema in v4)
z.record(z.string(), z.number())
z.record(z.enum(["a", "b"]), z.number())  // Exhaustive
z.partialRecord(z.enum(["a", "b"]), z.number())  // Optional keys

// Maps and Sets
z.map(z.string(), z.number())
z.set(z.string())
z.set(z.number()).min(2).max(10)
```

## Refinements

```typescript
// Basic refinement
z.string().refine(val => val.length >= 5, {
  error: "Must be at least 5 characters"
})

// With path for objects
z.object({ start: z.date(), end: z.date() })
  .refine(data => data.end > data.start, {
    error: "End must be after start",
    path: ["end"]
  })

// superRefine for multiple issues
z.string().superRefine((val, ctx) => {
  if (val.length < 5) {
    ctx.addIssue({
      code: z.ZodIssueCode.too_small,
      minimum: 5,
      type: "string",
      inclusive: true,
      error: "Too short"
    });
  }
})

// Simple check
z.string().check(val => val.length >= 5)
```

## Transforms

```typescript
// Transform
z.string().transform(val => val.length)
z.string().transform(val => val.toUpperCase())

// Pipe (chain schemas)
z.pipe(
  z.string(),
  z.coerce.number(),
  z.number().positive()
)

// Preprocess
z.preprocess(
  val => String(val).trim(),
  z.string().min(1)
)
```

## Defaults and Catch

```typescript
// Default (for missing/undefined)
z.string().default("guest")
z.number().default(() => Date.now())

// Prefault (parse default as input - v4)
z.string().transform(s => s.toUpperCase()).prefault("hello")
// Missing input => "HELLO"

// Catch (fallback on error)
z.number().catch(0)
```

## Type Inference

```typescript
const User = z.object({ name: z.string() });

// Infer output type
type User = z.infer<typeof User>;

// For transforms, get input/output separately
const schema = z.string().transform(val => val.length);
type Input = z.input<typeof schema>;   // string
type Output = z.output<typeof schema>; // number
```

## Parsing

```typescript
// Throws on failure
const data = schema.parse(input);
await schema.parseAsync(input);

// Safe (returns result object)
const result = schema.safeParse(input);
if (result.success) {
  result.data;  // Validated
} else {
  result.error; // ZodError
}
await schema.safeParseAsync(input);
```

## Advanced Types

```typescript
// Literals (multiple values in v4)
z.literal("admin")
z.literal([200, 201, 204])

// Template literals
z.templateLiteral([z.string(), "-", z.number()])

// Stringbool
z.stringbool()  // "true"/"false" => boolean

// Files
z.file()
z.file().min(1024).max(1024 * 1024)
z.file().mime(["image/png"])

// Branded types
const UserId = z.string().brand<"UserId">();

// Readonly
z.object({ name: z.string() }).readonly()

// JSON
z.json()

// Custom
z.custom<MyType>(val => val instanceof MyType)

// Recursive
const Category = z.object({
  name: z.string(),
  get subcategories() { return z.array(Category) }
});

// Functions
z.function({
  input: [z.string(), z.number()],
  output: z.boolean()
}).implement((str, num) => str.length > num)
```
