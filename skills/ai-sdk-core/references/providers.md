# AI SDK Core: Provider Configuration Reference

Comprehensive reference for configuring and using AI providers with Vercel AI SDK v6.

## OpenAI Provider

### Setup

```typescript
import { openai } from '@ai-sdk/openai';
import { createOpenAI } from '@ai-sdk/openai';

// Default instance (uses OPENAI_API_KEY env var)
const model = openai('gpt-5');

// Custom instance with settings
const custom = createOpenAI({
  apiKey: process.env.OPENAI_API_KEY,
  baseURL: 'https://api.openai.com/v1',
  organization: 'org-123',
  project: 'proj-456',
  headers: { 'Custom-Header': 'value' },
  fetch: customFetchImpl,
});
```

### Responses API vs Chat API

OpenAI provider defaults to the Responses API (since AI SDK 5). Explicitly select APIs:

```typescript
// Responses API (default)
const model = openai('gpt-5');
const model2 = openai.responses('gpt-5');

// Chat API
const chatModel = openai.chat('gpt-5');

// Completion API (legacy)
const completionModel = openai.completion('gpt-3.5-turbo-instruct');
```

### Responses API Provider Options

```typescript
import { openai, OpenAIResponsesProviderOptions } from '@ai-sdk/openai';
import { generateText } from 'ai';

const result = await generateText({
  model: openai('gpt-5'),
  prompt: 'Explain quantum computing',
  providerOptions: {
    openai: {
      parallelToolCalls: false,
      store: false,
      user: 'user_123',
      maxToolCalls: 10,
      metadata: { session: 'abc123' },

      // Conversation continuity
      conversation: 'conv_id_from_api',
      previousResponseId: 'resp_xyz',
      instructions: 'Updated system instructions',

      // Reasoning models (o3, o4, gpt-5)
      reasoningEffort: 'high', // 'none' | 'minimal' | 'low' | 'medium' | 'high'
      reasoningSummary: 'detailed', // 'auto' | 'detailed'

      // Structured outputs
      strictJsonSchema: true, // enabled by default

      // Service tier
      serviceTier: 'flex', // 'auto' | 'flex' | 'priority' | 'default'

      // Verbosity control
      textVerbosity: 'medium', // 'low' | 'medium' | 'high'

      // Advanced options
      include: ['file_search_call.results', 'message.output_text.logprobs'],
      truncation: 'auto', // or 'disabled'

      // Prompt caching
      promptCacheKey: 'my-custom-cache-key-123',
      promptCacheRetention: '24h', // 'in_memory' | '24h' (GPT-5.1 only)

      safetyIdentifier: 'user-stable-id',
    } satisfies OpenAIResponsesProviderOptions,
  },
});

// Access response metadata
const { responseId, cachedPromptTokens, reasoningTokens } =
  result.providerMetadata?.openai;
```

### Reasoning Output

For reasoning models (o3, o4-mini, gpt-5), enable reasoning summaries:

```typescript
// Streaming reasoning
const result = streamText({
  model: openai('gpt-5'),
  prompt: 'Explain the Mission burrito debate in San Francisco.',
  providerOptions: {
    openai: {
      reasoningSummary: 'detailed', // 'auto' or 'detailed'
    },
  },
});

for await (const part of result.fullStream) {
  if (part.type === 'reasoning') {
    console.log(`Reasoning: ${part.textDelta}`);
  } else if (part.type === 'text-delta') {
    process.stdout.write(part.textDelta);
  }
}

// Non-streaming reasoning
const { text, reasoning } = await generateText({
  model: openai('gpt-5'),
  prompt: 'Complex reasoning task',
  providerOptions: {
    openai: { reasoningSummary: 'auto' },
  },
});
console.log('Reasoning:', reasoning);
```

### Built-in Tools (Responses API)

