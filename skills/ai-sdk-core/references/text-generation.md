# AI SDK Core: Text Generation Reference

Comprehensive reference for text generation patterns using `generateText` and `streamText`.

## Table of Contents

1. [generateText Patterns](#generatetext-patterns)
2. [streamText Patterns](#streamtext-patterns)
3. [Sources](#sources)
4. [Settings Reference](#settings-reference)

---

## generateText Patterns

### Basic Usage

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

// Simple prompt
const { text } = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a vegetarian lasagna recipe for 4 people.',
});

// With system message
const { text } = await generateText({
  model: openai('gpt-4'),
  system: 'You are a professional writer. You write simple, clear, and concise content.',
  prompt: `Summarize the following article in 3-5 sentences: ${article}`,
});
```

### Result Object Properties

The result object contains several properties and promises that resolve when data is available:

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Explain quantum computing',
});

// Text content
result.text;              // Generated text (string)
result.content;           // Content from last step (array)
result.files;             // Files generated in last step

// Reasoning (model-specific)
result.reasoning;         // Full reasoning from last step
result.reasoningText;     // Reasoning text (only some models)

// Tool information
result.toolCalls;         // Tool calls from last step
result.toolResults;       // Tool results from last step

// Completion metadata
result.finishReason;      // Why model stopped ('stop' | 'length' | 'content-filter' | 'tool-calls' | 'error' | 'other')
result.usage;             // Token usage for final step
result.totalUsage;        // Total usage across all steps (multi-step)

// Response information
result.warnings;          // Provider warnings (e.g., unsupported settings)
result.request;           // Request metadata
result.response;          // Response metadata (headers, body, messages)
result.providerMetadata;  // Provider-specific metadata

// Step details
result.steps;             // Details for all steps (useful for multi-step)

// Structured output
result.output;            // Structured output via `output` specification
```

### Result Usage Example

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a haiku about coding',
  maxOutputTokens: 100,
});

console.log('Generated text:', result.text);
console.log('Finish reason:', result.finishReason);
console.log('Tokens used:', result.usage.totalTokens);
console.log('Prompt tokens:', result.usage.promptTokens);
console.log('Completion tokens:', result.usage.completionTokens);

// Check for warnings
if (result.warnings && result.warnings.length > 0) {
  console.warn('Provider warnings:', result.warnings);
}
```

### onFinish Callback

Triggered after the last step finishes. Useful for logging, saving chat history, or recording usage:

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
  onFinish({ text, finishReason, usage, response, steps, totalUsage }) {
    // Log completion
    console.log('Generation finished:', finishReason);
    console.log('Total tokens:', totalUsage.totalTokens);

    // Access generated messages
    const messages = response.messages;

    // Save to database
    saveToDatabase({
      text,
      usage: totalUsage,
      messages,
    });

    // Record metrics
    trackMetrics({
      promptTokens: totalUsage.promptTokens,
      completionTokens: totalUsage.completionTokens,
      steps: steps.length,
    });
  },
});
```

### Accessing Response Headers & Body

Access raw response headers and body for provider-specific metadata:

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Explain machine learning',
});

// Access response headers
console.log('Response headers:', JSON.stringify(result.response.headers, null, 2));

// Access response body
console.log('Response body:', JSON.stringify(result.response.body, null, 2));

// Access response ID and metadata
console.log('Response ID:', result.response.id);
console.log('Model ID:', result.response.modelId);
console.log('Timestamp:', result.response.timestamp);

// Access messages that were generated
console.log('Messages:', result.response.messages);
```

---

## streamText Patterns

### Basic Streaming with textStream

`textStream` is both a `ReadableStream` and an `AsyncIterable`:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
});

// Method 1: Async iterable
for await (const textPart of result.textStream) {
  console.log(textPart);
}

// Method 2: ReadableStream (e.g., in Next.js API routes)
return new Response(result.textStream, {
  headers: { 'Content-Type': 'text/plain' },
});
```

**Important Notes:**
- `streamText` immediately starts streaming and suppresses errors to prevent server crashes
- Use the `onError` callback to log errors
- `streamText` uses backpressure - only generates tokens as requested
- You must consume the stream for it to finish

### Backpressure Handling

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Write a long story',
});

