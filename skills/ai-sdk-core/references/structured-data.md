# AI SDK Core: Structured Data Generation Reference

Comprehensive guide to generating structured data with AI SDK Core v6, covering `generateObject`, `streamObject`, Output API, and Zod schema best practices.

---

## 1. generateObject Patterns

### Basic Usage with Zod Schema

Generate structured data from a prompt using Zod schema for validation:

```typescript
import { generateObject } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const { object } = await generateObject({
  model: openai('gpt-4o'),
  schema: z.object({
    recipe: z.object({
      name: z.string(),
      ingredients: z.array(
        z.object({
          name: z.string(),
          amount: z.string()
        })
      ),
      steps: z.array(z.string()),
    }),
  }),
  prompt: 'Generate a lasagna recipe.',
});

// object is typed and validated automatically
console.log(object.recipe.name);
```

### Schema Modes

#### Object Mode (Default)

Returns a single object matching the schema:

```typescript
const { object } = await generateObject({
  model: openai('gpt-4o'),
  // No output specified = object mode (default)
  schema: z.object({
    name: z.string(),
    age: z.number(),
    email: z.email(),
  }),
  prompt: 'Generate a test user.',
});
```

#### Array Mode

Generate an array of objects. Schema defines the shape of each element:

```typescript
const { object } = await generateObject({
  model: openai('gpt-4o'),
  output: 'array',
  schema: z.object({
    name: z.string(),
    class: z.string().describe('Character class, e.g. warrior, mage, or thief.'),
    description: z.string(),
  }),
  prompt: 'Generate 3 hero descriptions for a fantasy RPG.',
});

// object is an array of heroes
for (const hero of object) {
  console.log(hero.name, hero.class);
}
```

#### Enum Mode

Generate a specific enum value for classification tasks:

```typescript
const { object } = await generateObject({
  model: openai('gpt-4o'),
  output: 'enum',
  enum: ['action', 'comedy', 'drama', 'horror', 'sci-fi'],
  prompt:
    'Classify the genre of this movie plot: ' +
    '"A group of astronauts travel through a wormhole in search of a ' +
    'new habitable planet for humanity."',
});

// object is one of the enum values: 'sci-fi'
console.log(object);
```

#### No-Schema Mode

Generate dynamic JSON without schema validation:

```typescript
const { object } = await generateObject({
  model: openai('gpt-4o'),
  output: 'no-schema',
  prompt: 'Generate a lasagna recipe.',
});

// object is unknown type - no validation
```

### Schema Naming and Descriptions for LLM Guidance

Provide schema name and description to improve LLM understanding:

```typescript
const { object } = await generateObject({
  model: openai('gpt-4o'),
  schemaName: 'Recipe',
  schemaDescription: 'A recipe for a dish.',
  schema: z.object({
    name: z.string(),
    ingredients: z.array(
      z.object({
        name: z.string(),
        amount: z.string()
      })
    ),
    steps: z.array(z.string()),
  }),
  prompt: 'Generate a lasagna recipe.',
});
```

**Best Practice**: Use `schemaName` and `schemaDescription` for:
- Complex schemas with multiple nested objects
- Tool-based generation modes where the schema name appears in tool calls
- Disambiguation when using multiple similar schemas

### Error Handling: NoObjectGeneratedError

Handle generation failures gracefully:

```typescript
import { generateObject, NoObjectGeneratedError } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

try {
  const { object } = await generateObject({
    model: openai('gpt-4o'),
    schema: z.object({
      name: z.string(),
      age: z.number(),
    }),
    prompt: 'Generate a user.',
  });
} catch (error) {
  if (NoObjectGeneratedError.isInstance(error)) {
    console.error('Failed to generate object:');
    console.error('Cause:', error.cause);
    console.error('Generated text:', error.text);
    console.error('Response metadata:', error.response);
    console.error('Token usage:', error.usage);

    // Handle specific error types
    if (error.cause instanceof SyntaxError) {
      console.error('JSON parsing failed');
    }
  }
}
```

**Common causes**:
- Model failed to generate a response
- Model generated unparsable JSON
- Generated JSON doesn't match schema validation

