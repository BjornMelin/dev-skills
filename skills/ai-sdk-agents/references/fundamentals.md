# AI SDK Agents - Fundamentals

Core concepts for building agents with the ToolLoopAgent class in AI SDK v6+.

## ToolLoopAgent Class

ToolLoopAgent encapsulates model, tools, and loop control into a reusable agent. It runs a reasoning-and-acting loop (multi-step tool calling) and exposes `generate()` / `stream()`.

### Basic Agent Creation

```ts
import { ToolLoopAgent } from 'ai';
import { openai } from '@ai-sdk/openai';

const myAgent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  instructions: 'You are a helpful assistant.',
  tools: {
    // tools here
  },
});
```

### Complete Example

```ts
import { ToolLoopAgent, tool, stepCountIs } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const codeAgent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  instructions: 'You are an expert software engineer.',
  stopWhen: stepCountIs(20),
  tools: {
    runCode: tool({
      description: 'Execute Python code',
      inputSchema: z.object({ code: z.string() }),
      execute: async ({ code }) => ({ output: `ran: ${code.length} chars` }),
    }),
  },
  toolChoice: 'auto',
});
```

## Instructions vs System

- ToolLoopAgent uses `instructions` (renamed from `system` in v6 beta).
- `generateText` / `streamText` still use `system`.

```ts
const agent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  instructions: 'Be concise and use bullets.',
});
```

## Agent Outputs

ToolLoopAgent can return text or structured outputs via `Output`.

```ts
import { Output, ToolLoopAgent } from 'ai';
import { z } from 'zod';

const analysisAgent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  output: Output.object({
    schema: z.object({
      sentiment: z.enum(['positive', 'neutral', 'negative']),
      summary: z.string(),
    }),
  }),
});
```

## Streaming

```ts
const stream = myAgent.stream({ prompt: 'Summarize this report.' });
for await (const chunk of stream.textStream) {
  process.stdout.write(chunk);
}
```

## Type-safe UI Integration

```ts
import { ToolLoopAgent, InferAgentUIMessage } from 'ai';

const myAgent = new ToolLoopAgent({ model, tools });
export type MyAgentUIMessage = InferAgentUIMessage<typeof myAgent>;
```
