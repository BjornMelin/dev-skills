# AI SDK Core Tool Calling Reference

Use this reference to define tools, run multi-step tool loops, handle approvals, and integrate dynamic/MCP tools in AI SDK Core v6+.

---

## 1) Define Tools

### Use `tool()` for typed inputs

```ts
import { tool } from 'ai';
import { z } from 'zod';

export const weatherTool = tool({
  description: 'Get the weather in a location',
  inputSchema: z.object({
    location: z.string().describe('City and state'),
    units: z.enum(['c', 'f']).optional(),
  }),
  strict: true,
  inputExamples: [{ input: { location: 'San Francisco', units: 'f' } }],
  execute: async ({ location, units = 'f' }) => ({
    location,
    units,
    temperature: 72,
  }),
});
```

**Key fields**
- `description`: Influence model selection.
- `inputSchema`: Zod or JSON Schema; used for tool-call validation.
- `inputExamples`: Optional hints for providers that support them.
- `strict`: Enforce schema-conformant calls (provider-dependent).
- `execute`: Optional async handler. If omitted, tool calls are surfaced but not executed automatically.
- `outputSchema`: Optional output validation (useful for typed results).
- `toModelOutput`: Map tool outputs to LLM content (e.g., multimodal).
- `needsApproval`: Require approval before execution (boolean or async).
- `onInputStart` / `onInputDelta` / `onInputAvailable`: Tool input lifecycle hooks (streaming).

### Use `dynamicTool()` for runtime schemas

```ts
import { dynamicTool } from 'ai';
import { z } from 'zod';

const customTool = dynamicTool({
  description: 'Execute a custom function',
  inputSchema: z.object({}),
  execute: async input => {
    const { action, parameters } = input as { action: string; parameters?: unknown };
    return { result: `Executed ${action}`, parameters };
  },
});
```

**When to use**
- MCP tools without compile-time schemas
- User-defined functions created at runtime
- Tools loaded from external sources

Type-narrow dynamic tool calls via the `dynamic` flag:

```ts
const result = await generateText({ model, tools: { weather: weatherTool, custom: customTool } });

for (const call of result.toolCalls) {
  if (call.dynamic) {
    // input is unknown
    continue;
  }

  if (call.toolName === 'weather') {
    call.input.location; // typed
  }
}
```

---

## 2) Call Tools with `generateText` / `streamText`

```ts
import { generateText, tool } from 'ai';
import { z } from 'zod';

const result = await generateText({
  model,
  tools: {
    weather: tool({
      description: 'Get weather',
      inputSchema: z.object({ location: z.string() }),
      execute: async ({ location }) => ({ location, temperature: 72 }),
    }),
  },
  prompt: 'What is the weather in SF?',
});
```

### Tool choice

```ts
// 'auto' | 'required' | 'none' | { type: 'tool', toolName }
toolChoice: 'required'
```

### Active tools (limit tool set per call/step)

```ts
activeTools: ['weather']
```

---

## 3) Multi-Step Tool Loops

Enable multi-step loops with `stopWhen`:

```ts
import { generateText, stepCountIs, hasToolCall } from 'ai';

const result = await generateText({
  model,
  tools,
  stopWhen: [stepCountIs(5), hasToolCall('finalAnswer')],
  prompt: 'Research and summarize.',
});
```

Access step-by-step data:

```ts
const { steps } = result;
const allToolCalls = steps.flatMap(step => step.toolCalls);
```

### `onStepFinish`

```ts
const result = await generateText({
  model,
  tools,
  onStepFinish({ text, toolCalls, toolResults, finishReason, usage }) {
    // persist history, log usage, etc.
  },
});
```

### `prepareStep` (per-step control)

```ts
const result = await generateText({
  model,
  tools,
  prepareStep: async ({ stepNumber, steps, messages }) => {
    if (stepNumber === 0) {
      return {
        toolChoice: { type: 'tool', toolName: 'search' },
        activeTools: ['search'],
      };
    }

    // prompt compression example
    if (messages.length > 20) {
      return { messages: messages.slice(-10) };
    }

    return {};
  },
});
```

---

## 4) Tool Execution Options

Tools receive execution options as the second parameter:

```ts
execute: async (input, { toolCallId, messages, abortSignal, experimental_context }) => {
  // use toolCallId for status streaming
  // use messages for context-aware operations
  // forward abortSignal to fetch
  // use experimental_context for app-specific data
}
```

Pass `experimental_context` when invoking:

```ts
await generateText({
  model,
  tools,
  experimental_context: { requestId: 'req_123' },
});
```

---

## 5) Tool Approval Flow (`needsApproval`)

```ts
const runCommand = tool({
  description: 'Run a shell command',
  inputSchema: z.object({ command: z.string() }),
  needsApproval: true,
  execute: async ({ command }) => ({ ok: true, command }),
});
```

**Flow**
1) Call `generateText` / `streamText`.
2) Receive `tool-approval-request` parts in `result.content`.
3) Collect approval, then add `tool-approval-response` to messages.
4) Call the model again; tool executes if approved.

```ts
import { type ToolApprovalResponse } from 'ai';

const approvals: ToolApprovalResponse[] = [];
for (const part of result.content) {
  if (part.type === 'tool-approval-request') {
    approvals.push({
      type: 'tool-approval-response',
      approvalId: part.approvalId,
      approved: true,
      reason: 'User confirmed',
    });
  }
}

messages.push({ role: 'tool', content: approvals });
```

**Dynamic approval**

```ts
needsApproval: async ({ amount }) => amount > 1000
```

---

## 6) Tool Input Lifecycle Hooks (Streaming)

```ts
tool({
  inputSchema: z.object({ location: z.string() }),
  execute: async ({ location }) => ({ location }),
  onInputStart: () => console.log('start'),
  onInputDelta: ({ inputTextDelta }) => console.log(inputTextDelta),
  onInputAvailable: ({ input }) => console.log(input),
});
```

---

## 7) Preliminary Tool Results (AsyncIterable)

```ts
tool({
  inputSchema: z.object({ location: z.string() }),
  async *execute({ location }) {
    yield { status: 'loading', location };
    await new Promise(r => setTimeout(r, 3000));
    yield { status: 'success', location, temperature: 72 };
  },
});
```

The final yielded value is the tool result sent back to the model.

---

## 8) Multimodal Tool Results

Use `toModelOutput` to convert tool outputs to LLM-compatible content:

```ts
tool({
  inputSchema: z.object({ action: z.enum(['screenshot']) }),
  execute: async () => ({
    type: 'image',
    data: fs.readFileSync('./screenshot.png').toString('base64'),
  }),
  toModelOutput({ output }) {
    return {
      type: 'content',
      value: [{ type: 'media', data: output.data, mediaType: 'image/png' }],
    };
  },
});
```

---

## 9) Errors and Repair

### Tool schema errors
- `NoSuchToolError`
- `InvalidToolInputError`
- `ToolCallRepairError`

```ts
try {
  await generateText({ model, tools, prompt });
} catch (error) {
  if (NoSuchToolError.isInstance(error)) {
    // handle missing tool
  }
}
```

### Tool execution errors
Tool execution errors appear in `steps` as `tool-error` parts:

```ts
const toolErrors = steps.flatMap(step =>
  step.content.filter(part => part.type === 'tool-error')
);
```

### Tool call repair (experimental)

```ts
import { generateObject, generateText, NoSuchToolError } from 'ai';

await generateText({
  model,
  tools,
  prompt,
  experimental_repairToolCall: async ({ toolCall, tools, inputSchema, error }) => {
    if (NoSuchToolError.isInstance(error)) return null;

    const tool = tools[toolCall.toolName as keyof typeof tools];
    const { object } = await generateObject({
      model: repairModel,
      schema: tool.inputSchema,
      prompt: `Fix tool input: ${JSON.stringify(toolCall.input)}`,
    });

    return { ...toolCall, input: JSON.stringify(object) };
  },
});
```

---

## 10) Types and Message History

```ts
import { TypedToolCall, TypedToolResult } from 'ai';

type MyToolCall = TypedToolCall<typeof tools>;
type MyToolResult = TypedToolResult<typeof tools>;
```

Append assistant/tool messages for multi-step conversations:

```ts
const { response } = await generateText({ model, tools, messages });
messages.push(...response.messages);
```