```typescript
// Web Search Tool
const result = await generateText({
  model: openai('gpt-5'),
  prompt: 'What happened in San Francisco last week?',
  tools: {
    web_search: openai.tools.webSearch({
      externalWebAccess: true,
      searchContextSize: 'high',
      userLocation: {
        type: 'approximate',
        city: 'San Francisco',
        region: 'California',
      },
    }),
  },
  toolChoice: { type: 'tool', toolName: 'web_search' }, // force usage
});

// File Search Tool
const result = await generateText({
  model: openai('gpt-5'),
  prompt: 'What does the document say about authentication?',
  tools: {
    file_search: openai.tools.fileSearch({
      vectorStoreIds: ['vs_123'],
      maxNumResults: 5,
      filters: { key: 'author', type: 'eq', value: 'Jane Smith' },
      ranking: { ranker: 'auto', scoreThreshold: 0.5 },
    }),
  },
  providerOptions: {
    openai: { include: ['file_search_call.results'] },
  },
});

// Image Generation Tool (gpt-5 variants)
const result = await generateText({
  model: openai('gpt-5'),
  prompt: 'Generate an image of an echidna swimming.',
  tools: {
    image_generation: openai.tools.imageGeneration({
      outputFormat: 'webp',
      quality: 'low',
    }),
  },
});

for (const toolResult of result.staticToolResults) {
  if (toolResult.toolName === 'image_generation') {
    const base64Image = toolResult.output.result;
  }
}

// Code Interpreter Tool
const result = await generateText({
  model: openai('gpt-5'),
  prompt: 'Calculate the factorial of 10',
  tools: {
    code_interpreter: openai.tools.codeInterpreter({
      container: { fileIds: ['file-123', 'file-456'] },
    }),
  },
});

// MCP Tool
const result = await generateText({
  model: openai('gpt-5'),
  prompt: 'Search for AI developments',
  tools: {
    mcp: openai.tools.mcp({
      serverLabel: 'web-search',
      serverUrl: 'https://mcp.exa.ai/mcp',
      serverDescription: 'A web-search API for AI agents',
      allowedTools: ['search', 'summarize'],
      authorization: 'Bearer token',
      headers: { 'X-Custom': 'value' },
    }),
  },
});
```

### Chat API Provider Options

```typescript
import { openai, OpenAIChatLanguageModelOptions } from '@ai-sdk/openai';

await generateText({
  model: openai.chat('gpt-5'),
  prompt: 'Hello',
  providerOptions: {
    openai: {
      logitBias: { '50256': -100 },
      logprobs: true, // or number for top N
      parallelToolCalls: true,
      user: 'test-user',
      reasoningEffort: 'medium',
      maxCompletionTokens: 2048,
      store: true,
      metadata: { custom: 'value' },
      serviceTier: 'priority',
      strictJsonSchema: true,
      textVerbosity: 'low',
      promptCacheKey: 'cache-key',
      promptCacheRetention: '24h',
      safetyIdentifier: 'user-id',

      // Predicted outputs (gpt-4o, gpt-4o-mini)
      prediction: {
        type: 'content',
        content: existingCode,
      },
    } satisfies OpenAIChatLanguageModelOptions,
  },
});

// Access logprobs
const { providerMetadata } = await generateText({ /* ... */ });
const logprobs = providerMetadata?.openai?.logprobs;
```

### Prompt Caching

OpenAI automatically caches prompts ≥1024 tokens (5-10 min TTL, up to 1 hour off-peak):

```typescript
const { text, usage, providerMetadata } = await generateText({
  model: openai.chat('gpt-4o-mini'),
  prompt: 'A 1024-token or longer prompt...',
});

console.log('Cache hits:', providerMetadata?.openai?.cachedPromptTokens);

// Manual cache control
const result = await generateText({
  model: openai.chat('gpt-5'),
  prompt: 'Long prompt...',
  providerOptions: {
    openai: { promptCacheKey: 'my-custom-cache-key-123' },
  },
});

// Extended caching (GPT-5.1, 24h TTL)
const result = await generateText({
  model: openai.chat('gpt-5.1'),
  prompt: 'Long prompt...',
  providerOptions: {
    openai: {
      promptCacheKey: 'cache-key',
      promptCacheRetention: '24h',
    },
  },
});
```

### Image & PDF Inputs

