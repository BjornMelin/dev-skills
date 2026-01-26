# AI SDK Core: Language Model Middleware Reference

Language model middleware intercepts and modifies calls to language models, enabling features like guardrails, RAG, caching, and logging in a model-agnostic way. Middleware can be developed and distributed independently from the models they enhance.

## 1. Middleware Architecture

### wrapLanguageModel()

Wraps a language model with one or more middleware functions:

```typescript
import { wrapLanguageModel } from 'ai';

const wrappedModel = wrapLanguageModel({
  model: yourModel,
  middleware: yourMiddleware,
});

// Use wrapped model like any other model
const result = await streamText({
  model: wrappedModel,
  prompt: 'What cities are in the United States?',
});
```

### Core Middleware Interface

Middleware implements one or more of these methods:

```typescript
import type { LanguageModelMiddleware } from 'ai';

const middleware: LanguageModelMiddleware = {
  // Pre-process parameters before doGenerate/doStream
  transformParams: async ({ params }) => {
    // Modify params.prompt, params.temperature, etc.
    return modifiedParams;
  },

  // Wrap doGenerate (non-streaming)
  wrapGenerate: async ({ doGenerate, params }) => {
    // Pre-process
    const result = await doGenerate();
    // Post-process result.text, result.toolCalls, etc.
    return result;
  },

  // Wrap doStream (streaming)
  wrapStream: async ({ doStream, params }) => {
    const { stream, ...rest } = await doStream();
    // Transform stream chunks
    return { stream: transformedStream, ...rest };
  },
};
```

### transformParams: Pre-processing

Modifies parameters before they reach the model (applies to both generate and stream):

```typescript
const ragMiddleware: LanguageModelMiddleware = {
  transformParams: async ({ params }) => {
    const lastUserMessage = getLastUserMessageText({ prompt: params.prompt });

    if (!lastUserMessage) return params;

    const context = await findSources({ text: lastUserMessage });
    const instruction =
      'Use the following information to answer:\n' +
      context.map(chunk => JSON.stringify(chunk)).join('\n');

    return addToLastUserMessage({ params, text: instruction });
  },
};
```

### wrapGenerate: Request/Response Modification

Intercepts non-streaming `doGenerate` calls:

```typescript
const guardrailMiddleware: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate, params }) => {
    console.log('Request:', params.prompt);

    const { text, ...rest } = await doGenerate();

    // Filter sensitive information
    const cleanedText = text?.replace(/\b\d{3}-\d{2}-\d{4}\b/g, '[SSN-REDACTED]');

    console.log('Response:', cleanedText);
    return { text: cleanedText, ...rest };
  },
};
```

### wrapStream: Streaming Transformation

Intercepts streaming `doStream` calls with `TransformStream`:

```typescript
import type { LanguageModelStreamPart } from 'ai';

const streamLogMiddleware: LanguageModelMiddleware = {
  wrapStream: async ({ doStream, params }) => {
    const { stream, ...rest } = await doStream();

    let generatedText = '';
    const textBlocks = new Map<string, string>();

    const transformStream = new TransformStream<
      LanguageModelStreamPart,
      LanguageModelStreamPart
    >({
      transform(chunk, controller) {
        switch (chunk.type) {
          case 'text-start':
            textBlocks.set(chunk.id, '');
            break;
          case 'text-delta':
            const existing = textBlocks.get(chunk.id) || '';
            textBlocks.set(chunk.id, existing + chunk.delta);
            generatedText += chunk.delta;
            break;
          case 'text-end':
            console.log(`Block ${chunk.id}:`, textBlocks.get(chunk.id));
            break;
        }
        controller.enqueue(chunk);
      },

      flush() {
        console.log('Total generated:', generatedText);
      },
    });

    return { stream: stream.pipeThrough(transformStream), ...rest };
  },
};
```

## 2. Built-in Middleware

### extractReasoningMiddleware

Extracts reasoning from special tags (e.g., `<think>...</think>` for DeepSeek R1, Claude extended thinking) and exposes as `reasoning` property:

```typescript
import { wrapLanguageModel, extractReasoningMiddleware } from 'ai';

const model = wrapLanguageModel({
  model: yourModel,
  middleware: extractReasoningMiddleware({ tagName: 'think' }),
});

const result = await generateText({
  model,
  prompt: 'Solve this complex problem...',
});

// Access extracted reasoning
console.log(result.reasoning); // Content inside <think> tags
console.log(result.text);      // Final answer (tags removed)
```