### Accessing Response Headers and Body

Access raw provider response for debugging or provider-specific features:

```typescript
const result = await generateObject({
  model: openai('gpt-4o'),
  schema: z.object({ name: z.string() }),
  prompt: 'Generate a name.',
});

// Access raw response data
console.log(JSON.stringify(result.response.headers, null, 2));
console.log(JSON.stringify(result.response.body, null, 2));
console.log('Response ID:', result.response.id);
console.log('Model:', result.response.modelId);
console.log('Timestamp:', result.response.timestamp);
```

---

## 2. streamObject Patterns

### Basic Streaming with partialObjectStream

Stream partial objects as they're generated:

```typescript
import { streamObject } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const { partialObjectStream } = streamObject({
  model: openai('gpt-4o'),
  schema: z.object({
    recipe: z.object({
      name: z.string(),
      ingredients: z.array(
        z.object({ name: z.string(), amount: z.string() })
      ),
      steps: z.array(z.string()),
    }),
  }),
  prompt: 'Generate a lasagna recipe.',
});

// Stream partial objects as they arrive
for await (const partialObject of partialObjectStream) {
  console.log(partialObject);
  // Initially: { recipe: undefined }
  // Then: { recipe: { name: 'Lasagna', ingredients: undefined, steps: undefined } }
  // Finally: { recipe: { name: 'Lasagna', ingredients: [...], steps: [...] } }
}
```

### partialObjectStream vs objectStream

**partialObjectStream**: Emits incomplete objects during generation (useful for progressive UI updates)

**objectStream**: Only emits when the complete, validated object is ready

```typescript
const result = streamObject({
  model: openai('gpt-4o'),
  schema: z.object({
    name: z.string(),
    age: z.number(),
  }),
  prompt: 'Generate a user.',
});

// Option 1: Partial objects (streaming)
for await (const partial of result.partialObjectStream) {
  console.log(partial); // { name: 'John' }, then { name: 'John', age: 25 }
}

// Option 2: Complete object only (waits until finished)
for await (const complete of result.objectStream) {
  console.log(complete); // { name: 'John', age: 25 } (only once, when done)
}
```

### Array Streaming with elementStream

Stream individual array elements as they're generated:

```typescript
const { elementStream } = streamObject({
  model: openai('gpt-4o'),
  output: 'array',
  schema: z.object({
    name: z.string(),
    class: z.string().describe('Character class, e.g. warrior, mage, or thief.'),
    description: z.string(),
  }),
  prompt: 'Generate 3 hero descriptions for a fantasy RPG.',
});

// Each hero is streamed as it completes
for await (const hero of elementStream) {
  console.log(`New hero: ${hero.name} the ${hero.class}`);
}
```

### Validation During Streaming

Partial objects are validated against the schema as much as possible:

```typescript
const { partialObjectStream } = streamObject({
  model: openai('gpt-4o'),
  schema: z.object({
    email: z.email(),
    age: z.number().min(0).max(120),
  }),
  prompt: 'Generate a user.',
});

for await (const partial of partialObjectStream) {
  // Partial validation:
  // - If email is present but invalid, validation may fail
  // - If age is present but out of range, validation fails
  // - Missing fields are allowed until stream completes
  console.log(partial);
}
```

### Error Handling with onError Callback

Streaming errors don't throw to prevent server crashes:

```typescript
const result = streamObject({
  model: openai('gpt-4o'),
  schema: z.object({ name: z.string() }),
  prompt: 'Generate a name.',
  onError({ error }) {
    // Log errors without crashing
    console.error('Stream error:', error);

    // Send to error tracking
    // sentry.captureException(error);
  },
});

for await (const partial of result.partialObjectStream) {
  console.log(partial);
}
```

---

## 3. Output API

The Output API provides a unified interface for structured data generation with `generateText` and `streamText`.

### Output.object()

Generate typed objects with schema validation:

```typescript
import { generateText, Output } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.object({
    schema: z.object({
      name: z.string(),
      age: z.number().nullable().describe('Age of the person.'),
      contact: z.object({
        type: z.literal('email'),
        value: z.string(),
      }),
      occupation: z.object({
        type: z.literal('employed'),
        company: z.string(),
        position: z.string(),
      }),
    }),
  }),
  prompt: 'Generate an example person for testing.',
});

// output is fully typed
console.log(output.name);
console.log(output.occupation.company);
```

### Output.array()

Generate arrays of structured elements:

```typescript
const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.array({
    element: z.object({
      location: z.string(),
      temperature: z.number(),
      condition: z.string(),
    }),
  }),
  prompt: 'List the weather for San Francisco and Paris.',
});

// output is an array
for (const weather of output) {
  console.log(`${weather.location}: ${weather.temperature}°F, ${weather.condition}`);
}
```

### Output.choice()

Generate enum values for classification:

```typescript
const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.choice({
    options: ['sunny', 'rainy', 'snowy'],
  }),
  prompt: 'Is the weather sunny, rainy, or snowy today?',
});

// output is one of: 'sunny' | 'rainy' | 'snowy'
console.log(output);
```

### Output.json()

Generate unstructured JSON without schema validation:

```typescript
const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.json(),
  prompt:
    'For each city, return the current temperature and weather condition as a JSON object.',
});

// output is unknown type - valid JSON but no schema
console.log(output);
// Possible output:
// {
//   "San Francisco": { "temperature": 70, "condition": "Sunny" },
//   "Paris": { "temperature": 65, "condition": "Cloudy" }
// }
```

### Output.text()

Generate plain text (default behavior):

```typescript
const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.text(),
  prompt: 'Tell me a joke.',
});

// output is a string
console.log(output);
```

### Type Inference Patterns

TypeScript automatically infers output types:

```typescript
const personSchema = z.object({
  name: z.string(),
  age: z.number(),
  hobbies: z.array(z.string()),
});

const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.object({ schema: personSchema }),
  prompt: 'Generate a person.',
});

// TypeScript knows output is: { name: string; age: number; hobbies: string[] }
const name: string = output.name;
const age: number = output.age;
const hobbies: string[] = output.hobbies;
```

### Using with streamText

Stream structured data with `partialOutputStream`:

```typescript
import { streamText, Output } from 'ai';

const { partialOutputStream } = streamText({
  model: openai('gpt-4o'),
  output: Output.object({
    schema: z.object({
      name: z.string(),
      age: z.number().nullable(),
      email: z.email(),
    }),
  }),
  prompt: 'Generate a test user.',
});

for await (const partial of partialOutputStream) {
  console.log(partial);
  // Progressive updates: { name: 'John' }
  // Then: { name: 'John', age: 25 }
  // Finally: { name: 'John', age: 25, email: 'john@example.com' }
}
```

---

## 4. Zod Schema Best Practices

### Describing Fields with .describe()

Add descriptions to guide the LLM:

```typescript
const schema = z.object({
  title: z.string().describe('The title of the article'),
  summary: z.string().describe('A brief 1-2 sentence summary'),
  tags: z.array(z.string()).describe('Relevant topic tags (3-5 tags)'),
  sentiment: z.enum(['positive', 'neutral', 'negative'])
    .describe('Overall sentiment of the article'),
  readingTime: z.number()
    .describe('Estimated reading time in minutes'),
});

const { object } = await generateObject({
  model: openai('gpt-4o'),
  schema,
  prompt: 'Analyze this article: ...',
});
```

**Best Practice**: Use `.describe()` for:
- Enum values to explain each option
- Numeric fields to specify units or ranges
- Array fields to indicate expected count
- Ambiguous property names

### String Formats (Zod v4.3.5)

Prefer top-level format helpers instead of `z.string().email()` or `z.string().url()`:

```typescript
const schema = z.object({
  email: z.email(),
  homepage: z.url(),
  createdAt: z.iso.datetime(),
  birthDate: z.iso.date(),
});
```

The `z.string().email()`-style methods still work but are deprecated in Zod v4; use the top-level helpers going forward.

### Date Handling: Transform from String