```typescript
// Image input (Chat & Responses API)
const result = await generateText({
  model: openai('gpt-5'),
  messages: [{
    role: 'user',
    content: [
      { type: 'text', text: 'Describe the image.' },
      { type: 'image', image: fs.readFileSync('./image.png') },
      // Or URL: { type: 'image', image: 'https://example.com/img.png' }
      // Or file ID: { type: 'image', image: 'file-8EFBcWHsQxZV7YGezBC1fq' }

      // Image detail control (Chat API only)
      {
        type: 'image',
        image: 'https://example.com/img.png',
        providerOptions: {
          openai: { imageDetail: 'low' }, // 'low' | 'high' | 'auto'
        },
      },
    ],
  }],
});

// PDF input
const result = await generateText({
  model: openai('gpt-5'),
  messages: [{
    role: 'user',
    content: [
      { type: 'text', text: 'What is an embedding model?' },
      {
        type: 'file',
        data: fs.readFileSync('./ai.pdf'),
        mediaType: 'application/pdf',
        filename: 'ai.pdf', // optional
      },
      // Or URL: { type: 'file', data: 'https://example.com/doc.pdf', mediaType: 'application/pdf' }
      // Or file ID: { type: 'file', data: 'file-8EFBcWHsQxZV7YGezBC1fq', mediaType: 'application/pdf' }
    ],
  }],
});
```

### Embeddings

```typescript
import { openai } from '@ai-sdk/openai';
import { embed } from 'ai';

const { embedding } = await embed({
  model: openai.embedding('text-embedding-3-large'),
  value: 'sunny day at the beach',
  providerOptions: {
    openai: {
      dimensions: 512, // custom dimensions (text-embedding-3 only)
      user: 'test-user',
    },
  },
});
```

## Anthropic Provider

### Setup

```typescript
import { anthropic } from '@ai-sdk/anthropic';
import { createAnthropic } from '@ai-sdk/anthropic';

// Default instance (uses ANTHROPIC_API_KEY env var)
const model = anthropic('claude-opus-4-20250514');

// Custom instance
const custom = createAnthropic({
  apiKey: process.env.ANTHROPIC_API_KEY,
  baseURL: 'https://api.anthropic.com/v1',
  headers: { 'Custom-Header': 'value' },
  fetch: customFetchImpl,
});
```

### Claude Model Lineup

```typescript
// Opus: Most capable, complex tasks
anthropic('claude-opus-4-5');
anthropic('claude-opus-4-20250514');

// Sonnet: Balanced performance and speed
anthropic('claude-sonnet-4-5');
anthropic('claude-sonnet-4-20250514');
anthropic('claude-3-7-sonnet-20250219');

// Haiku: Fastest, simple tasks
anthropic('claude-haiku-4-5');
anthropic('claude-3-5-haiku-latest');
```

### Provider Options

```typescript
import { anthropic, AnthropicProviderOptions } from '@ai-sdk/anthropic';

const result = await generateText({
  model: anthropic('claude-opus-4-20250514'),
  prompt: 'Explain quantum computing',
  providerOptions: {
    anthropic: {
      // Disable parallel tool calls
      disableParallelToolUse: false,

      // Include reasoning content in requests
      sendReasoning: true,

      // Effort level (claude-opus-4-5)
      effort: 'high', // 'low' | 'medium' | 'high'

      // Thinking/reasoning
      thinking: {
        type: 'enabled',
        budgetTokens: 12000,
      },

      // Tool streaming
      toolStreaming: true,

      // Structured output mode
      structuredOutputMode: 'auto', // 'outputFormat' | 'jsonTool' | 'auto'
    } satisfies AnthropicProviderOptions,
  },
});
```

### Thinking/Reasoning (Claude Opus 4, Sonnet 4, 3.7)

```typescript
const { text, reasoning, reasoningDetails } = await generateText({
  model: anthropic('claude-opus-4-20250514'),
  prompt: 'How many people will live in the world in 2040?',
  providerOptions: {
    anthropic: {
      thinking: {
        type: 'enabled',
        budgetTokens: 12000, // thinking token budget
      },
    } satisfies AnthropicProviderOptions,
  },
});

console.log('Reasoning:', reasoning); // reasoning text
console.log('Reasoning details:', reasoningDetails); // includes redacted reasoning
console.log('Response:', text);
```

### Prompt Caching (Ephemeral, 24h retention)

Cache content using `providerOptions` on messages or message parts:

