# AI SDK Agents v6 Migration Guide

Migrate from AI SDK v6 beta to v6 stable release.

## Breaking Changes Summary

| Before (v6 Beta) | After (v6 Stable) |
|------------------|-------------------|
| `Experimental_Agent` | `ToolLoopAgent` |
| `system` parameter | `instructions` |
| Default `stepCountIs(1)` | Default `stepCountIs(20)` |
| `Experimental_InferAgentUIMessage` | `InferAgentUIMessage` |
| `Experimental_AgentSettings` | `ToolLoopAgentSettings` |

## Automated Migration

### Step 1: Run Codemod

```bash
npx @ai-sdk/codemod v6
```

Note: The codemod handles some type renames but NOT the agent class rename (`Experimental_Agent` to `ToolLoopAgent`).

### Step 2: Update Packages

```bash
pnpm add ai@^6.0.3
```

### Step 3: Manual Fixes (Required)

#### Agent Class Rename

```typescript
// Before
import { Experimental_Agent } from 'ai';
const agent = new Experimental_Agent({ ... });

// After
import { ToolLoopAgent } from 'ai';
const agent = new ToolLoopAgent({ ... });
```

#### Parameter Rename: system to instructions

```typescript
// Before
new Experimental_Agent({
  system: 'You are a helpful assistant.',
  model: openai('gpt-4o'),
  tools: { ... },
});

// After
new ToolLoopAgent({
  instructions: 'You are a helpful assistant.',
  model: openai('gpt-4o'),
  tools: { ... },
});
```

Note: The `system` parameter in `generateText()` and `streamText()` is unchanged. This rename only affects the `ToolLoopAgent` class.

#### Default stopWhen Changed

```typescript
// v6 Beta default: stepCountIs(1) - single step only
// v6 Stable default: stepCountIs(20) - multi-step agent loop

// If you relied on single-step behavior, explicitly set:
new ToolLoopAgent({
  model: openai('gpt-4o'),
  stopWhen: stepCountIs(1), // Restore v6 beta behavior
  tools: { ... },
});
```

This change reflects that agents are designed for multi-step reasoning. Most agents need multiple steps to call tools and generate responses.

#### Type Renames

```typescript
// Before
import {
  Experimental_AgentSettings,
  Experimental_InferAgentUIMessage
} from 'ai';

type MySettings = Experimental_AgentSettings;
type MyMessage = Experimental_InferAgentUIMessage<typeof myAgent>;

// After
import {
  ToolLoopAgentSettings,
  InferAgentUIMessage
} from 'ai';

type MySettings = ToolLoopAgentSettings;
type MyMessage = InferAgentUIMessage<typeof myAgent>;
```

### Step 4: Search/Replace Patterns

Run these replacements in your codebase:

| Search | Replace |
|--------|---------|
| `Experimental_Agent` | `ToolLoopAgent` |
| `system:` (in agent constructors) | `instructions:` |
| `Experimental_InferAgentUIMessage` | `InferAgentUIMessage` |
| `Experimental_AgentSettings` | `ToolLoopAgentSettings` |

Regex patterns for IDE search:

```regex
# Agent class rename
Experimental_Agent\b -> ToolLoopAgent

# System to instructions (in object literals)
(\s+)system:\s*(['"`]) -> $1instructions: $2

# Type renames
Experimental_InferAgentUIMessage -> InferAgentUIMessage
Experimental_AgentSettings -> ToolLoopAgentSettings
```

### Step 5: Type Check

```bash
pnpm type-check
```

Verify no TypeScript errors remain after migration.

## Complete Migration Example

### Before (v6 Beta)

```typescript
import { Experimental_Agent, Experimental_InferAgentUIMessage, stepCountIs, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const myAgent = new Experimental_Agent({
  model: openai('gpt-4o'),
  system: 'You are a helpful assistant.',
  stopWhen: stepCountIs(5),
  tools: {
    search: tool({
      description: 'Search the web',
      inputSchema: z.object({ query: z.string() }),
      execute: async ({ query }) => ({ results: [] }),
    }),
  },
});

export type MyMessage = Experimental_InferAgentUIMessage<typeof myAgent>;

const result = await myAgent.generate({
  prompt: 'Search for AI news',
});
```

### After (v6 Stable)

```typescript
import { ToolLoopAgent, InferAgentUIMessage, stepCountIs, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

const myAgent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  instructions: 'You are a helpful assistant.',
  stopWhen: stepCountIs(5),
  tools: {
    search: tool({
      description: 'Search the web',
      inputSchema: z.object({ query: z.string() }),
      execute: async ({ query }) => ({ results: [] }),
    }),
  },
});