**Options:**
- `tagName` (required): Tag name to extract (e.g., 'think', 'reasoning')
- `startWithReasoning` (optional): Prepend reasoning tag to response if model doesn't include it

```typescript
// For models that don't start with reasoning tag
const deepseekModel = wrapLanguageModel({
  model: deepseek('deepseek-reasoner'),
  middleware: extractReasoningMiddleware({
    tagName: 'think',
    startWithReasoning: true, // Adds <think> at start
  }),
});
```

### simulateStreamingMiddleware

Converts non-streaming responses to streaming format for consistent interface:

```typescript
import { wrapLanguageModel, simulateStreamingMiddleware } from 'ai';

const model = wrapLanguageModel({
  model: nonStreamingModel,
  middleware: simulateStreamingMiddleware(),
});

// Now works with streamText even if model doesn't support streaming
for await (const chunk of streamText({ model, prompt: '...' })) {
  console.log(chunk);
}
```

### defaultSettingsMiddleware

Applies default configuration to all model calls:

```typescript
import { wrapLanguageModel, defaultSettingsMiddleware } from 'ai';

const model = wrapLanguageModel({
  model: yourModel,
  middleware: defaultSettingsMiddleware({
    settings: {
      temperature: 0.5,
      maxOutputTokens: 800,
      topP: 0.9,
      frequencyPenalty: 0.5,
      providerOptions: {
        openai: { store: false },
      },
    },
  }),
});

// All calls inherit these defaults (can be overridden per-call)
const result = await generateText({
  model,
  prompt: '...',
  // temperature: 0.8, // Overrides default
});
```

### addToolInputExamplesMiddleware

Serializes `inputExamples` into tool descriptions for providers that don't support them natively:

```typescript
import { wrapLanguageModel, addToolInputExamplesMiddleware, tool } from 'ai';
import { z } from 'zod';

const model = wrapLanguageModel({
  model: yourModel,
  middleware: addToolInputExamplesMiddleware({
    examplesPrefix: 'Input Examples:',
  }),
});

const result = await generateText({
  model,
  tools: {
    weather: tool({
      description: 'Get the weather in a location',
      inputSchema: z.object({
        location: z.string(),
      }),
      inputExamples: [
        { input: { location: 'San Francisco' } },
        { input: { location: 'London' } },
      ],
    }),
  },
  prompt: 'What is the weather in Tokyo?',
});
```

**Transformed description:**
```
Get the weather in a location

Input Examples:
{"location":"San Francisco"}
{"location":"London"}
```

**Options:**
```typescript
addToolInputExamplesMiddleware({
  examplesPrefix: 'Examples:',
  formatExample: (example, index) => `${index + 1}. ${JSON.stringify(example.input)}`,
  removeInputExamples: true, // Remove from tool after adding to description
})
```

## 3. Custom Middleware Patterns

### Logging Middleware

Track requests, responses, and performance:

```typescript
const loggingMiddleware: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate, params }) => {
    const start = Date.now();
    console.log('Generate request:', {
      model: params.modelId,
      prompt: params.prompt,
      temperature: params.temperature,
    });

    const result = await doGenerate();
    const duration = Date.now() - start;

    console.log('Generate response:', {
      text: result.text,
      usage: result.usage,
      duration,
    });

    return result;
  },

  wrapStream: async ({ doStream, params }) => {
    console.log('Stream request:', params);
    const { stream, ...rest } = await doStream();

    let generatedText = '';
    const transformStream = new TransformStream({
      transform(chunk, controller) {
        if (chunk.type === 'text-delta') {
          generatedText += chunk.delta;
        }
        controller.enqueue(chunk);
      },
      flush() {
        console.log('Stream complete:', generatedText);
      },
    });

    return { stream: stream.pipeThrough(transformStream), ...rest };
  },
};
```

### Caching Middleware

Simple in-memory cache for identical requests:

```typescript
const cache = new Map<string, any>();

const cachingMiddleware: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate, params }) => {
    const cacheKey = JSON.stringify(params);

    if (cache.has(cacheKey)) {
      console.log('Cache hit');
      return cache.get(cacheKey);
    }

    const result = await doGenerate();
    cache.set(cacheKey, result);
    return result;
  },

  // Streaming cache implementation (store full response)
  wrapStream: async ({ doStream, params }) => {
    const cacheKey = JSON.stringify(params);

    if (cache.has(cacheKey)) {
      // Return cached stream
      const cachedResponse = cache.get(cacheKey);
      return {
        stream: new ReadableStream({
          start(controller) {
            for (const chunk of cachedResponse.chunks) {
              controller.enqueue(chunk);
            }
            controller.close();
          },
        }),
        ...cachedResponse.metadata,
      };
    }

    const { stream, ...rest } = await doStream();
    const chunks: any[] = [];

    const transformStream = new TransformStream({
      transform(chunk, controller) {
        chunks.push(chunk);
        controller.enqueue(chunk);
      },
      flush() {
        cache.set(cacheKey, { chunks, metadata: rest });
      },
    });

    return { stream: stream.pipeThrough(transformStream), ...rest };
  },
};
```

### RAG Context Injection

Add retrieved context to user messages:

```typescript
const ragMiddleware: LanguageModelMiddleware = {
  transformParams: async ({ params }) => {
    const lastUserMessageText = getLastUserMessageText({
      prompt: params.prompt,
    });

    if (!lastUserMessageText) {
      return params; // No user message to augment
    }

    // Retrieve relevant documents (pseudo-code)
    const sources = await findSources({ text: lastUserMessageText });

    const instruction =
      'Use the following information to answer the question:\n' +
      sources.map(chunk => JSON.stringify(chunk)).join('\n');

    return addToLastUserMessage({ params, text: instruction });
  },
};

// Helper functions (not part of AI SDK)
function getLastUserMessageText({ prompt }) {
  if (typeof prompt === 'string') return prompt;
  const userMessages = prompt.filter(m => m.role === 'user');
  return userMessages[userMessages.length - 1]?.content;
}

function addToLastUserMessage({ params, text }) {
  if (typeof params.prompt === 'string') {
    return { ...params, prompt: `${text}\n\n${params.prompt}` };
  }
  const messages = [...params.prompt];
  const lastUserIndex = messages.findLastIndex(m => m.role === 'user');
  messages[lastUserIndex] = {
    ...messages[lastUserIndex],
    content: `${text}\n\n${messages[lastUserIndex].content}`,
  };
  return { ...params, prompt: messages };
}
```

### Guardrails Middleware

Filter or validate generated content:

```typescript
const guardrailMiddleware: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate }) => {
    const { text, ...rest } = await doGenerate();

    // PII filtering
    const cleanedText = text
      ?.replace(/\b\d{3}-\d{2}-\d{4}\b/g, '[SSN-REDACTED]')
      ?.replace(/\b[\w.%+-]+@[\w.-]+\.[A-Z]{2,}\b/gi, '[EMAIL-REDACTED]');

    return { text: cleanedText, ...rest };
  },

  // Note: Streaming guardrails are difficult because you don't know
  // the full content until the stream finishes
  wrapStream: async ({ doStream }) => {
    const { stream, ...rest } = await doStream();

    let fullText = '';
    const transformStream = new TransformStream({
      transform(chunk, controller) {
        if (chunk.type === 'text-delta') {
          fullText += chunk.delta;
        }
        controller.enqueue(chunk);
      },
      flush(controller) {
        // Can only validate after stream completes
        if (containsProhibitedContent(fullText)) {
          controller.error(new Error('Content policy violation'));
        }
      },
    });

    return { stream: stream.pipeThrough(transformStream), ...rest };
  },
};
```

## 4. Composition

### Chaining Multiple Middleware

Apply multiple middleware in sequence:

```typescript
const wrappedModel = wrapLanguageModel({
  model: yourModel,
  middleware: [ragMiddleware, loggingMiddleware, cachingMiddleware],
});

// Applied as: ragMiddleware(loggingMiddleware(cachingMiddleware(yourModel)))
// Execution order:
// 1. RAG adds context
// 2. Logging wraps the call
// 3. Caching checks/stores result
```

### Order of Execution

Middleware executes in reverse order (last-to-first for wrapping):

```typescript
const middleware1: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate }) => {
    console.log('Middleware 1: before');
    const result = await doGenerate();
    console.log('Middleware 1: after');
    return result;
  },
};

const middleware2: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate }) => {
    console.log('Middleware 2: before');
    const result = await doGenerate();
    console.log('Middleware 2: after');
    return result;
  },
};

wrapLanguageModel({ model, middleware: [middleware1, middleware2] });

// Output:
// Middleware 1: before
// Middleware 2: before
// [actual model call]
// Middleware 2: after
// Middleware 1: after
```