```typescript
const errorMessage = '... long error message ...';

const result = await generateText({
  model: anthropic('claude-3-5-sonnet-20240620'),
  messages: [{
    role: 'user',
    content: [
      { type: 'text', text: 'You are a JavaScript expert.' },
      {
        type: 'text',
        text: `Error message: ${errorMessage}`,
        providerOptions: {
          anthropic: { cacheControl: { type: 'ephemeral' } },
        },
      },
      { type: 'text', text: 'Explain the error message.' },
    ],
  }],
});

// Check cached token usage
console.log(result.providerMetadata?.anthropic);
// e.g. { cacheCreationInputTokens: 2118 }

// Cache control on system messages
const result = await generateText({
  model: anthropic('claude-3-5-sonnet-20240620'),
  messages: [
    {
      role: 'system',
      content: 'Cached system message part',
      providerOptions: {
        anthropic: { cacheControl: { type: 'ephemeral' } },
      },
    },
    {
      role: 'system',
      content: 'Uncached system message part',
    },
    { role: 'user', content: 'User prompt' },
  ],
});

// Longer cache TTL (1 hour)
{
  type: 'text',
  text: 'Long cached message',
  providerOptions: {
    anthropic: {
      cacheControl: { type: 'ephemeral', ttl: '1h' },
    },
  },
}

// Cache control on tools
const result = await generateText({
  model: anthropic('claude-3-5-haiku-latest'),
  tools: {
    cityAttractions: tool({
      inputSchema: z.object({ city: z.string() }),
      providerOptions: {
        anthropic: { cacheControl: { type: 'ephemeral' } },
      },
    }),
  },
  messages: [{ role: 'user', content: 'User prompt' }],
});
```

Minimum cacheable lengths:
- Claude Opus 4.5: 4096 tokens
- Claude Opus 4.1, 4, Sonnet 4.5, 4, 3.7, Opus 3: 1024 tokens
- Claude Haiku 4.5: 4096 tokens
- Claude Haiku 3.5, 3: 2048 tokens

### Built-in Tools

```typescript
// Web Search Tool (claude-opus-4, claude-sonnet-4)
const result = await generateText({
  model: anthropic('claude-opus-4-20250514'),
  prompt: 'Latest AI developments',
  tools: {
    web_search: anthropic.tools.webSearch_20250305({
      maxUses: 5,
      allowedDomains: ['techcrunch.com', 'wired.com'],
      blockedDomains: ['example-spam-site.com'],
      userLocation: {
        type: 'approximate',
        country: 'US',
        region: 'California',
        city: 'San Francisco',
        timezone: 'America/Los_Angeles',
      },
    }),
  },
});

// Web Fetch Tool
const result = await generateText({
  model: anthropic('claude-sonnet-4-0'),
  prompt: 'What is this page about? https://en.wikipedia.org/wiki/Maglemosian_culture',
  tools: {
    web_fetch: anthropic.tools.webFetch_20250910({
      maxUses: 1,
      allowedDomains: ['wikipedia.org'],
      blockedDomains: ['ads.example.com'],
      citations: { enabled: true },
      maxContentTokens: 10000,
    }),
  },
});

// Code Execution Tool
const result = await generateText({
  model: anthropic('claude-opus-4-20250514'),
  prompt: 'Calculate the mean and standard deviation of [1, 2, 3, 4, 5]',
  tools: {
    code_execution: anthropic.tools.codeExecution_20250825(),
  },
});

// Tool Search (BM25 or Regex)
const result = await generateText({
  model: anthropic('claude-sonnet-4-5'),
  prompt: 'What is the weather in San Francisco?',
  tools: {
    toolSearch: anthropic.tools.toolSearchBm25_20251119(),
    // or: toolSearchRegex_20251119()

    get_weather: tool({
      description: 'Get the current weather at a location',
      inputSchema: z.object({
        location: z.string().describe('The city and state'),
      }),
      execute: async ({ location }) => ({ /* ... */ }),
      providerOptions: {
        anthropic: { deferLoading: true },
      },
    }),
  },
});
```

### MCP Connectors

```typescript
const result = await generateText({
  model: anthropic('claude-sonnet-4-5'),
  prompt: 'Call the echo tool with "hello world".',
  providerOptions: {
    anthropic: {
      mcpServers: [{
        type: 'url',
        name: 'echo',
        url: 'https://echo.mcp.inevitable.fyi/mcp',
        authorizationToken: mcpAuthToken,
        toolConfiguration: {
          enabled: true,
          allowedTools: ['echo'],
        },
      }],
    } satisfies AnthropicProviderOptions,
  },
});
```

### Agent Skills

