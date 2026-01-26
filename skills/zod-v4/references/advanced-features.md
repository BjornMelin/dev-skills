# Zod v4 Advanced Features

## Codecs (Bidirectional Transformations)

Codecs enable type-safe encode/decode operations for network boundaries, database serialization, and API transformations.

### Core Operations

```typescript
// Forward: Input -> Output
schema.parse(data)    // Accepts unknown
schema.decode(data)   // Type-safe input

// Backward: Output -> Input
schema.encode(data)   // Type-safe reverse
```

### Defining Codecs

```typescript
const stringToDate = z.codec(
  z.iso.datetime(),  // input schema
  z.date(),          // output schema
  {
    decode: (isoString) => new Date(isoString),
    encode: (date) => date.toISOString(),
  }
);

stringToDate.decode("2024-01-01T00:00:00Z"); // Date
stringToDate.encode(new Date());              // ISO string
```

### Built-in Codecs

```typescript
// String to number
z.coerce.number()

// ISO datetime to Date
z.pipe(z.iso.datetime(), z.transform(v => new Date(v)))

// JSON codec pattern
const jsonCodec = <T extends z.ZodType>(schema: T) =>
  z.codec(z.string(), schema, {
    decode: (str, ctx) => {
      try { return JSON.parse(str); }
      catch (err) {
        ctx.issues.push({ code: "invalid_format", format: "json", input: str, message: err.message });
        return z.NEVER;
      }
    },
    encode: (value) => JSON.stringify(value),
  });
```

### Async Support

```typescript
await schema.decodeAsync(data);
await schema.encodeAsync(data);
const result = await schema.safeDecodeAsync(data);
```

### Encoding Rules

- Refinements (`.refine()`, `.min()`, `.max()`) run in both directions
- Mutating transforms (`.trim()`, `.toLowerCase()`) work bidirectionally
- `.default()` and `.catch()` only apply during decode
- `.transform()` is unidirectional (throws on encode)

---

## Error Customization

### Unified Error Parameter (v4)

```typescript
// String message
z.string("Not a string!");
z.string().min(5, "Too short!");
z.email("Invalid email");

// Error map function
z.string({
  error: (iss) => iss.input === undefined
    ? "Field is required."
    : "Invalid input."
});

// Available context in error function
// - code: Issue code
// - input: Input data
// - inst: Schema instance
// - path: Error path
// - Method-specific: minimum, maximum, inclusive, etc.
```

### Precedence (Highest to Lowest)

1. Schema-level `error` parameter
2. Per-parse error maps
3. Global error configuration
4. Locale error maps

### Per-Parse Customization

```typescript
schema.parse(data, {
  error: (iss) => "Custom error for this parse"
});
```

### Global Configuration

```typescript
z.config({
  customError: (iss) => {
    if (iss.code === "invalid_type") {
      return `Invalid type, expected ${iss.expected}`;
    }
    // Return undefined to defer to next level
  },
});
```

### Internationalization

```typescript
import { en, es, fr } from "zod/locales";

z.config(en());  // English (default)
z.config(es());  // Spanish
z.config(fr());  // French

// Dynamic loading
const locale = await import(`zod/v4/locales/${lang}.js`);
z.config(locale.default());
```

Available: 40+ languages including ar, de, es, fr, ja, ko, pt, ru, zh.

### Including Input in Errors

```typescript
// Disabled by default (security)
z.string().parse(12, { reportInput: true });
```

---

## Error Formatting

### z.prettifyError()

Human-readable string for CLI/logging:

```typescript
const result = schema.safeParse(data);
if (!result.success) {
  console.log(z.prettifyError(result.error));
}
// Output:
// ✖ Invalid type, expected string
//   → at username
// ✖ Too short
//   → at password
```

### z.treeifyError()

Nested object structure mirroring schema:

```typescript
const tree = z.treeifyError(error);
// {
//   errors: string[],
//   properties: { username: { errors: [...] } },
//   items: [undefined, { errors: [...] }]
// }

const usernameErrors = tree.properties?.username?.errors;
```

### z.flattenError()

Flat structure for forms:

```typescript
const flat = z.flattenError(error);
// {
//   formErrors: string[],      // Top-level errors
//   fieldErrors: {
//     username: string[],
//     password: string[]
//   }
// }
```

---

## Metadata and Registries

### Using .meta()

```typescript
const emailSchema = z.email().meta({
  id: "email_address",
  title: "Email Address",
  description: "User's email",
  examples: ["user@example.com"],
  deprecated: false
});

// Retrieve metadata
emailSchema.meta(); // { id: "...", title: "...", ... }

// Shorthand for description only
emailSchema.describe("An email address");
```