// Stream will only generate tokens as you consume them
// This prevents overwhelming the consumer
for await (const textPart of result.textStream) {
  // Process at your own pace
  await processText(textPart);

  // Optional: Add delay for rate limiting
  await new Promise(resolve => setTimeout(resolve, 50));
}
```

### onError Callback

Errors are part of the stream and not thrown. Use `onError` for logging:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
  onError({ error }) {
    // Log to your error tracking service
    console.error('Stream error:', error);
    trackError(error);

    // Optionally notify monitoring service
    notifyMonitoring({
      type: 'stream_error',
      error: error.message,
      stack: error.stack,
    });
  },
});
```

### onChunk Callback

Triggered for each chunk of the stream:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
  onChunk({ chunk }) {
    // Handle different chunk types
    switch (chunk.type) {
      case 'text':
        console.log('Text chunk:', chunk.text);
        break;
      case 'reasoning':
        console.log('Reasoning chunk:', chunk.reasoning);
        break;
      case 'source':
        console.log('Source:', chunk);
        break;
      case 'tool-call':
        console.log('Tool call:', chunk.toolName, chunk.args);
        break;
      case 'tool-input-start':
        console.log('Tool input starting');
        break;
      case 'tool-input-delta':
        console.log('Tool input delta:', chunk.delta);
        break;
      case 'tool-result':
        console.log('Tool result:', chunk.result);
        break;
      case 'raw':
        console.log('Raw chunk:', chunk);
        break;
    }
  },
});
```

### onFinish Callback

Triggered when the stream finishes:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
  onFinish({ text, finishReason, usage, response, steps, totalUsage }) {
    // Save chat history
    saveChatHistory({
      text,
      finishReason,
      usage: totalUsage,
    });

    // Access generated messages
    const messages = response.messages;

    // Log completion
    console.log('Stream finished:', {
      reason: finishReason,
      totalTokens: totalUsage.totalTokens,
      steps: steps.length,
    });
  },
});
```

### fullStream Property

Access all stream events for custom handling:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const result = streamText({
  model: openai('gpt-4'),
  tools: {
    cityAttractions: {
      inputSchema: z.object({ city: z.string() }),
      execute: async ({ city }) => ({
        attractions: ['attraction1', 'attraction2', 'attraction3'],
      }),
    },
  },
  prompt: 'What are some San Francisco tourist attractions?',
});

for await (const part of result.fullStream) {
  switch (part.type) {
    case 'start':
      console.log('Stream started');
      break;

    case 'start-step':
      console.log('Step started');
      break;

    case 'text-start':
      console.log('Text generation started');
      break;

    case 'text-delta':
      process.stdout.write(part.text);
      break;

    case 'text-end':
      console.log('\nText generation ended');
      break;

    case 'reasoning-start':
      console.log('Reasoning started');
      break;

    case 'reasoning-delta':
      console.log('Reasoning:', part.reasoning);
      break;

    case 'reasoning-end':
      console.log('Reasoning ended');
      break;

    case 'source':
      console.log('Source:', part);
      break;

    case 'file':
      console.log('File:', part);
      break;

    case 'tool-call':
      console.log(`Tool called: ${part.toolName}`, part.args);
      break;

    case 'tool-input-start':
      console.log('Tool input started');
      break;

    case 'tool-input-delta':
      console.log('Tool input delta:', part.delta);
      break;

    case 'tool-input-end':
      console.log('Tool input ended');
      break;

    case 'tool-result':
      console.log(`Tool result for ${part.toolName}:`, part.result);
      break;

    case 'tool-error':
      console.error('Tool error:', part.error);
      break;

    case 'finish-step':
      console.log('Step finished:', part.finishReason);
      break;

    case 'finish':
      console.log('Stream finished:', part.finishReason);
      break;

    case 'error':
      console.error('Stream error:', part.error);
      break;

    case 'raw':
      console.log('Raw value:', part);
      break;
  }
}
```

### Helper Functions

#### toUIMessageStreamResponse()

Creates a UI Message stream HTTP response (with tool calls) for Next.js App Router:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

export async function POST(req: Request) {
  const { messages } = await req.json();

  const result = streamText({
    model: openai('gpt-4'),
    messages,
  });

  return result.toUIMessageStreamResponse();
}
```

#### pipeUIMessageStreamToResponse()

Writes UI Message stream delta output to a Node.js response-like object:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';
import type { ServerResponse } from 'http';