```typescript
const result = await generateText({
  model: anthropic('claude-sonnet-4-5'),
  tools: {
    code_execution: anthropic.tools.codeExecution_20250825(),
  },
  prompt: 'Create a presentation about renewable energy with 5 slides',
  providerOptions: {
    anthropic: {
      container: {
        skills: [
          {
            type: 'anthropic', // or 'custom'
            skillId: 'pptx', // Built-in: 'pptx', 'docx', 'pdf', 'xlsx'
            version: 'latest', // optional
          },
        ],
      },
    } satisfies AnthropicProviderOptions,
  },
});
```

## Google Generative AI Provider

### Setup

```typescript
import { google } from '@ai-sdk/google';
import { createGoogleGenerativeAI } from '@ai-sdk/google';

// Default instance (uses GOOGLE_GENERATIVE_AI_API_KEY env var)
const model = google('gemini-2.5-flash');

// Custom instance
const custom = createGoogleGenerativeAI({
  apiKey: process.env.GOOGLE_GENERATIVE_AI_API_KEY,
  baseURL: 'https://generativelanguage.googleapis.com/v1beta',
  headers: { 'Custom-Header': 'value' },
  fetch: customFetchImpl,
});
```

### Gemini Models

```typescript
// Gemini 3 (newest, with thinking levels)
google('gemini-3-pro-preview');

// Gemini 2.5 (production-ready)
google('gemini-2.5-pro');
google('gemini-2.5-flash');
google('gemini-2.5-flash-lite');

// Gemini 2.0
google('gemini-2.0-flash');

// Gemini 1.5
google('gemini-1.5-pro');
google('gemini-1.5-flash');
google('gemini-1.5-flash-8b');
```

### Provider Options

```typescript
import { google, GoogleGenerativeAIProviderOptions } from '@ai-sdk/google';

const result = await generateText({
  model: google('gemini-2.5-flash'),
  prompt: 'Explain quantum computing',
  providerOptions: {
    google: {
      // Cached content
      cachedContent: 'cachedContents/{cachedContent}',

      // Structured outputs
      structuredOutputs: true, // disable to avoid schema limitations

      // Safety settings
      safetySettings: [{
        category: 'HARM_CATEGORY_HATE_SPEECH',
        threshold: 'BLOCK_LOW_AND_ABOVE',
      }],

      // Response modalities
      responseModalities: ['TEXT', 'IMAGE'],

      // Image generation config
      imageConfig: {
        aspectRatio: '16:9', // 1:1, 2:3, 3:2, 3:4, 4:3, 4:5, 5:4, 9:16, 16:9, 21:9
      },
    } satisfies GoogleGenerativeAIProviderOptions,
  },
});

// Access safety ratings
const metadata = result.providerMetadata?.google;
console.log(metadata?.safetyRatings);
console.log(metadata?.usageMetadata);
```

### Thinking Configuration

```typescript
// Gemini 3: thinkingLevel
const { text, reasoning } = await generateText({
  model: google('gemini-3-pro-preview'),
  prompt: 'What is the sum of the first 10 prime numbers?',
  providerOptions: {
    google: {
      thinkingConfig: {
        thinkingLevel: 'high', // 'low' | 'high'
        includeThoughts: true,
      },
    } satisfies GoogleGenerativeAIProviderOptions,
  },
});

console.log('Reasoning:', reasoning);

// Gemini 2.5: thinkingBudget
const { text, reasoning } = await generateText({
  model: google('gemini-2.5-flash'),
  prompt: 'Complex reasoning task',
  providerOptions: {
    google: {
      thinkingConfig: {
        thinkingBudget: 8192, // thinking token budget
        includeThoughts: true,
      },
    } satisfies GoogleGenerativeAIProviderOptions,
  },
});
```

### Built-in Tools

```typescript
// Google Search
const { text, sources, providerMetadata } = await generateText({
  model: google('gemini-2.5-flash'),
  tools: {
    google_search: google.tools.googleSearch({}),
  },
  prompt: 'List the top 5 San Francisco news from the past week.',
});

const metadata = providerMetadata?.google;
console.log(metadata?.groundingMetadata);

// Code Execution
const { text, toolCalls, toolResults } = await generateText({
  model: google('gemini-2.5-pro'),
  tools: {
    code_execution: google.tools.codeExecution({}),
  },
  prompt: 'Use python to calculate the 20th fibonacci number.',
});

// File Search (Gemini 2.5)
const { text, sources } = await generateText({
  model: google('gemini-2.5-pro'),
  tools: {
    file_search: google.tools.fileSearch({
      fileSearchStoreNames: [
        'projects/my-project/locations/us/fileSearchStores/my-store',
      ],
      metadataFilter: 'author = "Robert Graves"',
      topK: 8,
    }),
  },
  prompt: "Summarise the key themes of 'I, Claudius'.",
});

// URL Context (Gemini 2.0+)
const { text, sources, providerMetadata } = await generateText({
  model: google('gemini-2.5-flash'),
  prompt: 'Based on the document: https://ai.google.dev/gemini-api/docs/url-context. Answer: How many links per request?',
  tools: {
    url_context: google.tools.urlContext({}),
  },
});

const urlMetadata = providerMetadata?.google?.urlContextMetadata;
```