### Global Registry

```typescript
// Register schemas
z.globalRegistry.add(UserSchema, { id: "User", title: "User" });
z.globalRegistry.add(PostSchema, { id: "Post", title: "Post" });

// Check registration
z.globalRegistry.has(UserSchema); // true

// Convert all to JSON Schema
const schemas = z.toJSONSchema(z.globalRegistry);
```

### Custom Registries

```typescript
const myRegistry = z.registry<{ description: string }>();
myRegistry.add(z.string(), { description: "A string" });

// Type-constrained registry
const stringRegistry = z.registry<{ desc: string }, z.ZodString>();
```

### .register() Method

Returns original instance (unique among methods):

```typescript
schema.register(myRegistry, { description: "..." });
// => schema (not a new instance)
```

---

## JSON Schema Conversion

### Basic Usage

```typescript
const jsonSchema = z.toJSONSchema(schema);
```

### Configuration

```typescript
z.toJSONSchema(schema, {
  target: "draft-2020-12",     // draft-4, draft-7, draft-2020-12, openapi-3.0
  metadata: z.globalRegistry,  // Include metadata
  unrepresentable: "throw",    // "throw" | "any"
  cycles: "ref",               // "ref" | "throw"
  reused: "inline",            // "inline" | "ref"
  io: "output",                // "output" | "input"
});
```

### OpenAPI Target

```typescript
z.toJSONSchema(schema, { target: "openapi-3.0" });
```

### Handling Unrepresentable Types

Non-representable: `z.bigint()`, `z.date()`, `z.map()`, `z.set()`, `z.transform()`, `z.custom()`, `z.lazy()`, `z.promise()`

```typescript
z.toJSONSchema(schema, {
  unrepresentable: "any",  // Convert to {}
  override: (ctx) => {
    if (ctx.schema._def.typeName === "ZodDate") {
      return { type: "string", format: "date-time" };
    }
  }
});
```

### File Schemas in JSON Schema

```typescript
const fileSchema = z.file().min(1024).max(5242880).mime(["image/png"]);
// Converts to:
// {
//   "type": "string",
//   "format": "binary",
//   "contentEncoding": "binary",
//   "contentMediaType": "image/png",
//   "minLength": 1024,
//   "maxLength": 5242880
// }
```

### Registry-Based Conversion

```typescript
z.globalRegistry.add(UserSchema, { id: "User" });
z.globalRegistry.add(PostSchema, { id: "Post" });

const allSchemas = z.toJSONSchema(z.globalRegistry);
// Returns interconnected schemas with $ref pointers
```

---

## Branded Types

Nominal typing for type-safe domain modeling:

```typescript
const UserId = z.string().brand<"UserId">();
const PostId = z.string().brand<"PostId">();

type UserId = z.infer<typeof UserId>;
type PostId = z.infer<typeof PostId>;

const userId: UserId = UserId.parse("user-123");
const postId: PostId = PostId.parse("post-456");

// TypeScript error: types not assignable
const wrong: UserId = postId;
```

### Common Use Cases

```typescript
// IDs
const Email = z.email().brand<"Email">();
const Currency = z.number().positive().brand<"Currency">();
const SanitizedHtml = z.string().brand<"SanitizedHtml">();

// Validated domain values
const ValidatedUrl = z.url().brand<"ValidatedUrl">();
```

---

## Recursive Schemas (v4)

No type casting required:

```typescript
const Category = z.object({
  name: z.string(),
  get subcategories() { return z.array(Category) }
});

type Category = z.infer<typeof Category>;
// { name: string; subcategories: Category[] }
```

### Mutually Recursive

```typescript
const Node = z.object({
  value: z.string(),
  get children() { return z.array(Edge) }
});

const Edge = z.object({
  label: z.string(),
  get target() { return Node }
});
```

---

## File Schemas

```typescript
z.file()
z.file().min(1024)           // Min bytes
z.file().max(1024 * 1024)    // Max bytes
z.file().mime(["image/png", "image/jpeg"])

// Combined
const imageSchema = z.file()
  .max(5 * 1024 * 1024)
  .mime(["image/png", "image/jpeg", "image/webp"]);
```

---

## Template Literals

Pattern-based string validation:

```typescript
z.templateLiteral([z.string(), "-", z.number()])
// Matches: "abc-123", "test-456"

z.templateLiteral(["user_", z.string()])
// Matches: "user_alice", "user_bob"
```

---

## Overwrite Method (v4)

Transform without changing type (stored as refinement):

```typescript
const schema = z.string().overwrite(s => s.trim());
type T = z.infer<typeof schema>; // string (not affected)
```