### Middleware + Custom Providers

Middleware works with both official and custom providers:

```typescript
import { createOpenAI } from '@ai-sdk/openai';

const customOpenAI = createOpenAI({
  baseURL: 'https://custom-endpoint.example.com/v1',
  apiKey: process.env.CUSTOM_API_KEY,
});

const wrappedModel = wrapLanguageModel({
  model: customOpenAI('gpt-4'),
  middleware: [loggingMiddleware, cachingMiddleware],
});
```

### Custom Metadata in Middleware

Pass request-specific context via `providerOptions`:

```typescript
const metadataLoggingMiddleware: LanguageModelMiddleware = {
  wrapGenerate: async ({ doGenerate, params }) => {
    const metadata = params?.providerMetadata?.yourLogMiddleware;
    console.log('User ID:', metadata?.userId);
    console.log('Session:', metadata?.sessionId);

    const result = await doGenerate();
    return result;
  },
};

const { text } = await generateText({
  model: wrapLanguageModel({
    model: yourModel,
    middleware: metadataLoggingMiddleware,
  }),
  prompt: 'Invent a new holiday',
  providerOptions: {
    yourLogMiddleware: {
      userId: '12345',
      sessionId: 'abc-def',
      timestamp: Date.now(),
    },
  },
});
```

## 5. Embedding Middleware

### wrapEmbeddingModel()

Similar pattern for embedding models (inferred from language model pattern):

```typescript
import type { EmbeddingModelV1Middleware } from '@ai-sdk/provider';

const embeddingMiddleware: EmbeddingModelV1Middleware = {
  transformParams: async ({ params }) => {
    // Pre-process embedding parameters
    return params;
  },

  wrapEmbed: async ({ doEmbed, params }) => {
    const result = await doEmbed();
    // Post-process embeddings
    return result;
  },
};

const wrappedEmbedding = wrapEmbeddingModel({
  model: yourEmbeddingModel,
  middleware: embeddingMiddleware,
});
```

### Custom Embedding Transformations

Normalize or transform embedding vectors:

```typescript
const normalizeEmbeddingMiddleware: EmbeddingModelV1Middleware = {
  wrapEmbed: async ({ doEmbed }) => {
    const result = await doEmbed();

    // Normalize embeddings to unit vectors
    const normalizedEmbeddings = result.embeddings.map(embedding => {
      const magnitude = Math.sqrt(
        embedding.reduce((sum, val) => sum + val * val, 0)
      );
      return embedding.map(val => val / magnitude);
    });

    return { ...result, embeddings: normalizedEmbeddings };
  },
};
```

## Community Middleware

### Custom Tool Call Parser

[@ai-sdk-tool/parser](https://github.com/minpeter/ai-sdk-tool-call-middleware) enables function calling for models without native support:

```typescript
import { wrapLanguageModel } from 'ai';
import { gemmaToolMiddleware } from '@ai-sdk-tool/parser';

const model = wrapLanguageModel({
  model: openrouter('google/gemma-3-27b-it'),
  middleware: gemmaToolMiddleware,
});

// Now Gemma supports tool calls
const result = await generateText({
  model,
  tools: {
    weather: tool({ /* ... */ }),
  },
  prompt: 'What is the weather in SF?',
});
```

Available variants:
- `createToolMiddleware`: Custom tool call formats
- `hermesToolMiddleware`: Hermes/Qwen format
- `gemmaToolMiddleware`: Gemma 3 format

## Best Practices

1. **Keep middleware focused**: Each middleware should have a single responsibility
2. **Handle both generate and stream**: Implement both unless your use case is specific
3. **Preserve original behavior**: Don't break the model contract; add, don't replace
4. **Use TransformStream for streaming**: Native stream transformation prevents buffering
5. **Cache carefully**: Consider memory limits and cache invalidation
6. **Log to stderr**: stdout is reserved for MCP JSON-RPC in server contexts
7. **Test middleware independently**: Unit test each middleware before composition
8. **Document side effects**: Make it clear if middleware performs I/O or mutations

## References

- [AI SDK Middleware Docs](https://ai-sdk.dev/docs/ai-sdk-core/middleware)
- [DeepSeek R1 Guide](https://ai-sdk.dev/docs/guides/r1#deepseek-r1-middleware)
- [Community Middleware Examples](https://github.com/minpeter/ai-sdk-tool-call-middleware/tree/main/examples/core/src)