### Prompt Caching

Gemini 2.5 has implicit caching (automatic 75% discount on cached content):

```typescript
// Implicit caching (automatic)
const baseContext = 'Long context (1024+ tokens for Flash, 2048+ for Pro)...';

const { text } = await generateText({
  model: google('gemini-2.5-pro'),
  prompt: `${baseContext}\n\nQuestion 1`,
});

const { text: text2, providerMetadata } = await generateText({
  model: google('gemini-2.5-pro'),
  prompt: `${baseContext}\n\nQuestion 2`, // Cache hit!
});

console.log(providerMetadata?.google?.usageMetadata?.cachedContentTokenCount);

// Explicit caching (Gemini 2.5, 2.0)
import { GoogleAICacheManager } from '@google/generative-ai/server';

const cacheManager = new GoogleAICacheManager(process.env.GOOGLE_GENERATIVE_AI_API_KEY);

const { name: cachedContent } = await cacheManager.create({
  model: 'gemini-2.5-pro',
  contents: [{ role: 'user', parts: [{ text: '1000 Lasagna Recipes...' }] }],
  ttlSeconds: 60 * 5,
});

const { text } = await generateText({
  model: google('gemini-2.5-pro'),
  prompt: 'Write a vegetarian lasagna recipe for 4 people.',
  providerOptions: {
    google: { cachedContent },
  },
});
```

## AI Gateway Provider

### Setup

```typescript
import { gateway } from 'ai';
import { createGateway } from 'ai';

// Default instance (uses AI_GATEWAY_API_KEY env var or OIDC)
const { text } = await generateText({
  model: 'openai/gpt-5',
  prompt: 'Hello world',
});

// Or use gateway provider instance
const { text } = await generateText({
  model: gateway('openai/gpt-5'),
  prompt: 'Hello world',
});

// Custom instance
const custom = createGateway({
  apiKey: process.env.AI_GATEWAY_API_KEY,
  baseURL: 'https://ai-gateway.vercel.sh/v1/ai',
  headers: { 'Custom-Header': 'value' },
  fetch: customFetchImpl,
  metadataCacheRefreshMillis: 300000, // 5 minutes
});
```

### OIDC Authentication (Vercel Deployments)

Automatic OIDC authentication on Vercel:

```bash
# Production/Preview: automatic
# Local development:
vercel env pull  # Download OIDC token
vercel dev       # Auto-refresh tokens
```

### BYOK (Bring Your Own Key)

Add provider credentials in Vercel team's AI Gateway settings. No code changes needed.

### Provider Options

```typescript
import type { GatewayProviderOptions } from '@ai-sdk/gateway';

const { text } = await generateText({
  model: 'anthropic/claude-sonnet-4',
  prompt: 'Explain quantum computing',
  providerOptions: {
    gateway: {
      // Provider routing
      order: ['vertex', 'anthropic'], // Try Vertex first
      only: ['vertex', 'anthropic'],  // Limit to these providers

      // Model fallbacks
      models: ['openai/gpt-5-nano', 'gemini-2.0-flash'],

      // Usage tracking
      user: 'user-abc-123', // Track by end-user
      tags: ['document-summary', 'premium-feature'], // Categorize usage
    } satisfies GatewayProviderOptions,
  },
});
```

### Provider-Specific Options Through Gateway

Use actual provider names for provider-specific options:

```typescript
import type { AnthropicProviderOptions } from '@ai-sdk/anthropic';
import type { GatewayProviderOptions } from '@ai-sdk/gateway';

const { text } = await generateText({
  model: 'anthropic/claude-sonnet-4',
  prompt: 'Explain quantum computing',
  providerOptions: {
    gateway: {
      order: ['vertex', 'anthropic'],
    } satisfies GatewayProviderOptions,
    anthropic: {
      thinking: { type: 'enabled', budgetTokens: 12000 },
    } satisfies AnthropicProviderOptions,
  },
});
```