export type MyMessage = InferAgentUIMessage<typeof myAgent>;

const result = await myAgent.generate({
  prompt: 'Search for AI news',
});
```

## New v6 Stable Features

### ToolLoopAgent Constructor Options

```typescript
new ToolLoopAgent({
  // Required
  model: LanguageModel,

  // Optional - Agent behavior
  instructions: string,                    // Renamed from 'system'
  tools: Record<string, Tool>,
  stopWhen: StopCondition,                 // Default: stepCountIs(20)
  toolChoice: 'auto' | 'required' | 'none' | { type: 'tool', toolName: string },
  output: Output,                          // Output.object(), Output.array(), etc.
  activeTools: string[],

  // Optional - Dynamic configuration
  prepareStep: PrepareStepFunction,
  prepareCall: PrepareCallFunction,
  callOptionsSchema: ZodSchema,

  // Optional - Callbacks
  onStepFinish: Callback,
  onFinish: Callback,

  // Optional - Model settings
  temperature: number,
  maxOutputTokens: number,
  // ... other LanguageModelSettings
});
```

### hasToolCall Stop Condition

```typescript
import { ToolLoopAgent, hasToolCall, stepCountIs } from 'ai';

const agent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  tools: {
    search: searchTool,
    submit: submitTool,
  },
  // Stop when submit tool is called OR after 20 steps
  stopWhen: [stepCountIs(20), hasToolCall('submit')],
});
```

### Custom Stop Condition Signature

```typescript
import { StopCondition, ToolSet } from 'ai';

const tools = { ... } satisfies ToolSet;

const customStop: StopCondition<typeof tools> = ({ steps, stepNumber }) => {
  // Access typed step information
  const hasFoundAnswer = steps.some(step =>
    step.text?.includes('FINAL ANSWER:')
  );
  return hasFoundAnswer;
};
```

### MCP Integration (Now Stable)

```typescript
import { createMCPClient } from '@ai-sdk/mcp';

// HTTP transport
const mcpClient = createMCPClient({
  transport: {
    type: 'http',
    url: 'https://mcp-server.example.com',
  },
});

// SSE transport with OAuth
const mcpClient = createMCPClient({
  transport: {
    type: 'sse',
    url: 'https://mcp-server.example.com/sse',
    headers: {
      Authorization: `Bearer ${accessToken}`,
    },
  },
});

// Use MCP tools in agent
const agent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  tools: await mcpClient.tools(),
});
```

## Troubleshooting

### Agent runs forever / too many steps

The default changed from `stepCountIs(1)` to `stepCountIs(20)`. If your agent previously ran once and stopped, it may now run multiple times.

**Fix**: Explicitly set `stopWhen: stepCountIs(1)` or adjust to appropriate limit.

### Type errors with InferAgentUIMessage

Ensure you're importing from `'ai'` not a subpath:

```typescript
// Correct
import { InferAgentUIMessage } from 'ai';

// Incorrect
import { InferAgentUIMessage } from 'ai/react';
```

### "system" property does not exist

The `system` parameter was renamed to `instructions` for `ToolLoopAgent` only.

**Fix**: Replace `system:` with `instructions:` in agent constructors.

## Verification Checklist

- [ ] Updated `ai` package to `^6.0.3`
- [ ] Ran `npx @ai-sdk/codemod v6`
- [ ] Replaced `Experimental_Agent` with `ToolLoopAgent`
- [ ] Replaced `system` with `instructions` in agent constructors
- [ ] Replaced `Experimental_InferAgentUIMessage` with `InferAgentUIMessage`
- [ ] Replaced `Experimental_AgentSettings` with `ToolLoopAgentSettings`
- [ ] Reviewed `stopWhen` defaults (now 20 steps instead of 1)
- [ ] Ran `pnpm type-check` with no errors
- [ ] Tested agent behavior in development
