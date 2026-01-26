# Production Agent Patterns

Best practices for deploying AI agents to production.

> **Note**: ToolLoopAgent defaults to `stopWhen: stepCountIs(20)`. For production, always explicitly set `stopWhen` with appropriate limits and cost controls.

## Token Budget Management

Control costs with token-aware stopping:

```typescript
import { ToolLoopAgent, StopCondition, ToolSet } from 'ai';

const tools = { /* ... */ } satisfies ToolSet;

const budgetExceeded: StopCondition<typeof tools> = ({ steps }) => {
  const totalUsage = steps.reduce(
    (acc, step) => ({
      inputTokens: acc.inputTokens + (step.usage?.inputTokens ?? 0),
      outputTokens: acc.outputTokens + (step.usage?.outputTokens ?? 0),
    }),
    { inputTokens: 0, outputTokens: 0 },
  );

  // Estimate cost (adjust rates per model)
  const costEstimate =
    (totalUsage.inputTokens * 0.01 + totalUsage.outputTokens * 0.03) / 1000;

  return costEstimate > 0.50; // Stop if cost exceeds $0.50
};

const agent = new ToolLoopAgent({
  model: 'anthropic/claude-sonnet-4.5',
  tools,
  stopWhen: [stepCountIs(20), budgetExceeded],
});
```

## Cost-Aware Model Selection

```typescript
const agent = new ToolLoopAgent({
  model: 'openai/gpt-4o-mini', // Start cheap
  tools: { /* ... */ },
  prepareStep: async ({ stepNumber, steps }) => {
    // Calculate running cost
    const totalTokens = steps.reduce(
      (sum, step) => sum + (step.usage?.inputTokens ?? 0) + (step.usage?.outputTokens ?? 0),
      0
    );

    // If early and cheap, try stronger model for complex reasoning
    if (stepNumber > 2 && totalTokens < 5000) {
      return { model: 'anthropic/claude-sonnet-4.5' };
    }

    // If budget is running low, switch to cheaper model
    if (totalTokens > 10000) {
      return { model: 'openai/gpt-4o-mini' };
    }

    return {};
  },
});
```

## Context Window Management

Prune messages to stay within limits:

```typescript
const agent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  tools: { /* ... */ },
  prepareStep: async ({ messages }) => {
    // Estimate token count (rough: 4 chars ≈ 1 token)
    const estimatedTokens = messages.reduce(
      (sum, msg) => sum + (typeof msg.content === 'string' ? msg.content.length / 4 : 100),
      0
    );

    // If approaching limit, summarize older messages
    if (estimatedTokens > 100000) { // 128k context window
      const systemMsg = messages[0];
      const recentMsgs = messages.slice(-10);

      // Summarize middle messages
      const { text: summary } = await generateText({
        model: 'openai/gpt-4o-mini',
        prompt: `Summarize these conversation messages concisely:
${JSON.stringify(messages.slice(1, -10))}`,
      });

      return {
        messages: [
          systemMsg,
          { role: 'system', content: `Previous conversation summary: ${summary}` },
          ...recentMsgs,
        ],
      };
    }

    return {};
  },
});
```

## Error Recovery

Implement retry logic and fallbacks:

```typescript
import { ToolLoopAgent, AI_APICallError } from 'ai';

async function runAgentWithRetry(
  agent: ToolLoopAgent,
  prompt: string,
  maxRetries = 3
) {
  let lastError: Error | null = null;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await agent.generate({ prompt });
    } catch (error) {
      lastError = error as Error;

      if (error instanceof AI_APICallError) {
        // Rate limit - wait and retry
        if (error.statusCode === 429) {
          await sleep(Math.pow(2, attempt) * 1000); // Exponential backoff
          continue;
        }

        // Server error - retry with different model
        if (error.statusCode >= 500) {
          // Try fallback model
          const fallbackAgent = new ToolLoopAgent({
            ...agent.config,
            model: 'openai/gpt-4o', // Fallback
          });
          return await fallbackAgent.generate({ prompt });
        }
      }

      throw error; // Non-retryable error
    }
  }

  throw lastError;
}
```

## Type-Safe UI Integration

Use InferAgentUIMessage for typed messages:

```typescript
import { ToolLoopAgent, InferAgentUIMessage, tool } from 'ai';
import { z } from 'zod';

const myAgent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  tools: {
    getWeather: tool({
      description: 'Get weather',
      inputSchema: z.object({ city: z.string() }),
      execute: async ({ city }) => ({ temp: 72, condition: 'sunny' }),
    }),
  },
});

// Type-safe message type for this specific agent
type MyAgentMessage = InferAgentUIMessage<typeof myAgent>;

// Use in React component
function ChatMessage({ message }: { message: MyAgentMessage }) {
  return (
    <div>
      {message.parts.map((part, i) => {
        switch (part.type) {
          case 'text':
            return <p key={i}>{part.text}</p>;
          case 'tool-getWeather':
            if (part.state === 'output-available') {
              return <WeatherCard key={i} {...part.output} />;
            }
            return <Loading key={i} />;
          default:
            return null;
        }
      })}
    </div>
  );
}
```

## createAgentUIStreamResponse

Stream agent responses to UI:

```typescript
// app/api/chat/route.ts
import { createAgentUIStreamResponse } from 'ai';
import { myAgent } from '@/ai/agents/my-agent';

export async function POST(request: Request) {
  const { messages, options } = await request.json();

  return createAgentUIStreamResponse({
    agent: myAgent,
    messages,
    options,
    onFinish({ steps, usage }) {
      // Log for monitoring
      console.log('Agent completed', {
        stepCount: steps.length,
        totalTokens: usage?.totalTokens,
      });
    },
  });
}
```

## Monitoring and Observability

```typescript
const agent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  tools: { /* ... */ },
  experimental_telemetry: {
    isEnabled: true,
    functionId: 'my-agent',
    metadata: {
      version: '1.0.0',
      environment: process.env.NODE_ENV,
    },
  },
  onStepFinish({ stepNumber, usage, toolCalls }) {
    // Send to monitoring service
    metrics.track('agent_step', {
      stepNumber,
      inputTokens: usage?.inputTokens,
      outputTokens: usage?.outputTokens,
      toolsUsed: toolCalls.map(tc => tc.toolName),
    });
  },
});
```

## Request Timeouts

```typescript
const result = await agent.generate({
  prompt: 'Complex research task...',
  abortSignal: AbortSignal.timeout(60000), // 60 second timeout
});
```

## Best Practices Summary

1. **Budget limits**: Always set token/cost budgets
2. **Step limits**: Use stepCountIs to prevent infinite loops
3. **Fallback models**: Have backup models for failures
4. **Context pruning**: Manage message history size
5. **Retry logic**: Implement exponential backoff
6. **Monitoring**: Log all agent runs with telemetry
7. **Timeouts**: Set request timeouts for all calls
8. **Type safety**: Use InferAgentUIMessage for UI