### Dynamic Model Discovery

```typescript
import { gateway } from 'ai';

const availableModels = await gateway.getAvailableModels();

availableModels.models.forEach(model => {
  console.log(`${model.id}: ${model.name}`);
  if (model.pricing) {
    console.log(`  Input: $${model.pricing.input}/token`);
    console.log(`  Output: $${model.pricing.output}/token`);
    if (model.pricing.cachedInputTokens) {
      console.log(`  Cached read: $${model.pricing.cachedInputTokens}/token`);
    }
  }
});

// Use discovered model
const { text } = await generateText({
  model: availableModels.models[0].id,
  prompt: 'Hello world',
});
```

### Credit Usage Tracking

```typescript
import { gateway } from 'ai';

const credits = await gateway.getCredits();

console.log(`Team balance: ${credits.balance} credits`);
console.log(`Team total used: ${credits.total_used} credits`);
```

## Provider Management (v6)

### Custom Provider (Pre-configured Models)

```typescript
import { customProvider, gateway, wrapLanguageModel, defaultSettingsMiddleware } from 'ai';

// Model aliases with custom settings
export const openai = customProvider({
  languageModels: {
    // Replacement model with custom options
    'gpt-5.1': wrapLanguageModel({
      model: gateway('openai/gpt-5.1'),
      middleware: defaultSettingsMiddleware({
        settings: {
          providerOptions: {
            openai: { reasoningEffort: 'high' },
          },
        },
      }),
    }),

    // Alias with custom options
    'gpt-5.1-high-reasoning': wrapLanguageModel({
      model: gateway('openai/gpt-5.1'),
      middleware: defaultSettingsMiddleware({
        settings: {
          providerOptions: {
            openai: { reasoningEffort: 'high' },
          },
        },
      }),
    }),
  },
  fallbackProvider: gateway,
});

// Simple model aliases
export const anthropic = customProvider({
  languageModels: {
    opus: gateway('anthropic/claude-opus-4.1'),
    sonnet: gateway('anthropic/claude-sonnet-4.5'),
    haiku: gateway('anthropic/claude-haiku-4.5'),
  },
  fallbackProvider: gateway,
});

// Limit available models (no fallback)
export const myProvider = customProvider({
  languageModels: {
    'text-medium': gateway('anthropic/claude-3-5-sonnet-20240620'),
    'text-small': gateway('openai/gpt-5-mini'),
  },
  embeddingModels: {
    embedding: gateway.embeddingModel('openai/text-embedding-3-small'),
  },
  // no fallback provider
});
```

### Provider Registry (Multi-provider Apps)

```typescript
import { anthropic } from '@ai-sdk/anthropic';
import { openai } from '@ai-sdk/openai';
import { createProviderRegistry, gateway } from 'ai';

// Create registry
export const registry = createProviderRegistry({
  gateway,
  anthropic,
  openai,
});

// Custom separator
export const customRegistry = createProviderRegistry(
  { gateway, anthropic, openai },
  { separator: ' > ' }
);

// Use language models
const { text } = await generateText({
  model: registry.languageModel('openai:gpt-5.1'),
  // or: customRegistry.languageModel('openai > gpt-5.1')
  prompt: 'Invent a new holiday.',
});

// Use embeddings
const { embedding } = await embed({
  model: registry.embeddingModel('openai:text-embedding-3-small'),
  value: 'sunny day',
});

// Use image models
const { image } = await generateImage({
  model: registry.imageModel('openai:dall-e-3'),
  prompt: 'A sunset over the ocean',
});
```

### Combined Example (Custom + Registry + Middleware)

