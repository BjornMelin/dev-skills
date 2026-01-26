# Tool Integration Reference

Using tools with useChat in AI SDK v6.

## Tool Types

1. **Server-side tools**: Execute on server with `execute` function
2. **Client-side automatic**: Handle in `onToolCall` callback
3. **Client-side interactive**: Render UI for user interaction

## Flow Overview

1. User sends message
2. Model generates tool calls
3. Server tools execute automatically
4. Client receives tool call parts
5. Client-side tools handled via onToolCall or UI
6. addToolOutput provides results
7. sendAutomaticallyWhen triggers next iteration

## Server-Side Tools

```typescript
// app/api/chat/route.ts
import { convertToModelMessages, streamText, UIMessage } from 'ai';
import { z } from 'zod';

export async function POST(req: Request) {
  const { messages }: { messages: UIMessage[] } = await req.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
    tools: {
      getWeather: {
        description: 'Get weather for a city',
        inputSchema: z.object({ city: z.string() }),
        execute: async ({ city }) => {
          // Server-side execution
          const weather = await fetchWeatherAPI(city);
          return { temperature: weather.temp, condition: weather.condition };
        },
      },
    },
  });

  return result.toUIMessageStreamResponse();
}
```

## Client-Side Automatic Tools

```typescript
'use client';
import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport, lastAssistantMessageIsCompleteWithToolCalls } from 'ai';

export default function Chat() {
  const { messages, sendMessage, addToolOutput } = useChat({
    transport: new DefaultChatTransport({ api: '/api/chat' }),

    // Auto-submit when all tool results available
    sendAutomaticallyWhen: lastAssistantMessageIsCompleteWithToolCalls,

    async onToolCall({ toolCall }) {
      // Check for dynamic tools first (type narrowing)
      if (toolCall.dynamic) return;

      if (toolCall.toolName === 'getLocation') {
        // No await - avoids potential deadlocks
        addToolOutput({
          tool: 'getLocation',
          toolCallId: toolCall.toolCallId,
          output: navigator.geolocation ? await getPosition() : 'Unknown',
        });
      }
    },
  });
  // ...
}
```

## Interactive Tools (User Confirmation)

```typescript
// Render tool parts with user interaction
{message.parts.map((part, index) => {
  if (part.type === 'tool-askForConfirmation') {
    const callId = part.toolCallId;

    switch (part.state) {
      case 'input-streaming':
        return <div key={callId}>Loading...</div>;

      case 'input-available':
        return (
          <div key={callId}>
            <p>{part.input.message}</p>
            <button onClick={() => addToolOutput({
              tool: 'askForConfirmation',
              toolCallId: callId,
              output: 'Confirmed',
            })}>
              Yes
            </button>
            <button onClick={() => addToolOutput({
              tool: 'askForConfirmation',
              toolCallId: callId,
              output: 'Denied',
            })}>
              No
            </button>
          </div>
        );

      case 'output-available':
        return <div key={callId}>Result: {part.output}</div>;

      case 'output-error':
        return <div key={callId}>Error: {part.errorText}</div>;
    }
  }
})}
```

## Tool Part States

| State | Description |
|-------|-------------|
| `input-streaming` | Tool input being streamed |
| `input-available` | Tool input complete, waiting for execution |
| `approval-requested` | v6: Tool requires user approval before execution |
| `output-available` | Tool execution complete with output |
| `output-error` | Tool execution failed |

### State Transitions

```
input-streaming → input-available → [approval-requested →] output-available
                                  ↘                        ↗
                                    → output-error ←───────
```

**Standard Flow (no approval):**
`input-streaming` → `input-available` → `output-available` (or `output-error`)

**With `needsApproval: true`:**
`input-streaming` → `input-available` → `approval-requested` → `output-available` (after approval)

> **Note**: There is no `approval-responded` state. After calling `addToolApprovalResponse()`, the tool transitions directly to `output-available` or `output-error`.

## Error Handling

