# AI SDK v6 Migration Guide

Comprehensive guide for migrating from AI SDK v5 to v6 stable.

## Package Versions

```bash
pnpm add ai@^6.0.3 @ai-sdk/openai@^3.0.1 @ai-sdk/anthropic@^3.0.1 @ai-sdk/google@^3.0.1 @ai-sdk/react@^3.0.3
```

---

## Automated Migration

### Step 1: Run Codemod

```bash
npx @ai-sdk/codemod v6
```

The codemod automatically handles:
- `textEmbeddingModel()` → `embeddingModel()`
- `MockLanguageModelV2` → `MockLanguageModel`
- `MockEmbeddingModelV2` → `MockEmbeddingModel`
- `ToolCallOptions` → `ToolExecutionOptions`
- `CoreMessage` → `ModelMessage`
- `convertToCoreMessages` → `convertToModelMessages`
- `LanguageModelV2Middleware` → `LanguageModelMiddleware`
- `LanguageModelV2StreamPart` → `LanguageModelStreamPart`

### Step 2: Update Packages

```bash
pnpm add ai@^6.0.3 @ai-sdk/openai@^3.0.1 @ai-sdk/anthropic@^3.0.1 @ai-sdk/google@^3.0.1 @ai-sdk/react@^3.0.3
```

### Step 3: Manual Fixes

#### Async convertToModelMessages

The function is now async and must be awaited:

```typescript
// Before (v5)
const messages = convertToCoreMessages(uiMessages);

// After (v6)
const messages = await convertToModelMessages(uiMessages);
```

#### Output API (Recommended for Structured Data)

The Output API provides a unified interface for structured outputs with `generateText`/`streamText`:

```typescript
// Before (deprecated but still works)
import { generateObject } from 'ai';
import { z } from 'zod';

const { object } = await generateObject({
  model: openai('gpt-4o'),
  schema: z.object({ name: z.string() }),
  prompt: 'Generate a name',
});

// After (v6 - preferred)
import { generateText, Output } from 'ai';
import { z } from 'zod';

const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.object({
    schema: z.object({ name: z.string() }),
  }),
  prompt: 'Generate a name',
});
```

**Output API variants:**
- `Output.object({ schema })` - Single typed object
- `Output.array({ element })` - Array of objects
- `Output.choice({ options })` - Enum classification
- `Output.json()` - Untyped JSON
- `Output.text()` - Plain text (default)

#### Embedding Model Method

```typescript
// Before (v5)
const model = openai.textEmbeddingModel('text-embedding-3-small');
// or
const model = openai.embeddingModel('text-embedding-3-small');

// After (v6)
const model = openai.embedding('text-embedding-3-small');
```

#### MCP Client (Now Stable)

```typescript
// Before (v5)
import { experimental_createMCPClient as createMCPClient } from '@ai-sdk/mcp';

// After (v6)
import { createMCPClient } from '@ai-sdk/mcp';
```

### Step 4: Type Check

```bash
pnpm type-check
```

---

## Breaking Changes Reference

| Before (v5) | After (v6) |
|-------------|------------|
| `CoreMessage` | `ModelMessage` |
| `convertToCoreMessages` | `await convertToModelMessages` |
| `ToolCallOptions` | `ToolExecutionOptions` |
| `textEmbeddingModel()` | `embedding()` |
| `embeddingModel()` | `embedding()` |
| `MockLanguageModelV2` | `MockLanguageModel` |
| `MockEmbeddingModelV2` | `MockEmbeddingModel` |
| `LanguageModelV2Middleware` | `LanguageModelMiddleware` |
| `LanguageModelV2StreamPart` | `LanguageModelStreamPart` |
| Finish reason `'unknown'` | `'other'` |
| `experimental_createMCPClient` | `createMCPClient` |

---

## New Features in v6

### Output API

Unified structured output with text generation:

```typescript
import { generateText, Output } from 'ai';
import { z } from 'zod';

// Object output
const { output } = await generateText({
  model: openai('gpt-4o'),
  output: Output.object({
    schema: z.object({
      name: z.string(),
      age: z.number(),
    }),
  }),
  prompt: 'Generate a person',
});

// Array output
const { output: people } = await generateText({
  model: openai('gpt-4o'),
  output: Output.array({
    element: z.object({ name: z.string() }),
  }),
  prompt: 'Generate 3 names',
});

// Choice output (enum)
const { output: category } = await generateText({
  model: openai('gpt-4o'),
  output: Output.choice({
    options: ['tech', 'business', 'sports'],
  }),
  prompt: 'Classify this article',
});
```

### Tool Enhancements

#### needsApproval

Request human approval before tool execution:

```typescript
import { tool } from 'ai';
import { z } from 'zod';

const deleteTool = tool({
  description: 'Delete a file',
  inputSchema: z.object({ path: z.string() }),
  needsApproval: true, // Requires approval before execution
  execute: async ({ path }) => {
    await deleteFile(path);
    return { success: true };
  },
});
```

#### inputExamples

Provide example inputs for better model guidance:

```typescript
const searchTool = tool({
  description: 'Search for information',
  inputSchema: z.object({
    query: z.string(),
    filters: z.object({
      date: z.string().optional(),
    }).optional(),
  }),
  inputExamples: [
    { input: { query: 'AI news', filters: { date: '2024-01-15' } } },
    { input: { query: 'weather forecast' } },
  ],
  execute: async ({ query, filters }) => {
    // Implementation
  },
});
```

#### toModelOutput

Transform tool results for model consumption:

```typescript
const apiTool = tool({
  description: 'Call an API',
  inputSchema: z.object({ endpoint: z.string() }),
  execute: async ({ endpoint }) => {
    const response = await fetch(endpoint);
    return await response.json();
  },
  toModelOutput: (result) => {
    // Return simplified version for model context
    return JSON.stringify(result, null, 2).slice(0, 1000);
  },
});
```

### DevTools

Visual debugging and inspection:

```typescript
import { devtools } from '@ai-sdk/devtools';

// Wrap your model for debugging
const debugModel = devtools(openai('gpt-4o'));

const { text } = await generateText({
  model: debugModel,
  prompt: 'Hello',
});
```

### Reranking with Bedrock

AWS Bedrock reranking support:

```typescript
import { rerank } from 'ai';
import { bedrock } from '@ai-sdk/amazon-bedrock';

const { ranking, rerankedDocuments } = await rerank({
  model: bedrock.reranking('amazon.rerank-v1:0'),
  documents: ['doc1', 'doc2', 'doc3'],
  query: 'search query',
  topN: 2,
});
```

---

## Provider Updates

### OpenAI

- **Responses API** is now default (use `openai.chat()` for Chat API)
- **o3/o4 models** with `reasoningEffort` and `reasoningSummary`
- **strictJsonSchema** defaults to `true`
- **Built-in tools**: `webSearch`, `fileSearch`, `imageGeneration`, `codeInterpreter`, `mcp`
- **Prompt caching** with `promptCacheKey` and `promptCacheRetention`

```typescript
import { openai } from '@ai-sdk/openai';
import { generateText } from 'ai';

const { text, reasoning } = await generateText({
  model: openai('gpt-5'),
  prompt: 'Complex reasoning task',
  providerOptions: {
    openai: {
      reasoningEffort: 'high',
      reasoningSummary: 'detailed',
    },
  },
});
```

### Anthropic

- **Claude 4 models**: `claude-opus-4`, `claude-sonnet-4`, `claude-haiku-4`
- **Agent skills**: `pptx`, `docx`, `xlsx`, `pdf`
- **structuredOutputMode** option
- **MCP connectors** for server integration
- **Web search and code execution** tools

```typescript
import { anthropic } from '@ai-sdk/anthropic';
import { generateText } from 'ai';

const { text, reasoning } = await generateText({
  model: anthropic('claude-opus-4-20250514'),
  prompt: 'Complex task',
  providerOptions: {
    anthropic: {
      thinking: { type: 'enabled', budgetTokens: 12000 },
    },
  },
});
```

### Google

- **Gemini 3 models** with `thinkingLevel` configuration
- **Gemini 2.5** with `thinkingBudget`
- **Built-in tools**: `googleSearch`, `codeExecution`, `fileSearch`, `urlContext`
- **Implicit caching** with 75% discount

```typescript
import { google } from '@ai-sdk/google';
import { generateText } from 'ai';

const { text, reasoning } = await generateText({
  model: google('gemini-3-pro-preview'),
  prompt: 'Math problem',
  providerOptions: {
    google: {
      thinkingConfig: {
        thinkingLevel: 'high',
        includeThoughts: true,
      },
    },
  },
});
```

### AI Gateway

- **OIDC authentication** for Vercel deployments
- **BYOK** (Bring Your Own Key) support
- **Dynamic model discovery** with `gateway.getAvailableModels()`
- **Credit tracking** with `gateway.getCredits()`

```typescript
import { gateway } from 'ai';

// OIDC auth is automatic on Vercel
const { text } = await generateText({
  model: 'openai/gpt-5',
  prompt: 'Hello',
});

// Model discovery
const models = await gateway.getAvailableModels();
```

---

## Testing Migration

Update test mocks:

```typescript
// Before (v5)
import { MockLanguageModelV2 } from 'ai/test';

const mockModel = new MockLanguageModelV2({ /* ... */ });

// After (v6)
import { MockLanguageModel } from 'ai/test';

const mockModel = new MockLanguageModel({ /* ... */ });
```

---

## Troubleshooting

### "convertToCoreMessages is not a function"

The function was renamed. Update to:

```typescript
import { convertToModelMessages } from 'ai';
const messages = await convertToModelMessages(uiMessages);
```

### "Property 'textEmbeddingModel' does not exist"

Use the new method name:

```typescript
const model = openai.embedding('text-embedding-3-small');
```

### "MockLanguageModelV2 is not exported"

Update the import:

```typescript
import { MockLanguageModel } from 'ai/test';
```

### Finish reason 'unknown' not in union

Update type checks:

```typescript
// Before
if (result.finishReason === 'unknown') { /* ... */ }

// After
if (result.finishReason === 'other') { /* ... */ }
```