```typescript
import { anthropic, AnthropicProviderOptions } from '@ai-sdk/anthropic';
import { createOpenAICompatible } from '@ai-sdk/openai-compatible';
import { xai } from '@ai-sdk/xai';
import { groq } from '@ai-sdk/groq';
import {
  createProviderRegistry,
  customProvider,
  defaultSettingsMiddleware,
  gateway,
  wrapLanguageModel,
} from 'ai';

export const registry = createProviderRegistry(
  {
    // Pass through gateway
    gateway,

    // Pass through full provider
    xai,

    // Custom OpenAI-compatible provider
    custom: createOpenAICompatible({
      name: 'provider-name',
      apiKey: process.env.CUSTOM_API_KEY,
      baseURL: 'https://api.custom.com/v1',
    }),

    // Model aliases with custom settings
    anthropic: customProvider({
      languageModels: {
        fast: anthropic('claude-haiku-4-5'),
        writing: anthropic('claude-sonnet-4-5'),
        reasoning: wrapLanguageModel({
          model: anthropic('claude-sonnet-4-5'),
          middleware: defaultSettingsMiddleware({
            settings: {
              maxOutputTokens: 100000,
              providerOptions: {
                anthropic: {
                  thinking: {
                    type: 'enabled',
                    budgetTokens: 32000,
                  },
                } satisfies AnthropicProviderOptions,
              },
            },
          }),
        }),
      },
      fallbackProvider: anthropic,
    }),

    // Limited models without fallback
    groq: customProvider({
      languageModels: {
        'gemma2-9b-it': groq('gemma2-9b-it'),
        'qwen-qwq-32b': groq('qwen-qwq-32b'),
      },
    }),
  },
  { separator: ' > ' }
);

// Usage
const model = registry.languageModel('anthropic > reasoning');
```

### Global Provider Configuration

```typescript
// setup.ts (initialize once during startup)
import { openai } from '@ai-sdk/openai';

globalThis.AI_SDK_DEFAULT_PROVIDER = openai;

// app.ts (use without prefix)
import { streamText } from 'ai';

const result = await streamText({
  model: 'gpt-5.1', // Uses global provider (OpenAI)
  prompt: 'Invent a new holiday.',
});
```

## Common Provider Option Patterns

### providerOptions Syntax

```typescript
import type { OpenAIResponsesProviderOptions } from '@ai-sdk/openai';
import type { AnthropicProviderOptions } from '@ai-sdk/anthropic';
import type { GoogleGenerativeAIProviderOptions } from '@ai-sdk/google';
import type { GatewayProviderOptions } from '@ai-sdk/gateway';

const result = await generateText({
  model: 'openai/gpt-5',
  prompt: 'Hello',
  providerOptions: {
    openai: {
      reasoningEffort: 'high',
    } satisfies OpenAIResponsesProviderOptions,

    anthropic: {
      thinking: { type: 'enabled', budgetTokens: 12000 },
    } satisfies AnthropicProviderOptions,

    google: {
      thinkingConfig: { thinkingLevel: 'high' },
    } satisfies GoogleGenerativeAIProviderOptions,

    gateway: {
      order: ['vertex', 'anthropic'],
    } satisfies GatewayProviderOptions,
  },
});
```

### Reasoning/Thinking Across Providers

```typescript
// OpenAI reasoning
providerOptions: {
  openai: {
    reasoningEffort: 'high', // 'none' | 'minimal' | 'low' | 'medium' | 'high'
    reasoningSummary: 'detailed', // 'auto' | 'detailed'
  },
}

// Anthropic thinking
providerOptions: {
  anthropic: {
    thinking: {
      type: 'enabled',
      budgetTokens: 12000,
    },
  },
}

// Google thinking (Gemini 3)
providerOptions: {
  google: {
    thinkingConfig: {
      thinkingLevel: 'high', // 'low' | 'high'
      includeThoughts: true,
    },
  },
}

// Google thinking (Gemini 2.5)
providerOptions: {
  google: {
    thinkingConfig: {
      thinkingBudget: 8192,
      includeThoughts: true,
    },
  },
}
```

### Caching Across Providers

```typescript
// OpenAI prompt caching
providerOptions: {
  openai: {
    promptCacheKey: 'cache-key',
    promptCacheRetention: '24h', // GPT-5.1 only
  },
}

// Anthropic prompt caching
messages: [{
  role: 'user',
  content: [{
    type: 'text',
    text: 'Cached content',
    providerOptions: {
      anthropic: { cacheControl: { type: 'ephemeral', ttl: '1h' } },
    },
  }],
}]

// Google caching
providerOptions: {
  google: {
    cachedContent: 'cachedContents/abc123',
  },
}
```

This reference covers the major provider configuration patterns in AI SDK v6. For specific edge cases and advanced features, consult the individual provider documentation pages.
