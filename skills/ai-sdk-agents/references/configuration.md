# Call Options Configuration Reference

Type-safe runtime inputs to dynamically configure agent behavior.

> Default `stopWhen` is `stepCountIs(20)`. Override explicitly if needed.

## Why Use Call Options

- Add dynamic context (retrieved docs, user preferences, session data)
- Select models dynamically based on request complexity
- Configure tools per request (user location, API keys)
- Customize provider options (reasoning effort, temperature)

## Basic Example

```ts
import { ToolLoopAgent } from 'ai';
import { z } from 'zod';

const supportAgent = new ToolLoopAgent({
  model: 'anthropic/claude-sonnet-4.5',
  callOptionsSchema: z.object({
    userId: z.string(),
    accountType: z.enum(['free', 'pro', 'enterprise']),
  }),
  instructions: 'You are a helpful customer support agent.',
  prepareCall: ({ options, ...settings }) => ({
    ...settings,
    instructions: `${settings.instructions}\n\nUser context:\n- Account type: ${options.accountType}\n- User ID: ${options.userId}`,
  }),
});

const result = await supportAgent.generate({
  prompt: 'How do I upgrade my account?',
  options: { userId: 'user_123', accountType: 'free' },
});
```

## Dynamic Model Selection

```ts
const agent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  callOptionsSchema: z.object({ complexity: z.enum(['simple', 'complex']) }),
  prepareCall: ({ options, ...settings }) => ({
    ...settings,
    model: options.complexity === 'simple'
      ? 'openai/gpt-4o-mini'
      : 'openai/o1-mini',
  }),
});
```

## Dynamic Tool Configuration

```ts
import { openai } from '@ai-sdk/openai';

const newsAgent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  callOptionsSchema: z.object({
    userCity: z.string().optional(),
    userRegion: z.string().optional(),
  }),
  tools: { web_search: openai.tools.webSearch() },
  prepareCall: ({ options, ...settings }) => ({
    ...settings,
    tools: {
      web_search: openai.tools.webSearch({
        searchContextSize: 'low',
        userLocation: {
          type: 'approximate',
          city: options.userCity,
          region: options.userRegion,
          country: 'US',
        },
      }),
    },
  }),
});
```

## Provider-Specific Options

```ts
import { OpenAIProviderOptions } from '@ai-sdk/openai';

const agent = new ToolLoopAgent({
  model: 'openai/o3',
  callOptionsSchema: z.object({
    taskDifficulty: z.enum(['low', 'medium', 'high']),
  }),
  prepareCall: ({ options, ...settings }) => ({
    ...settings,
    providerOptions: {
      openai: {
        reasoningEffort: options.taskDifficulty,
      } satisfies OpenAIProviderOptions,
    },
  }),
});
```

## RAG Pattern (Async prepareCall)

```ts
const ragAgent = new ToolLoopAgent({
  model: 'anthropic/claude-sonnet-4.5',
  callOptionsSchema: z.object({ query: z.string() }),
  prepareCall: async ({ options, ...settings }) => {
    const documents = await vectorSearch(options.query);
    return {
      ...settings,
      instructions: `Answer questions using the following context:\n\n${documents.map(doc => doc.content).join('\n\n')}`,
    };
  },
});
```

## prepareStep vs prepareCall

| Callback | Timing | Use Case |
|----------|--------|----------|
| `prepareCall` | Once before agent starts | Model selection, inject context, validate options |
| `prepareStep` | Before each step | Budget limits, dynamic tools, step-specific context |

### Combined Example

```ts
const agent = new ToolLoopAgent({
  model: 'openai/gpt-4o-mini',
  callOptionsSchema: z.object({ isPremium: z.boolean() }),
  prepareCall: ({ options, ...settings }) => ({
    ...settings,
    model: options.isPremium ? 'openai/gpt-4o' : settings.model,
  }),
  prepareStep: async ({ stepNumber, steps }) => {
    const totalTokens = steps.reduce(
      (acc, s) => acc + (s.usage?.totalTokens ?? 0),
      0
    );

    if (totalTokens > 10000) return { toolChoice: 'none' };
    if (totalTokens > 5000) return { model: 'openai/gpt-4o-mini' };

    return {};
  },
});
```
