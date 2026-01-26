# Dynamic Tools Reference (AI SDK Core v6+)

Use this guide for building dynamic tools with unknown schemas at compile time, integrating MCP tools, and scaling to large tool sets. Assumes Zod v4.3.5.

---

## 1) When to Use `dynamicTool()`

Use `dynamicTool()` when the input/output types are unknown at build time:

- MCP tools without schemas
- User-defined functions loaded at runtime
- Tools sourced from external databases or registries
- Dynamic tool generation based on user input

Prefer `tool()` for stable, type-safe tools with known schemas.

---

## 2) Dynamic Tool Basics

`dynamicTool()` still requires a schema, but the types are `unknown`. Use `z.unknown()` or a permissive shape and validate inside `execute`. The resulting tool is flagged as `dynamic` in tool calls for type narrowing.

```ts
import { dynamicTool } from 'ai';
import { z } from 'zod';

export const customTool = dynamicTool({
  description: 'Execute a custom user-defined function',
  inputSchema: z.object({
    action: z.string(),
    payload: z.unknown(),
  }),
  execute: async input => {
    const { action, payload } = input as { action: string; payload: unknown };
    return { ok: true, action, payload };
  },
});
```

**Notes**
- Use Zod v4.3.5 for schema typing consistency.
- Use `z.any()` or `z.unknown()` for fully dynamic payloads, then validate at runtime.
- Consider per-action schema maps to validate `payload` before use.

---

## 3) Type-Safe Handling with the `dynamic` Flag

When mixing static and dynamic tools, narrow by the `dynamic` flag:

```ts
const result = await generateText({
  model,
  tools: {
    weather: weatherTool,
    custom: customTool,
  },
  onStepFinish: ({ toolCalls }) => {
    for (const call of toolCalls) {
      if (call.dynamic) {
        // input/output are unknown
        continue;
      }

      if (call.toolName === 'weather') {
        call.input.location; // typed
      }
    }
  },
});
```

---

## 4) Dynamic Tools + Large Tool Sets

- Use `dynamicTool()` when schemas are unknown at compile time.
- Use `activeTools` to avoid sending large tool lists every step.
- Prefer schema definition over discovery for type safety and control.

**Pattern: phase-based tooling**

```ts
prepareStep: async ({ stepNumber }) => {
  if (stepNumber <= 2) return { activeTools: ['search'], toolChoice: 'required' };
  if (stepNumber <= 5) return { activeTools: ['analyze'] };
  return { activeTools: ['summarize'], toolChoice: 'required' };
}
```

---

## 5) Dynamic Tools + MCP Client

MCP tool discovery often results in unknown types. You can either:

1) Use schema discovery (all tools, unknown inputs), or
2) Use schema definition (typed + selective), or
3) Combine MCP tools with local `dynamicTool()` wrappers.

### Local MCP server (stdio)

```ts
import { createMCPClient } from '@ai-sdk/mcp';
import { Experimental_StdioMCPTransport } from '@ai-sdk/mcp/mcp-stdio';
import { generateText } from 'ai';

let client;

try {
  client = await createMCPClient({
    transport: new Experimental_StdioMCPTransport({
      command: 'node',
      args: ['src/stdio/dist/server.js'],
    }),
  });

  const mcpTools = await client.tools(); // schema discovery

  const result = await generateText({
    model,
    tools: {
      ...mcpTools,
      custom: customTool,
    },
    stopWhen: stepCountIs(5),
    prompt: 'Use available tools to answer.',
  });
} finally {
  await client?.close();
}
```

### Typed MCP tools (schema definition)

```ts
import { z } from 'zod';

const mcpTools = await client.tools({
  schemas: {
    'get-weather': {
      inputSchema: z.object({ location: z.string() }),
      outputSchema: z.object({
        temperature: z.number(),
        conditions: z.string(),
      }),
    },
  },
});
```

---

## 6) ToolLoopAgent + Dynamic Tools

```ts
import { ToolLoopAgent } from 'ai';

const agent = new ToolLoopAgent({
  model,
  tools: {
    weather: weatherTool,
    custom: customTool,
  },
  prepareStep: async ({ stepNumber }) => {
    if (stepNumber === 0) {
      return { activeTools: ['custom'], toolChoice: 'required' };
    }
    return {};
  },
});
```

---

## 7) Runtime Validation and Safety

Best practices for dynamic tools:

- Validate `unknown` inputs with per-action schemas.
- Use `needsApproval` for sensitive actions.
- Enforce timeouts via `abortSignal`.
- Log tool calls/results per step.
- Use `inputExamples` to guide models when possible.
- Pass request metadata via `experimental_context` if your tools need per-call context.

---

## 8) Research Tooling Example (Exa + Context7)

Below is a **pattern** for a dynamic research tool that routes to different backends.
Replace the placeholders with your own HTTP clients.

```ts
import { dynamicTool } from 'ai';
import { z } from 'zod';

export const researchTool = dynamicTool({
  description: 'Run research actions (search, crawl, docs).',
  inputSchema: z.object({
    action: z.enum(['exaDeepSearch', 'exaCrawl', 'context7Query']),
    query: z.string().optional(),
    url: z.url().optional(),
  }),
  execute: async input => {
    const { action, query, url } = input as {
      action: 'exaDeepSearch' | 'exaCrawl' | 'context7Query';
      query?: string;
      url?: string;
    };

    switch (action) {
      case 'exaDeepSearch':
        return exaDeepSearch({ query });
      case 'exaCrawl':
        return exaCrawl({ url });
      case 'context7Query':
        return context7Query({ query });
      default:
        return { error: 'Unknown action' };
    }
  },
});
```

**Tip:** If you’re in an agent runtime that exposes `exa.deep_search_exa`, `exa.crawling_exa`, or Context7 tools directly, you can call them in `execute`. Otherwise, call their HTTP APIs.

---

## 9) UI Handling

When using `useChat`, dynamic tools appear as `dynamic-tool` parts. Render them explicitly in your UI for visibility and debugging.
