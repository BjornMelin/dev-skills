# AI SDK Core - Production Patterns Reference

Comprehensive guide to production-ready patterns for AI SDK Core v6, covering telemetry, error handling, testing, prompt engineering, and media APIs.

## Table of Contents

- [Telemetry (OpenTelemetry)](#telemetry-opentelemetry)
- [DevTools](#devtools)
- [Error Handling](#error-handling)
- [Testing Patterns](#testing-patterns)
- [Prompt Engineering](#prompt-engineering)
- [Media APIs (Experimental)](#media-apis-experimental)

---

## Telemetry (OpenTelemetry)

**Status**: Experimental (may change)

AI SDK uses [OpenTelemetry](https://opentelemetry.io/) for standardized observability instrumentation.

### Enabling Telemetry

For Next.js apps, follow the [Next.js OpenTelemetry guide](https://nextjs.org/docs/app/building-your-application/optimizing/open-telemetry) first.

Enable telemetry per function call using `experimental_telemetry`:

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a short story about a cat.',
  experimental_telemetry: { isEnabled: true },
});
```

### Recording Controls

Control recording of inputs and outputs (default: both enabled). Disable for privacy, data transfer, or performance reasons:

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Sensitive user data here...',
  experimental_telemetry: {
    isEnabled: true,
    recordInputs: false,  // Don't record sensitive prompts
    recordOutputs: true,  // Record outputs only
  },
});
```

### Telemetry Metadata

Add custom identifiers and metadata for tracking:

```typescript
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a short story about a cat.',
  experimental_telemetry: {
    isEnabled: true,
    functionId: 'story-generator-v2',
    metadata: {
      userId: 'user-123',
      feature: 'creative-writing',
      environment: 'production',
    },
  },
});
```

### Custom Tracer Provider

Provide a custom `TracerProvider` instead of using the `@opentelemetry/api` singleton:

```typescript
import { NodeTracerProvider } from '@opentelemetry/sdk-trace-node';
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const tracerProvider = new NodeTracerProvider();
const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a short story about a cat.',
  experimental_telemetry: {
    isEnabled: true,
    tracer: tracerProvider.getTracer('ai'),
  },
});
```

### Span Types

#### generateText Spans

**`ai.generateText` (span)**: Full function call, contains 1+ `ai.generateText.doGenerate` spans

Attributes:
- `operation.name`: `ai.generateText` + functionId
- `ai.operationId`: `"ai.generateText"`
- `ai.prompt`: Input prompt
- `ai.response.text`: Generated text
- `ai.response.toolCalls`: Tool calls (stringified JSON)
- `ai.response.finishReason`: Completion reason
- `ai.settings.maxOutputTokens`: Max output tokens

**`ai.generateText.doGenerate` (span)**: Individual provider call, can contain `ai.toolCall` spans

Attributes:
- `operation.name`: `ai.generateText.doGenerate` + functionId
- `ai.operationId`: `"ai.generateText.doGenerate"`
- `ai.prompt.messages`: Messages passed to provider
- `ai.prompt.tools`: Tool definitions array (stringified)
- `ai.prompt.toolChoice`: Tool choice setting (stringified JSON)
- `ai.response.text`: Generated text
- `ai.response.toolCalls`: Tool calls (stringified JSON)
- `ai.response.finishReason`: Completion reason

**`ai.toolCall` (span)**: Individual tool execution

Attributes:
- `operation.name`: `"ai.toolCall"`
- `ai.operationId`: `"ai.toolCall"`
- `ai.toolCall.name`: Tool name
- `ai.toolCall.id`: Tool call ID
- `ai.toolCall.args`: Input parameters
- `ai.toolCall.result`: Output result (if serializable and successful)

#### streamText Spans & Events

**`ai.streamText` (span)**: Full streaming call, contains `ai.streamText.doStream`

Attributes: Same as `ai.generateText`

**`ai.streamText.doStream` (span)**: Provider streaming call

Additional attributes:
- `ai.response.msToFirstChunk`: Time to first chunk (ms)
- `ai.response.msToFinish`: Time to finish part (ms)
- `ai.response.avgCompletionTokensPerSecond`: Average tokens/second

**`ai.stream.firstChunk` (event)**: Emitted on first chunk
- `ai.response.msToFirstChunk`: Time to first chunk

**`ai.stream.finish` (event)**: Emitted on stream completion

#### generateObject Spans

**`ai.generateObject` (span)**: Full function call

Additional attributes:
- `ai.schema`: JSON schema (stringified)
- `ai.schema.name`: Schema name
- `ai.schema.description`: Schema description
- `ai.response.object`: Generated object (stringified JSON)
- `ai.settings.output`: Output type (`object`, `no-schema`)

**`ai.generateObject.doGenerate` (span)**: Provider call

#### streamObject Spans

**`ai.streamObject` (span)**: Full streaming call

**`ai.streamObject.doStream` (span)**: Provider streaming call

Additional attributes:
- `ai.response.msToFirstChunk`: Time to first chunk

#### embed Spans

**`ai.embed` (span)**: Full embedding call

Attributes:
- `ai.value`: Input value
- `ai.embedding`: JSON-stringified embedding

**`ai.embed.doEmbed` (span)**: Provider call

Attributes:
- `ai.values`: Input values array
- `ai.embeddings`: Embeddings array (stringified)

#### embedMany Spans

**`ai.embedMany` (span)**: Batch embedding call

**`ai.embedMany.doEmbed` (span)**: Provider batch call

### Basic LLM Span Information

All LLM spans include:
- `resource.name`: functionId
- `ai.model.id`: Model ID
- `ai.model.provider`: Provider name
- `ai.request.headers.*`: Request headers
- `ai.response.providerMetadata`: Provider-specific metadata
- `ai.settings.maxRetries`: Max retries
- `ai.telemetry.functionId`: Function ID
- `ai.telemetry.metadata.*`: Custom metadata
- `ai.usage.completionTokens`: Completion tokens
- `ai.usage.promptTokens`: Prompt tokens

### Call LLM Span Information

Individual LLM call spans add:
- `ai.response.model`: Actual model used (may differ from requested)
- `ai.response.id`: Response ID
- `ai.response.timestamp`: Response timestamp
- Semantic Conventions for GenAI:
  - `gen_ai.system`: Provider name
  - `gen_ai.request.model`: Requested model
  - `gen_ai.request.temperature`: Temperature setting
  - `gen_ai.request.max_tokens`: Max tokens
  - `gen_ai.request.frequency_penalty`: Frequency penalty
  - `gen_ai.request.presence_penalty`: Presence penalty
  - `gen_ai.request.top_k`: Top K
  - `gen_ai.request.top_p`: Top P
  - `gen_ai.request.stop_sequences`: Stop sequences
  - `gen_ai.response.finish_reasons`: Finish reasons
  - `gen_ai.usage.input_tokens`: Input tokens
  - `gen_ai.usage.output_tokens`: Output tokens

---

## DevTools

Debug AI SDK applications with the official DevTools during development.

### Installation & Usage

```bash
# Install
npm install @ai-sdk/devtools

# Run DevTools server
npx @ai-sdk/devtools
```

DevTools provides a local web interface at `http://localhost:3001` for debugging AI SDK calls.

### Features

- **Request/Response Inspection**: Real-time view of all AI SDK calls
- **Token Usage Visualization**: See input/output token counts per call
- **Latency Breakdown**: Per-provider timing metrics
- **Tool Call Tracing**: Step-by-step tool execution visibility
- **Stream Debugging**: Inspect streaming responses chunk by chunk
- **Message History**: Full conversation context for multi-turn sessions

### Integration with Telemetry

DevTools complements OpenTelemetry. Use both for comprehensive observability:

| Tool | Use Case | Environment |
|------|----------|-------------|
| DevTools | Interactive debugging UI | Development |
| OpenTelemetry | Traces, metrics, alerting | Production |

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Write a short story.',
  // DevTools captures this automatically when running
  experimental_telemetry: {
    isEnabled: true,
    functionId: 'story-generator',
  },
});
```

DevTools automatically captures all AI SDK calls in the same Node.js process. No code changes required—just start the DevTools server alongside your application.

---

## Error Handling

AI SDK Core provides typed errors for robust error handling in production.

### Error Types

- `AI_APICallError`: API call failures
- `AI_NoContentGeneratedError`: No content in response
- `AI_InvalidPromptError`: Invalid prompt format
- `AI_InvalidModelError`: Invalid model configuration
- `AI_NoImageGeneratedError`: Image generation failed
- `AI_NoSpeechGeneratedError`: Speech generation failed
- `AI_NoTranscriptGeneratedError`: Transcription failed

### Synchronous Error Handling

Use try/catch for regular errors:

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

try {
  const { text } = await generateText({
    model: openai('gpt-4'),
    prompt: 'Write a vegetarian lasagna recipe for 4 people.',
  });
  console.log(text);
} catch (error) {
  // Handle API errors, invalid prompts, etc.
  console.error('Generation failed:', error);
}
```

### Streaming Error Handling (Simple Streams)

For streams without error chunk support, errors throw as regular errors:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

try {
  const { textStream } = streamText({
    model: openai('gpt-4'),
    prompt: 'Write a vegetarian lasagna recipe for 4 people.',
  });

  for await (const textPart of textStream) {
    process.stdout.write(textPart);
  }
} catch (error) {
  console.error('Stream failed:', error);
}
```

### Streaming Error Handling (Full Streams)

Full streams support error parts for in-stream error handling:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

try {
  const { fullStream } = streamText({
    model: openai('gpt-4'),
    prompt: 'Write a vegetarian lasagna recipe for 4 people.',
  });

  for await (const part of fullStream) {
    switch (part.type) {
      case 'text-delta':
        process.stdout.write(part.textDelta);
        break;

      case 'error': {
        const error = part.error;
        console.error('Stream error:', error);
        // Handle error (log, retry, notify user)
        break;
      }

      case 'abort': {
        console.log('Stream aborted by user');
        // Handle stream abort
        break;
      }

      case 'tool-error': {
        const error = part.error;
        console.error('Tool execution error:', error);
        // Handle tool-specific errors
        break;
      }

      case 'finish':
        console.log('\nStream completed');
        break;
    }
  }
} catch (error) {
  // Handle errors outside streaming (setup, network)
  console.error('Stream initialization failed:', error);
}
```

### onError and onAbort Callbacks

Use callbacks for cleanup and state updates:

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const { textStream } = streamText({
  model: openai('gpt-4'),
  prompt: 'Write a vegetarian lasagna recipe for 4 people.',

  onAbort: ({ steps }) => {
    // Called when stream aborted via AbortSignal (onFinish NOT called)
    console.log('Stream aborted after', steps.length, 'steps');
    // Update UI state, save partial results, etc.
  },

  onFinish: ({ steps, totalUsage, text }) => {
    // Called on normal completion (NOT called on abort)
    console.log('Stream completed normally');
    console.log('Total tokens:', totalUsage.totalTokens);
  },
});

for await (const textPart of textStream) {
  process.stdout.write(textPart);
}
```

`onAbort` receives:
- `steps`: Array of completed steps before abort

### Handling Abort Events Directly

```typescript
import { streamText } from 'ai';
import { openai } from '@ai-sdk/openai';

const { fullStream } = streamText({
  model: openai('gpt-4'),
  prompt: 'Write a vegetarian lasagna recipe for 4 people.',
});

for await (const chunk of fullStream) {
  switch (chunk.type) {
    case 'abort': {
      console.log('Stream was aborted');
      // Perform cleanup immediately
      break;
    }
    case 'text-delta':
      process.stdout.write(chunk.textDelta);
      break;
  }
}
```

### Tool-Specific Error Extraction

Extract and handle tool errors separately:

```typescript
import { generateText, stepCountIs, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const result = await generateText({
  model: openai('gpt-4'),
  tools: {
    weather: tool({
      description: 'Get weather for a location',
      inputSchema: z.object({
        location: z.string(),
      }),
      execute: async ({ location }) => {
        // Simulate tool error
        throw new Error(`Weather API unavailable for ${location}`);
      },
    }),
  },
  prompt: 'What is the weather in Paris?',
  stopWhen: stepCountIs(3),
});

// Check for tool errors in response
const toolErrors = result.steps.flatMap(step =>
  step.content.filter(part => part.type === 'tool-error')
);

for (const toolError of toolErrors) {
  console.error('Tool error:', toolError.toolName, toolError.error);
}
```

---

## Testing Patterns

AI SDK Core provides mock providers and helpers for deterministic, fast, cost-free testing.

### Mock Providers

Import from `ai/test`:
- `MockLanguageModel`: Mock language model (v6)
- `MockEmbeddingModel`: Mock embedding model (v6)
- `mockId`: Incrementing integer ID generator
- `mockValues`: Iterator over values array
- `simulateReadableStream`: Simulates streams with delays

**Note**: In v6, `MockLanguageModelV2` was renamed to `MockLanguageModel` and `MockEmbeddingModelV2` to `MockEmbeddingModel`.

### Testing generateText

```typescript
import { generateText } from 'ai';
import { MockLanguageModel } from 'ai/test';

const result = await generateText({
  model: new MockLanguageModel({
    doGenerate: async () => ({
      finishReason: 'stop',
      usage: { inputTokens: 10, outputTokens: 20, totalTokens: 30 },
      content: [{ type: 'text', text: 'Hello, world!' }],
      warnings: [],
    }),
  }),
  prompt: 'Hello, test!',
});

expect(result.text).toBe('Hello, world!');
expect(result.usage.totalTokens).toBe(30);
```

### Testing streamText

```typescript
import { streamText } from 'ai';
import { MockLanguageModel, simulateReadableStream } from 'ai/test';

const result = streamText({
  model: new MockLanguageModel({
    doStream: async () => ({
      stream: simulateReadableStream({
        chunks: [
          { type: 'text-start', id: 'text-1' },
          { type: 'text-delta', id: 'text-1', delta: 'Hello' },
          { type: 'text-delta', id: 'text-1', delta: ', ' },
          { type: 'text-delta', id: 'text-1', delta: 'world!' },
          { type: 'text-end', id: 'text-1' },
          {
            type: 'finish',
            finishReason: 'stop',
            logprobs: undefined,
            usage: { inputTokens: 3, outputTokens: 10, totalTokens: 13 },
          },
        ],
      }),
    }),
  }),
  prompt: 'Hello, test!',
});

const parts: string[] = [];
for await (const part of result.textStream) {
  parts.push(part);
}

expect(parts.join('')).toBe('Hello, world!');
```

### Testing generateObject

```typescript
import { generateObject } from 'ai';
import { MockLanguageModel } from 'ai/test';
import { z } from 'zod';

const result = await generateObject({
  model: new MockLanguageModel({
    doGenerate: async () => ({
      finishReason: 'stop',
      usage: { inputTokens: 10, outputTokens: 20, totalTokens: 30 },
      content: [{ type: 'text', text: '{"content":"Hello, world!"}' }],
      warnings: [],
    }),
  }),
  schema: z.object({ content: z.string() }),
  prompt: 'Hello, test!',
});

expect(result.object).toEqual({ content: 'Hello, world!' });
```

### Testing streamObject

```typescript
import { streamObject } from 'ai';
import { MockLanguageModel, simulateReadableStream } from 'ai/test';
import { z } from 'zod';

const result = streamObject({
  model: new MockLanguageModel({
    doStream: async () => ({
      stream: simulateReadableStream({
        chunks: [
          { type: 'text-start', id: 'text-1' },
          { type: 'text-delta', id: 'text-1', delta: '{ ' },
          { type: 'text-delta', id: 'text-1', delta: '"content": ' },
          { type: 'text-delta', id: 'text-1', delta: '"Hello, ' },
          { type: 'text-delta', id: 'text-1', delta: 'world' },
          { type: 'text-delta', id: 'text-1', delta: '!"' },
          { type: 'text-delta', id: 'text-1', delta: ' }' },
          { type: 'text-end', id: 'text-1' },
          {
            type: 'finish',
            finishReason: 'stop',
            logprobs: undefined,
            usage: { inputTokens: 3, outputTokens: 10, totalTokens: 13 },
          },
        ],
      }),
    }),
  }),
  schema: z.object({ content: z.string() }),
  prompt: 'Hello, test!',
});

const finalObject = await result.object;
expect(finalObject).toEqual({ content: 'Hello, world!' });
```

### Simulating UI Message Streams

Test or debug UI message streams with controlled delays:

```typescript
import { simulateReadableStream } from 'ai';

// Next.js route example
export async function POST(req: Request) {
  return new Response(
    simulateReadableStream({
      initialDelayInMs: 1000,  // Delay before first chunk
      chunkDelayInMs: 300,     // Delay between chunks
      chunks: [
        'data: {"type":"start","messageId":"msg-123"}\n\n',
        'data: {"type":"text-start","id":"text-1"}\n\n',
        'data: {"type":"text-delta","id":"text-1","delta":"This"}\n\n',
        'data: {"type":"text-delta","id":"text-1","delta":" is an"}\n\n',
        'data: {"type":"text-delta","id":"text-1","delta":" example."}\n\n',
        'data: {"type":"text-end","id":"text-1"}\n\n',
        'data: {"type":"finish"}\n\n',
        'data: [DONE]\n\n',
      ],
    }).pipeThrough(new TextEncoderStream()),
    {
      status: 200,
      headers: {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        Connection: 'keep-alive',
        'x-vercel-ai-ui-message-stream': 'v1',
      },
    },
  );
}
```

---

## Prompt Engineering

Best practices for effective prompts, especially for tools and structured data.

### Prompts for Tools

As tool count/complexity increases, results degrade. Follow these tips:

1. **Use strong models**: `gpt-5`, `gpt-4.1` excel at tool calling. Weaker models struggle.
2. **Limit tool count**: Keep to 5 or fewer tools.
3. **Simplify schemas**: Avoid deep nesting, excessive optional fields, complex unions.
4. **Use semantic names**: Meaningful tool names, parameters, and properties help models understand intent.
5. **Add descriptions**: Use `.describe("...")` on Zod schema properties.
6. **Document tool outputs**: If tool output is unclear or tools depend on each other, describe the output in the tool's `description`.
7. **Include examples**: Provide example input/output JSON in prompts for complex tools.

**Example with descriptions:**

```typescript
import { generateText, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const result = await generateText({
  model: openai('gpt-4'),
  tools: {
    weather: tool({
      description: 'Get current weather for a location. Returns temperature in Celsius and conditions.',
      inputSchema: z.object({
        location: z.string().describe('City name, e.g. "San Francisco" or "London"'),
        units: z.enum(['celsius', 'fahrenheit']).describe('Temperature units'),
      }),
      execute: async ({ location, units }) => {
        return { temperature: 22, conditions: 'Sunny', units };
      },
    }),
  },
  prompt: 'What is the weather in Paris?',
});
```

### Tool & Structured Data Schemas

#### Zod Dates

Models return dates as strings, not Date objects. Use transformations:

```typescript
import { generateObject } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const result = await generateObject({
  model: openai('gpt-4'),
  schema: z.object({
    events: z.array(
      z.object({
        event: z.string(),
        date: z
          .iso.date()  // Validates ISO 8601 date string
          .transform(value => new Date(value)),  // Converts to Date
      }),
    ),
  }),
  prompt: 'List 5 important events from the year 2000.',
});

console.log(result.object.events[0].date instanceof Date);  // true
```

#### Optional Parameters (Strict Schemas)

For strict schema validation (e.g., OpenAI structured outputs), use `.nullable()` instead of `.optional()`:

```typescript
import { tool } from 'ai';
import { z } from 'zod';

// ❌ This may fail with strict schema validation
const failingTool = tool({
  description: 'Execute a command',
  inputSchema: z.object({
    command: z.string(),
    workdir: z.string().optional(),  // May cause errors
    timeout: z.string().optional(),
  }),
});

// ✅ This works with strict schema validation
const workingTool = tool({
  description: 'Execute a command',
  inputSchema: z.object({
    command: z.string(),
    workdir: z.string().nullable(),  // Use nullable
    timeout: z.string().nullable(),
  }),
});
```

#### Temperature Settings

For deterministic tool calls and object generation, use `temperature: 0`:

```typescript
import { generateText, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const result = await generateText({
  model: openai('gpt-4'),
  temperature: 0,  // Deterministic tool calls
  tools: {
    executeCommand: tool({
      description: 'Execute a shell command',
      inputSchema: z.object({
        command: z.string(),
      }),
      execute: async ({ command }) => {
        // Execute command
        return { output: 'Command executed' };
      },
    }),
  },
  prompt: 'Execute the ls command',
});
```

Lower temperatures reduce randomness, crucial for:
- Structured data generation
- Precise tool calls with correct parameters
- Consistent schema adherence

### Debugging

#### Inspecting Warnings

Check call warnings to ensure your prompt/tools are handled correctly:

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Hello, world!',
  temperature: 2.5,  // May trigger warning if unsupported
});

console.log(result.warnings);
// Check for unsupported parameters, deprecations, etc.
```

#### HTTP Request Bodies

Inspect raw HTTP request bodies for debugging (provider-specific):

```typescript
import { generateText } from 'ai';
import { openai } from '@ai-sdk/openai';

const result = await generateText({
  model: openai('gpt-4'),
  prompt: 'Hello, world!',
});

console.log(result.request.body);
// View exact payload sent to provider
```

---

## Media APIs (Experimental)

AI SDK Core provides experimental APIs for image generation, speech synthesis, and audio transcription.

### generateImage

Generate images from text prompts using image models.

```typescript
import { experimental_generateImage as generateImage } from 'ai';
import { openai } from '@ai-sdk/openai';

const { image } = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
});

const base64 = image.base64;         // Base64 image data
const uint8Array = image.uint8Array; // Uint8Array binary data
```

#### Size and Aspect Ratio

Specify size (format: `{width}x{height}`) or aspect ratio (format: `{width}:{height}`). Supported values vary by model.

```typescript
// Size (OpenAI)
const { image } = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
  size: '1024x1024',  // 1024x1024, 1792x1024, 1024x1792
});

// Aspect Ratio (Google Vertex)
import { vertex } from '@ai-sdk/google-vertex';

const { image: vertexImage } = await generateImage({
  model: vertex.image('imagen-3.0-generate-002'),
  prompt: 'Santa Claus driving a Cadillac',
  aspectRatio: '16:9',  // 1:1, 3:4, 4:3, 9:16, 16:9
});
```

#### Generating Multiple Images

```typescript
const { images } = await generateImage({
  model: openai.image('dall-e-2'),
  prompt: 'Santa Claus driving a Cadillac',
  n: 4,  // Generate 4 images
});

// SDK batches requests automatically based on model limits
// Override with maxImagesPerCall:
const { images: batchedImages } = await generateImage({
  model: openai.image('dall-e-2'),
  prompt: 'Santa Claus driving a Cadillac',
  maxImagesPerCall: 5,  // Custom batch size
  n: 10,  // 2 calls of 5 images each
});
```

#### Seed for Reproducibility

```typescript
const { image } = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
  seed: 1234567890,  // Same seed = same image (if supported)
});
```

#### Provider-Specific Options

```typescript
const { image } = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
  size: '1024x1024',
  providerOptions: {
    openai: {
      style: 'vivid',   // 'vivid' or 'natural'
      quality: 'hd',    // 'hd' or 'standard'
    },
  },
});
```

#### Error Handling

```typescript
import {
  experimental_generateImage as generateImage,
  NoImageGeneratedError,
} from 'ai';
import { openai } from '@ai-sdk/openai';

try {
  const { image } = await generateImage({
    model: openai.image('dall-e-3'),
    prompt: 'Santa Claus driving a Cadillac',
  });
} catch (error) {
  if (NoImageGeneratedError.isInstance(error)) {
    console.log('NoImageGeneratedError');
    console.log('Cause:', error.cause);
    console.log('Responses:', error.responses);
  }
}
```

### generateSpeech

Generate speech audio from text using text-to-speech models.

```typescript
import { experimental_generateSpeech as generateSpeech } from 'ai';
import { openai } from '@ai-sdk/openai';

const audio = await generateSpeech({
  model: openai.speech('tts-1'),
  text: 'Hello, world!',
  voice: 'alloy',  // OpenAI voices: alloy, echo, fable, onyx, nova, shimmer
});

const audioData = audio.audioData;  // Uint8Array
```

#### Language Setting

```typescript
import { experimental_generateSpeech as generateSpeech } from 'ai';
import { lmnt } from '@ai-sdk/lmnt';

const audio = await generateSpeech({
  model: lmnt.speech('aurora'),
  text: 'Hola, mundo!',
  language: 'es',  // Spanish (provider support varies)
});
```

#### Provider-Specific Options

```typescript
const audio = await generateSpeech({
  model: openai.speech('tts-1'),
  text: 'Hello, world!',
  voice: 'alloy',
  providerOptions: {
    openai: {
      speed: 1.25,  // 0.25 to 4.0
    },
  },
});
```

#### Error Handling

```typescript
import {
  experimental_generateSpeech as generateSpeech,
  NoSpeechGeneratedError,
} from 'ai';
import { openai } from '@ai-sdk/openai';

try {
  const audio = await generateSpeech({
    model: openai.speech('tts-1'),
    text: 'Hello, world!',
    voice: 'alloy',
  });
} catch (error) {
  if (NoSpeechGeneratedError.isInstance(error)) {
    console.log('NoSpeechGeneratedError');
    console.log('Cause:', error.cause);
    console.log('Responses:', error.responses);
  }
}
```

### transcribe

Transcribe audio to text using transcription models.

```typescript
import { experimental_transcribe as transcribe } from 'ai';
import { openai } from '@ai-sdk/openai';
import { readFile } from 'fs/promises';

const transcript = await transcribe({
  model: openai.transcription('whisper-1'),
  audio: await readFile('audio.mp3'),  // Uint8Array, ArrayBuffer, Buffer, base64 string, or URL
});

const text = transcript.text;                      // "Hello, world!"
const segments = transcript.segments;              // Segments with timestamps (if available)
const language = transcript.language;              // "en" (if available)
const durationInSeconds = transcript.durationInSeconds;  // Duration (if available)
```

#### Provider-Specific Options

```typescript
const transcript = await transcribe({
  model: openai.transcription('whisper-1'),
  audio: await readFile('audio.mp3'),
  providerOptions: {
    openai: {
      timestampGranularities: ['word'],  // Get word-level timestamps
    },
  },
});

// Access word-level segments
for (const segment of transcript.segments ?? []) {
  console.log(`[${segment.start}s - ${segment.end}s]: ${segment.text}`);
}
```

#### Error Handling

```typescript
import {
  experimental_transcribe as transcribe,
  NoTranscriptGeneratedError,
} from 'ai';
import { openai } from '@ai-sdk/openai';
import { readFile } from 'fs/promises';

try {
  const transcript = await transcribe({
    model: openai.transcription('whisper-1'),
    audio: await readFile('audio.mp3'),
  });
} catch (error) {
  if (NoTranscriptGeneratedError.isInstance(error)) {
    console.log('NoTranscriptGeneratedError');
    console.log('Cause:', error.cause);
    console.log('Responses:', error.responses);
  }
}
```

### Common Media API Settings

All media APIs support:

#### Abort Signals and Timeouts

```typescript
const result = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
  abortSignal: AbortSignal.timeout(5000),  // 5 second timeout
});
```

#### Custom Headers

```typescript
const result = await generateSpeech({
  model: openai.speech('tts-1'),
  text: 'Hello, world!',
  voice: 'alloy',
  headers: { 'X-Custom-Header': 'custom-value' },
});
```

#### Warnings

```typescript
const { image, warnings } = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
});

console.log(warnings);  // Check for unsupported parameters
```

#### Provider Metadata

```typescript
const { image, providerMetadata } = await generateImage({
  model: openai.image('dall-e-3'),
  prompt: 'Santa Claus driving a Cadillac',
});

// OpenAI returns revised prompts
const revisedPrompt = providerMetadata.openai.images[0]?.revisedPrompt;
console.log('Original:', 'Santa Claus driving a Cadillac');
console.log('Revised:', revisedPrompt);
```