```typescript
async onToolCall({ toolCall }) {
  if (toolCall.dynamic) return;

  if (toolCall.toolName === 'getWeather') {
    try {
      const weather = await fetchWeather(toolCall.input.city);
      addToolOutput({
        tool: 'getWeather',
        toolCallId: toolCall.toolCallId,
        output: weather,
      });
    } catch (err) {
      addToolOutput({
        tool: 'getWeather',
        toolCallId: toolCall.toolCallId,
        state: 'output-error',
        errorText: 'Unable to get weather',
      });
    }
  }
}
```

## Dynamic Tools (MCP, Runtime)

```typescript
{message.parts.map((part, index) => {
  switch (part.type) {
    // Static tools with specific types
    case 'tool-getWeather':
      return <WeatherDisplay part={part} />;

    // Dynamic tools use generic type
    case 'dynamic-tool':
      return (
        <div key={index}>
          <h4>Tool: {part.toolName}</h4>
          {part.state === 'output-available' && (
            <pre>{JSON.stringify(part.output, null, 2)}</pre>
          )}
        </div>
      );
  }
})}
```

## Tool Approval (v6)

Require user approval before tool execution:

### Server Configuration

```typescript
// app/api/chat/route.ts
const result = streamText({
  model: 'openai/gpt-4o',
  messages: await convertToModelMessages(messages),
  tools: {
    deleteFile: {
      description: 'Delete a file',
      inputSchema: z.object({ path: z.string() }),
      needsApproval: true, // Requires user approval
      execute: async ({ path }) => {
        await fs.unlink(path);
        return { deleted: path };
      },
    },
  },
});
```

### Client Approval Handling

```typescript
'use client';
import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport } from 'ai';

export default function Chat() {
  const { messages, addToolApprovalResponse } = useChat({
    transport: new DefaultChatTransport({ api: '/api/chat' }),
  });

  return (
    <>
      {messages.map(message => (
        <div key={message.id}>
          {message.parts.map((part, index) => {
            if (part.type === 'tool-deleteFile') {
              if (part.state === 'approval-requested') {
                return (
                  <div key={index}>
                    <p>Delete {part.input.path}?</p>
                    <button onClick={() => addToolApprovalResponse({
                      id: part.approval.id,
                      approved: true,
                    })}>
                      Approve
                    </button>
                    <button onClick={() => addToolApprovalResponse({
                      id: part.approval.id,
                      approved: false,
                    })}>
                      Deny
                    </button>
                  </div>
                );
              }
              if (part.state === 'output-available') {
                return <div key={index}>Deleted: {part.output.deleted}</div>;
              }
            }
            return null;
          })}
        </div>
      ))}
    </>
  );
}
```

## Multi-Step Server-Side Tools

```typescript
// app/api/chat/route.ts
import { stepCountIs } from 'ai';

const result = streamText({
  model: 'openai/gpt-4o',
  messages: await convertToModelMessages(messages), // v6: now async
  tools: {
    search: { /* ... execute function ... */ },
    analyze: { /* ... execute function ... */ },
  },
  stopWhen: stepCountIs(5), // Allow up to 5 steps
});
```

## Step Boundaries

```typescript
{message.parts.map((part, index) => {
  switch (part.type) {
    case 'step-start':
      // Show step boundaries
      return index > 0 ? <hr key={index} /> : null;
    case 'text':
      return <span key={index}>{part.text}</span>;
    case 'tool-getWeather':
      // ...
  }
})}
```

## Server Error Handling

```typescript
// app/api/chat/route.ts
function errorHandler(error: unknown) {
  if (error instanceof Error) return error.message;
  return JSON.stringify(error);
}

return result.toUIMessageStreamResponse({
  onError: errorHandler,
});
```

## Best Practices

1. **Check dynamic first**: Always check `toolCall.dynamic` before type narrowing
2. **No await on addToolOutput**: Prevents potential deadlocks
3. **Handle all states**: Cover all tool part states including `approval-requested`
4. **Error gracefully**: Use `output-error` state for failures
5. **Use sendAutomaticallyWhen**: Simplifies multi-step flows
6. **Use needsApproval**: For destructive or sensitive operations