Zod expects Date objects, but LLMs return strings:

```typescript
const schema = z.object({
  events: z.array(
    z.object({
      event: z.string(),
      // Validate as ISO date string, then transform to Date
      date: z
        .iso.date()
        .transform(value => new Date(value)),
    })
  ),
});

const { object } = await generateObject({
  model: openai('gpt-4o'),
  schema,
  prompt: 'List 5 important events from the year 2000.',
});

// object.events[0].date is a Date object
console.log(object.events[0].date.getFullYear());
```

**Datetime variant**:

```typescript
const schema = z.object({
  timestamp: z
    .iso.datetime()
    .transform(value => new Date(value)),
});
```

### Optional vs Nullable: Use Nullable for Strict Schemas

Use `.nullable()` instead of `.optional()` for strict schema validation (OpenAI structured outputs):

Zod v4 removed the old optional shorthands (e.g., `z.ostring()`), so prefer `.optional()` / `.nullable()` on base types.

```typescript
// ❌ May fail with strict schema validation
const failingSchema = z.object({
  command: z.string(),
  workdir: z.string().optional(),
  timeout: z.string().optional(),
});

// ✅ Works with strict schema validation
const workingSchema = z.object({
  command: z.string(),
  workdir: z.string().nullable(),
  timeout: z.string().nullable(),
});

const { object } = await generateObject({
  model: openai('gpt-4o'),
  schema: workingSchema,
  prompt: 'Generate a command execution.',
});

// Handle nullable fields
if (object.workdir !== null) {
  console.log('Working directory:', object.workdir);
}
```

### Keep Schemas Simple: 5 or Fewer Properties Recommended

Complex schemas reduce reliability. Break them down:

```typescript
// ❌ Too complex - hard for LLM to handle reliably
const complexSchema = z.object({
  user: z.object({
    personal: z.object({
      firstName: z.string(),
      lastName: z.string(),
      middleName: z.string().optional(),
    }),
    contact: z.object({
      primary: z.object({
        type: z.enum(['email', 'phone']),
        value: z.string(),
      }),
      secondary: z.array(z.object({
        type: z.enum(['email', 'phone']),
        value: z.string(),
      })),
    }),
    preferences: z.object({
      notifications: z.boolean(),
      newsletter: z.boolean(),
      theme: z.enum(['light', 'dark']),
    }),
  }),
});

// ✅ Simpler - split into multiple calls or flatten
const simpleSchema = z.object({
  firstName: z.string(),
  lastName: z.string(),
  email: z.email(),
  phone: z.string().nullable(),
  theme: z.enum(['light', 'dark']),
});
```

**Recommended limits**:
- **5 or fewer top-level properties**
- **2 levels of nesting maximum**
- **Use arrays of simple objects, not complex nested structures**

### Semantic Naming for Better Results

Use descriptive, meaningful names:

```typescript
// ❌ Unclear names
const poorSchema = z.object({
  val: z.string(),
  num: z.number(),
  arr: z.array(z.string()),
});

// ✅ Clear, semantic names
const goodSchema = z.object({
  productName: z.string(),
  priceInCents: z.number(),
  availableSizes: z.array(z.string()),
});
```

### Temperature Settings for Structured Data

Use `temperature: 0` for deterministic structured generation:

```typescript
const { object } = await generateObject({
  model: openai('gpt-4o'),
  temperature: 0, // Recommended for structured data
  schema: z.object({
    category: z.enum(['technical', 'business', 'personal']),
    priority: z.number().min(1).max(5),
  }),
  prompt: 'Categorize this task: ...',
});
```

---

## 5. Reasoning Extraction

### Extracting Reasoning from Structured Generation

Access the model's reasoning process (OpenAI o1/o3 models):

