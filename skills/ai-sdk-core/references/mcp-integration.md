# MCP Integration Reference

Model Context Protocol (MCP) integration in AI SDK Core v6+ enables dynamic tool discovery, resource access, prompts, and elicitation handling.

---

## 1) Create an MCP Client

Use HTTP transport for production; use stdio for local servers only.

```ts
import { createMCPClient } from '@ai-sdk/mcp';

const mcpClient = await createMCPClient({
  transport: {
    type: 'http',
    url: 'https://your-server.com/mcp',
    headers: { Authorization: 'Bearer my-api-key' },
    authProvider: myOAuthClientProvider,
  },
});
```

### Transports

**HTTP (recommended)**

```ts
const mcpClient = await createMCPClient({
  transport: { type: 'http', url: 'https://api.example.com/mcp' },
});
```

**SSE (alternative HTTP)**

```ts
const mcpClient = await createMCPClient({
  transport: { type: 'sse', url: 'https://api.example.com/sse' },
});
```

**Stdio (local only, Node.js)**

```ts
import { Experimental_StdioMCPTransport } from '@ai-sdk/mcp/mcp-stdio';

const mcpClient = await createMCPClient({
  transport: new Experimental_StdioMCPTransport({
    command: 'node',
    args: ['src/stdio/dist/server.js'],
  }),
});
```

You can also use the MCP SDK transports (`StdioClientTransport`,
`StreamableHTTPClientTransport`, `SSEClientTransport`) if preferred.

### Close the client

```ts
let client;

try {
  client = await createMCPClient({ transport: { type: 'http', url } });
  const tools = await client.tools();
  await generateText({ model, tools, prompt });
} finally {
  await client?.close();
}
```

For streaming, close in `onFinish`.

---

## 2) Load MCP Tools

### Schema discovery (load all tools)

```ts
const tools = await mcpClient.tools();
```

### Schema definition (typed + selective)

```ts
import { z } from 'zod';

const tools = await mcpClient.tools({
  schemas: {
    'get-weather': {
      inputSchema: z.object({ location: z.string() }),
      outputSchema: z.object({ temperature: z.number(), conditions: z.string() }),
    },
    'tool-with-no-args': { inputSchema: z.object({}) },
  },
});
```

**Tip:** Use schema definition to load only what you need and keep the tool list small. Use `activeTools` to further limit tools per request/step.

### Typed output with `outputSchema`

When MCP tools return `structuredContent`, `outputSchema` validates and types the result.
If `structuredContent` is missing, the client tries to parse JSON from the text content.

---

## 3) Use MCP Tools in AI SDK Calls

```ts
const tools = await mcpClient.tools();

const result = await generateText({
  model,
  tools,
  stopWhen: stepCountIs(5),
  prompt: 'What is the weather in Paris?'
});
```

To combine MCP tools with local tools:

```ts
const tools = {
  localTool,
  ...(await mcpClient.tools()),
};
```

---

## 4) Dynamic Tools + Large Tool Sets

- Use `dynamicTool()` when schemas are unknown at compile time.
- Use `activeTools` to avoid sending large tool lists every step.
- Prefer schema definition over discovery for type safety and control.

---

## 5) Resources

```ts
const resources = await mcpClient.listResources();
const resource = await mcpClient.readResource({ uri: 'file:///path/to/doc.txt' });
const templates = await mcpClient.listResourceTemplates();
```

Resources are **app-driven**; decide when to fetch and how to pass them as context.

---

## 6) Prompts (Experimental)

```ts
const prompts = await mcpClient.experimental_listPrompts();
const prompt = await mcpClient.experimental_getPrompt({
  name: 'code_review',
  arguments: { code: 'function add(a,b){return a+b;}' },
});

const result = await generateText({ model, messages: prompt.messages });
```

---

## 7) Elicitation

Enable elicitation and register a handler:

```ts
import { ElicitationRequestSchema } from '@ai-sdk/mcp';

const client = await createMCPClient({
  transport: { type: 'sse', url: 'https://server.com/sse' },
  capabilities: { elicitation: {} },
});

client.onElicitationRequest(ElicitationRequestSchema, async request => {
  const { message, requestedSchema } = request.params;
  const userInput = await getInputFromUser(message, requestedSchema);
  return { action: 'accept', content: userInput };
});
```

Actions: `accept` (with content), `decline`, or `cancel`.

---

## 8) Notes and Caveats

- The MCP client is lightweight and does not support notifications or resumable streams.
- Stdio transport is local-only and not suitable for production.