export async function handler(req: any, res: ServerResponse) {
  const result = streamText({
    model: openai('gpt-4'),
    prompt: 'Write a story',
  });

  result.pipeUIMessageStreamToResponse(res);
}
```

### Stream Transformation

#### smoothStream

Use `smoothStream` to smooth out text streaming:

```typescript
import { smoothStream, streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Write a story',
  experimental_transform: smoothStream(),
});

for await (const textPart of result.textStream) {
  console.log(textPart); // Smoother output
}
```

#### Custom Transformations

Transform the stream before callbacks are invoked and promises are resolved:

```typescript
import { streamText, ToolSet, TextStreamPart } from 'ai';
import { openai } from '@ai-sdk/openai';

// Example: Convert text to uppercase
const upperCaseTransform =
  <TOOLS extends ToolSet>() =>
  (options: { tools: TOOLS; stopStream: () => void }) =>
    new TransformStream<TextStreamPart<TOOLS>, TextStreamPart<TOOLS>>({
      transform(chunk, controller) {
        controller.enqueue(
          chunk.type === 'text'
            ? { ...chunk, text: chunk.text.toUpperCase() }
            : chunk,
        );
      },
    });

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Write a haiku',
  experimental_transform: upperCaseTransform(),
});

for await (const textPart of result.textStream) {
  console.log(textPart); // ALL UPPERCASE
}
```

#### Stopping Streams with stopStream

Stop the stream when conditions are met (e.g., guardrails violated):

```typescript
import { streamText, ToolSet, TextStreamPart } from 'ai';
import { openai } from '@ai-sdk/openai';

const stopWordTransform =
  <TOOLS extends ToolSet>() =>
  ({ stopStream }: { stopStream: () => void }) =>
    new TransformStream<TextStreamPart<TOOLS>, TextStreamPart<TOOLS>>({
      transform(chunk, controller) {
        if (chunk.type !== 'text') {
          controller.enqueue(chunk);
          return;
        }

        if (chunk.text.includes('STOP')) {
          // Stop the stream
          stopStream();

          // Simulate finish-step event
          controller.enqueue({
            type: 'finish-step',
            finishReason: 'stop',
            logprobs: undefined,
            usage: {
              completionTokens: NaN,
              promptTokens: NaN,
              totalTokens: NaN,
            },
            request: {},
            response: {
              id: 'response-id',
              modelId: 'mock-model-id',
              timestamp: new Date(0),
            },
            warnings: [],
            isContinued: false,
          });

          // Simulate finish event
          controller.enqueue({
            type: 'finish',
            finishReason: 'stop',
            logprobs: undefined,
            usage: {
              completionTokens: NaN,
              promptTokens: NaN,
              totalTokens: NaN,
            },
            response: {
              id: 'response-id',
              modelId: 'mock-model-id',
              timestamp: new Date(0),
            },
          });

          return;
        }

        controller.enqueue(chunk);
      },
    });

const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Write until you say STOP',
  experimental_transform: stopWordTransform(),
});
```

#### Multiple Transformations

Apply multiple transformations in order:

```typescript
const result = streamText({
  model: openai('gpt-4'),
  prompt: 'Write a story',
  experimental_transform: [
    firstTransform,
    secondTransform,
    smoothStream(),
  ],
});
```

---

## Sources

Some providers (Perplexity, Google Generative AI) include sources in responses to ground the output with web pages.

### Source Properties

Each `url` source contains:

- `id`: Source ID
- `url`: Source URL
- `title`: Optional title
- `sourceType`: Always `'url'` for URL sources
- `providerMetadata`: Provider-specific metadata

### Sources with generateText

```typescript
import { generateText } from 'ai';
import { google } from '@ai-sdk/google';

const result = await generateText({
  model: google('gemini-2.5-flash'),
  tools: {
    google_search: google.tools.googleSearch({}),
  },
  prompt: 'List the top 5 San Francisco news from the past week.',
});

for (const source of result.sources) {
  if (source.sourceType === 'url') {
    console.log('ID:', source.id);
    console.log('Title:', source.title);
    console.log('URL:', source.url);
    console.log('Provider metadata:', source.providerMetadata);
    console.log();
  }
}
```

### Sources with streamText (fullStream)

```typescript
import { streamText } from 'ai';
import { google } from '@ai-sdk/google';

const result = streamText({
  model: google('gemini-2.5-flash'),
  tools: {
    google_search: google.tools.googleSearch({}),
  },
  prompt: 'List the top 5 San Francisco news from the past week.',
});

