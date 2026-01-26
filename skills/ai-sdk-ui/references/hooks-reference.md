# Hooks Reference

useObject and useCompletion hooks for specialized streaming.

## useObject

Stream structured JSON data with type safety.

### Basic Usage

```typescript
'use client';
import { useObject } from '@ai-sdk/react';
import { z } from 'zod';

const recipeSchema = z.object({
  name: z.string(),
  ingredients: z.array(z.object({
    name: z.string(),
    amount: z.string(),
  })),
  steps: z.array(z.string()),
});

export default function RecipeGenerator() {
  const { object, submit, isLoading, error } = useObject({
    api: '/api/generate-recipe',
    schema: recipeSchema,
  });

  return (
    <div>
      <button onClick={() => submit({ prompt: 'chocolate chip cookies' })}>
        Generate Recipe
      </button>

      {isLoading && <div>Generating...</div>}
      {error && <div>Error: {error.message}</div>}

      {/* Render partial data as it streams */}
      {object && (
        <div>
          <h2>{object.name ?? 'Loading name...'}</h2>
          {object.ingredients?.map((ing, i) => (
            <div key={i}>{ing.name}: {ing.amount}</div>
          ))}
          <ol>
            {object.steps?.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>
        </div>
      )}
    </div>
  );
}
```

### API Route

```typescript
// app/api/generate-recipe/route.ts
import { streamObject } from 'ai';
import { z } from 'zod';

export async function POST(request: Request) {
  const { prompt } = await request.json();

  const result = streamObject({
    model: 'openai/gpt-4o',
    schema: z.object({
      name: z.string(),
      ingredients: z.array(z.object({
        name: z.string(),
        amount: z.string(),
      })),
      steps: z.array(z.string()),
    }),
    prompt: `Generate a recipe for: ${prompt}`,
  });

  return result.toTextStreamResponse();
}
```

### Partial Rendering

Object fields stream progressively:

```typescript
// object updates as data streams:
// { name: undefined, ingredients: undefined, steps: undefined }
// { name: "Chocolate", ingredients: undefined, steps: undefined }
// { name: "Chocolate Chip Cookies", ingredients: [{ name: "flour" }], ... }
// { name: "Chocolate Chip Cookies", ingredients: [...], steps: [...] }
```

### With Custom Output

```typescript
const { object, submit } = useObject({
  api: '/api/generate',
  schema: mySchema,
  onFinish: ({ object }) => {
    // Final complete object
    console.log('Generated:', object);
  },
  onError: (error) => {
    console.error('Generation failed:', error);
  },
});
```

## useCompletion

Stream text completions (non-chat).

### Basic Usage

```typescript
'use client';
import { useCompletion } from '@ai-sdk/react';

export default function TextCompletion() {
  const {
    completion,   // Streamed text
    input,        // Input state
    handleInputChange,
    handleSubmit,
    isLoading,
    error,
  } = useCompletion({
    api: '/api/completion',
  });

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <input
          value={input}
          onChange={handleInputChange}
          placeholder="Start typing..."
        />
        <button type="submit" disabled={isLoading}>
          Complete
        </button>
      </form>

      {completion && (
        <div className="whitespace-pre-wrap">{completion}</div>
      )}
    </div>
  );
}
```

### API Route

```typescript
// app/api/completion/route.ts
import { streamText } from 'ai';

export async function POST(request: Request) {
  const { prompt } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    prompt: `Complete this text: ${prompt}`,
  });

  return result.toTextStreamResponse();
}
```

### With Body and Headers

```typescript
const { completion, handleSubmit } = useCompletion({
  api: '/api/completion',
  body: {
    model: 'gpt-4o',
    temperature: 0.7,
  },
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});
```

## Shared Patterns

### Cancellation

```typescript
const { stop, isLoading } = useObject({ /* ... */ });
// or
const { stop, isLoading } = useCompletion({ /* ... */ });

// Cancel ongoing stream
<button onClick={stop} disabled={!isLoading}>
  Stop
</button>
```

### Callbacks

```typescript
useObject({
  api: '/api/generate',
  schema: mySchema,
  onFinish: ({ object }) => {
    // Called when streaming completes
    saveToDatabase(object);
  },
  onError: (error) => {
    // Handle errors
    toast.error(error.message);
  },
});
```

### Throttling

Control UI update frequency:

```typescript
useObject({
  api: '/api/generate',
  schema: mySchema,
  // Note: Still experimental in v6
  experimental_throttle: 100, // Update UI every 100ms max
});
```

## When to Use

| Hook | Use Case |
|------|----------|
| useChat | Multi-turn conversation |
| useObject | Structured data generation |
| useCompletion | Single text completion |

## useObject vs useChat

```typescript
// useObject: Single structured output
const { object } = useObject({
  schema: productSchema,
});

// useChat: Conversation with optional tools
const { messages } = useChat({
  tools: { /* ... */ },
});
```

## Type Safety

```typescript
import { z } from 'zod';

const schema = z.object({
  title: z.string(),
  items: z.array(z.string()),
});

type GeneratedData = z.infer<typeof schema>;

const { object } = useObject<GeneratedData>({
  api: '/api/generate',
  schema,
});

// object is typed as Partial<GeneratedData> during streaming
// TypeScript knows object.title is string | undefined
```
