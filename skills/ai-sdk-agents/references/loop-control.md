# Loop Control Reference

Use stop conditions and per-step configuration to control ToolLoopAgent execution.

## Stop Conditions

- `stepCountIs(n)` - Stop after n steps (default: 20)
- `hasToolCall('toolName')` - Stop when a specific tool is called

```ts
import { ToolLoopAgent, stepCountIs, hasToolCall } from 'ai';

const agent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  tools: { /* ... */ },
  stopWhen: [stepCountIs(20), hasToolCall('finalAnswer')],
});
```

### Custom Stop Condition

```ts
import type { StopCondition, ToolSet } from 'ai';

const tools = { /* ... */ } satisfies ToolSet;

const budgetExceeded: StopCondition<typeof tools> = ({ steps }) => {
  const totalTokens = steps.reduce((acc, s) => acc + (s.usage?.totalTokens ?? 0), 0);
  return totalTokens > 10000;
};
```

## prepareStep

`prepareStep` runs before each step and can override settings.

### Model Switching

```ts
prepareStep: async ({ stepNumber, messages }) => {
  if (stepNumber > 2 && messages.length > 10) {
    return { model: 'anthropic/claude-sonnet-4.5' };
  }
  return {};
}
```

### Context Management

```ts
prepareStep: async ({ messages }) => {
  if (messages.length > 20) {
    return { messages: messages.slice(-10) };
  }
  return {};
}
```

### Phase-Based Tooling

```ts
prepareStep: async ({ stepNumber }) => {
  if (stepNumber <= 2) return { activeTools: ['search'], toolChoice: 'required' };
  if (stepNumber <= 5) return { activeTools: ['analyze'] };
  return { activeTools: ['summarize'], toolChoice: 'required' };
}
```

### Force Specific Tool

```ts
prepareStep: async ({ stepNumber }) => {
  if (stepNumber === 0) return { toolChoice: { type: 'tool', toolName: 'search' } };
  if (stepNumber === 5) return { toolChoice: { type: 'tool', toolName: 'summarize' } };
  return {};
}
```