for await (const part of result.fullStream) {
  if (part.type === 'source' && part.sourceType === 'url') {
    console.log('ID:', part.id);
    console.log('Title:', part.title);
    console.log('URL:', part.url);
    console.log('Provider metadata:', part.providerMetadata);
    console.log();
  }
}
```

### Sources Promise

Sources are also available via the `result.sources` promise:

```typescript
const result = streamText({
  model: google('gemini-2.5-flash'),
  tools: {
    google_search: google.tools.googleSearch({}),
  },
  prompt: 'Latest AI research',
});

// Consume stream
for await (const textPart of result.textStream) {
  process.stdout.write(textPart);
}

// Access sources after streaming
const sources = await result.sources;
console.log('Sources:', sources);
```

---

## Settings Reference

Common settings supported by `generateText`, `streamText`, and other AI SDK functions.

### maxOutputTokens

Maximum number of tokens to generate.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a short story',
  maxOutputTokens: 512, // Limit to 512 tokens
});
```

### temperature

Temperature setting for randomness. Range depends on provider (typically 0-2).

- `0`: Almost deterministic results
- Higher values: More randomness and creativity

**Recommendation:** Set either `temperature` or `topP`, but not both.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a creative poem',
  temperature: 0.8, // More creative
});

const deterministic = await generateText({
  model: openai('gpt-4'),
  prompt: 'Summarize this article',
  temperature: 0.1, // More focused
});
```

**Note:** In AI SDK 5.0+, temperature is no longer set to `0` by default.

### topP

Nucleus sampling. Number between 0 and 1 for most providers.

- `0.1`: Only tokens with top 10% probability mass considered
- `1.0`: All tokens considered

**Recommendation:** Set either `temperature` or `topP`, but not both.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Explain quantum physics',
  topP: 0.9,
});
```

### topK

Sample from top K options for each token. Removes "long tail" low probability responses.

**Recommendation:** Advanced use cases only. Usually only need `temperature`.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write code',
  topK: 50,
});
```

### presencePenalty

Affects likelihood of model repeating information already in the prompt.

- `0`: No penalty (default for most providers)
- Range varies by provider

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write about machine learning',
  presencePenalty: 0.6, // Discourage repetition
});
```

### frequencyPenalty

Affects likelihood of repeatedly using the same words or phrases.

- `0`: No penalty (default for most providers)
- Range varies by provider

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a diverse essay',
  frequencyPenalty: 0.5, // Discourage word repetition
});
```

### stopSequences

Stop sequences to halt text generation. Model stops when any sequence is generated.

Providers may have limits on the number of stop sequences.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a numbered list',
  stopSequences: ['\n\n', 'END'],
});
```

### seed

Integer seed for random sampling. Generates deterministic results when set and supported.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Generate a random number',
  seed: 42, // Same seed = same output
});
```

### maxRetries

Maximum number of retries on failures. Default: `2`.

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Analyze this data',
  maxRetries: 5, // Retry up to 5 times
});

const noRetries = await generateText({
  model: openai('gpt-4'),
  prompt: 'Quick query',
  maxRetries: 0, // Disable retries
});
```

### abortSignal

Abort signal to cancel the call. Useful for timeouts or user cancellation.

```typescript
// Example: Timeout
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
  abortSignal: AbortSignal.timeout(5000), // 5 second timeout
});

// Example: User cancellation
const controller = new AbortController();

// In another part of code (e.g., button click)
// controller.abort();

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Long running task',
  abortSignal: controller.signal,
});
```

### headers

Additional HTTP headers sent with request. Only for HTTP-based providers.

Use for provider-specific metadata or observability (e.g., `Prompt-Id`).

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Invent a new holiday and describe its traditions.',
  headers: {
    'Prompt-Id': 'my-prompt-id',
    'X-Custom-Header': 'custom-value',
  },
});
```

**Note:** This is for request-specific headers. You can also set headers in the provider configuration for all requests:

```typescript
import { openai } from '@ai-sdk/openai';

const provider = openai({
  apiKey: process.env.OPENAI_API_KEY,
  headers: {
    'X-Global-Header': 'global-value',
  },
});
```

### Provider Warnings

Some providers don't support all settings. Check `warnings` for unsupported settings:

```typescript
const result = await generateText({
  model: someProvider('model'),
  prompt: 'Test',
  temperature: 0.7,
  topK: 50, // May not be supported
});

if (result.warnings && result.warnings.length > 0) {
  console.warn('Settings warnings:', result.warnings);
}
```