```typescript
import { generateObject } from 'ai';
import { openai, OpenAIResponsesProviderOptions } from '@ai-sdk/openai';
import { z } from 'zod';

const result = await generateObject({
  model: openai('gpt-5'),
  schema: z.object({
    recipe: z.object({
      name: z.string(),
      ingredients: z.array(
        z.object({
          name: z.string(),
          amount: z.string(),
        })
      ),
      steps: z.array(z.string()),
    }),
  }),
  prompt: 'Generate a lasagna recipe.',
  providerOptions: {
    openai: {
      strictJsonSchema: true,
      reasoningSummary: 'detailed',
    } satisfies OpenAIResponsesProviderOptions,
  },
});

// Access reasoning if available
console.log('Model reasoning:', result.reasoning);
```

### experimental_repairText for JSON Repair

Attempt to repair invalid or malformed JSON:

```typescript
import { generateObject } from 'ai';
import { z } from 'zod';

const { object } = await generateObject({
  model: openai('gpt-4o'),
  schema: z.object({
    name: z.string(),
    items: z.array(z.string()),
  }),
  prompt: 'Generate a shopping list.',
  experimental_repairText: async ({ text, error }) => {
    console.log('Attempting to repair JSON:', error);

    // Example: Add missing closing brace
    if (error.message.includes('Unexpected end')) {
      return text + '}';
    }

    // Example: Call another LLM to repair
    const { output } = await generateText({
      model: openai('gpt-4o'),
      output: Output.text(),
      prompt: `Repair this malformed JSON: ${text}`,
    });

    return output;
  },
});
```

**Error types handled**:
- `JSONParseError`: Invalid JSON syntax
- `TypeValidationError`: JSON is valid but doesn't match schema

---

## Complete Example: Recipe Generator

Combining patterns for a production-ready recipe generator:

```typescript
import { generateObject, NoObjectGeneratedError } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const recipeSchema = z.object({
  name: z.string().describe('Name of the recipe'),
  servings: z.number().min(1).describe('Number of servings'),
  prepTime: z.number().describe('Preparation time in minutes'),
  cookTime: z.number().describe('Cooking time in minutes'),
  difficulty: z.enum(['easy', 'medium', 'hard'])
    .describe('Recipe difficulty level'),
  ingredients: z.array(
    z.object({
      name: z.string(),
      amount: z.string(),
      unit: z.string().nullable(),
    })
  ).describe('List of ingredients with amounts'),
  steps: z.array(z.string())
    .describe('Step-by-step cooking instructions'),
  tags: z.array(z.string())
    .describe('Cuisine tags (e.g., italian, vegetarian)'),
});

async function generateRecipe(dishName: string) {
  try {
    const result = await generateObject({
      model: openai('gpt-4o'),
      temperature: 0,
      schemaName: 'Recipe',
      schemaDescription: 'A complete recipe with ingredients and instructions',
      schema: recipeSchema,
      prompt: `Generate a detailed recipe for ${dishName}.`,
      experimental_repairText: async ({ text, error }) => {
        console.warn('Repairing malformed JSON:', error.message);
        return text + '}';
      },
    });

    return result.object;
  } catch (error) {
    if (NoObjectGeneratedError.isInstance(error)) {
      console.error('Failed to generate recipe:', {
        cause: error.cause,
        text: error.text,
        usage: error.usage,
      });
      throw new Error('Recipe generation failed');
    }
    throw error;
  }
}

// Usage
const recipe = await generateRecipe('chocolate chip cookies');
console.log(`${recipe.name} (${recipe.difficulty})`);
console.log(`Prep: ${recipe.prepTime}min, Cook: ${recipe.cookTime}min`);
```

---

## Key Takeaways

1. **Use `generateObject` for one-shot structured generation**, `streamObject` for progressive UI updates
2. **Schema modes**: `object` (default), `array`, `enum`, `no-schema`
3. **Output API**: Unified interface for `generateText`/`streamText` with structured outputs
4. **Zod best practices**:
   - Add `.describe()` to guide the LLM
   - Transform dates from strings
   - Use `.nullable()` for strict schemas
   - Keep schemas simple (≤5 properties, ≤2 nesting levels)
5. **Error handling**: Catch `NoObjectGeneratedError`, use `onError` callback for streams
6. **Set `temperature: 0`** for deterministic structured generation
7. **Use `experimental_repairText`** to handle malformed JSON gracefully
