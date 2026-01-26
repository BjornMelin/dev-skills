# AI SDK Integration

Complete guide for integrating Streamdown with AI SDK v6 for streaming chat applications.

## Client Setup

### Basic useChat Integration

```tsx
'use client';
import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport } from 'ai';
import { useState } from 'react';
import { Streamdown } from 'streamdown';

export default function Chat() {
  const { messages, sendMessage, status } = useChat({
    transport: new DefaultChatTransport({ api: '/api/chat' }),
  });
  const [input, setInput] = useState('');

  return (
    <div className="flex flex-col h-screen">
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.map(message => (
          <div
            key={message.id}
            className={message.role === 'user' ? 'text-right' : 'text-left'}
          >
            <div className="inline-block max-w-2xl">
              {message.parts
                .filter(part => part.type === 'text')
                .map((part, index) => (
                  <Streamdown
                    key={index}
                    isAnimating={status === 'streaming'}
                  >
                    {part.text}
                  </Streamdown>
                ))}
            </div>
          </div>
        ))}
      </div>

      <form
        onSubmit={e => {
          e.preventDefault();
          if (input.trim()) {
            sendMessage({ text: input });
            setInput('');
          }
        }}
        className="p-4 border-t"
      >
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          disabled={status !== 'ready'}
          placeholder="Type a message..."
          className="w-full px-4 py-2 border rounded-lg"
        />
      </form>
    </div>
  );
}
```

## Status Handling

### Status Values

The `status` from useChat indicates the current state:

| Status | Description | isAnimating |
|--------|-------------|-------------|
| `'submitted'` | Request sent, waiting for response | `true` |
| `'streaming'` | Receiving streamed response | `true` |
| `'ready'` | Idle, ready for new messages | `false` |
| `'error'` | An error occurred | `false` |

### Status-Based Rendering

```tsx
const { status, error, stop, reload } = useChat();

// Loading indicator
{(status === 'submitted' || status === 'streaming') && (
  <div className="flex items-center gap-2">
    {status === 'submitted' && <Spinner />}
    <button onClick={stop}>Stop</button>
  </div>
)}

// Error handling
{error && (
  <div className="text-destructive">
    <p>Error: {error.message}</p>
    <button onClick={reload}>Retry</button>
  </div>
)}

// Streamdown with status
<Streamdown isAnimating={status === 'streaming' || status === 'submitted'}>
  {content}
</Streamdown>
```

## Message Parts Rendering

### AI SDK v6 Message Structure

```typescript
interface UIMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  parts: MessagePart[];
  metadata?: Record<string, unknown>;
}

type MessagePart =
  | { type: 'text'; text: string }
  | { type: 'file'; filename: string; mediaType: string; url: string }
  | { type: 'tool-invocation'; toolName: string; input: unknown; result?: unknown }
  | { type: 'reasoning'; text: string }
  | { type: 'source-url'; id: string; url: string; title?: string };
```

### Rendering Different Part Types

```tsx
function MessageContent({ message, isStreaming }: { message: UIMessage; isStreaming: boolean }) {
  return (
    <>
      {message.parts.map((part, index) => {
        switch (part.type) {
          case 'text':
            return (
              <Streamdown key={index} isAnimating={isStreaming}>
                {part.text}
              </Streamdown>
            );

          case 'reasoning':
            return (
              <details key={index} className="text-muted-foreground">
                <summary>Reasoning</summary>
                <Streamdown isAnimating={isStreaming}>{part.text}</Streamdown>
              </details>
            );

          case 'source-url':
            return (
              <a key={index} href={part.url} className="text-primary underline">
                {part.title ?? 'Source'}
              </a>
            );

          case 'tool-invocation':
            return (
              <div key={index} className="border rounded p-2 bg-muted">
                <span className="font-mono text-sm">{part.toolName}</span>
                {part.result && <pre>{JSON.stringify(part.result, null, 2)}</pre>}
              </div>
            );

          default:
            return null;
        }
      })}
    </>
  );
}
```

## Memoization Pattern

### Memoized Response Component

Wrap Streamdown in React.memo to prevent unnecessary re-renders:

```tsx
import { memo, type ComponentProps } from 'react';
import { Streamdown } from 'streamdown';
import { cn } from '@/lib/utils';

type ResponseProps = ComponentProps<typeof Streamdown>;

export const Response = memo(
  ({
    className,
    controls = { code: true, table: true, mermaid: true },
    mode = 'streaming',
    shikiTheme = ['github-light', 'github-dark'],
    ...props
  }: ResponseProps) => (
    <Streamdown
      className={cn(
        'prose dark:prose-invert max-w-none',
        '[&>*:first-child]:mt-0 [&>*:last-child]:mb-0',
        className
      )}
      controls={controls}
      mode={mode}
      shikiTheme={shikiTheme}
      {...props}
    />
  )
);

Response.displayName = 'Response';
```

### Usage with Memoized Component

```tsx
import { Response } from '@/components/ai-elements/response';

{messages.map(message => (
  <div key={message.id}>
    {message.parts
      .filter(part => part.type === 'text')
      .map((part, index) => (
        <Response key={index} isAnimating={status === 'streaming'}>
          {part.text}
        </Response>
      ))}
  </div>
))}
```

## Server Setup

### Route Handler (Next.js App Router)

```typescript
// app/api/chat/route.ts
import { convertToModelMessages, streamText, UIMessage } from 'ai';
import { openai } from '@ai-sdk/openai';

export const maxDuration = 30;

export async function POST(req: Request) {
  const { messages }: { messages: UIMessage[] } = await req.json();

  const result = streamText({
    model: openai('gpt-4o'),
    system: 'You are a helpful assistant.',
    messages: convertToModelMessages(messages),
  });

  return result.toUIMessageStreamResponse();
}
```

### With Tools

```typescript
import { convertToModelMessages, streamText, UIMessage, tool } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

export async function POST(req: Request) {
  const { messages }: { messages: UIMessage[] } = await req.json();

  const result = streamText({
    model: openai('gpt-4o'),
    system: 'You are a helpful assistant with access to tools.',
    messages: convertToModelMessages(messages),
    tools: {
      weather: tool({
        description: 'Get weather for a location',
        inputSchema: z.object({ location: z.string() }),
        execute: async ({ location }) => {
          return { temperature: 72, condition: 'sunny', location };
        },
      }),
    },
  });

  return result.toUIMessageStreamResponse();
}
```

### With Reasoning and Sources

```typescript
const result = streamText({
  model: anthropic('claude-sonnet-4'),
  messages: convertToModelMessages(messages),
  providerOptions: {
    anthropic: {
      thinking: { type: 'enabled', budgetTokens: 10000 },
    },
  },
});

return result.toUIMessageStreamResponse({
  sendReasoning: true,   // Include reasoning parts
  sendSources: true,     // Include source parts (Perplexity, Google)
});
```

## Complete Example

### Client Component

```tsx
// app/chat/page.tsx
'use client';
import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport } from 'ai';
import { useState } from 'react';
import { Response } from '@/components/ai-elements/response';

export default function ChatPage() {
  const { messages, sendMessage, status, error, stop, reload } = useChat({
    transport: new DefaultChatTransport({ api: '/api/chat' }),
  });
  const [input, setInput] = useState('');

  const isLoading = status === 'submitted' || status === 'streaming';

  return (
    <div className="flex flex-col h-screen max-w-3xl mx-auto">
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {messages.map(message => (
          <div
            key={message.id}
            className={cn(
              'flex',
              message.role === 'user' ? 'justify-end' : 'justify-start'
            )}
          >
            <div
              className={cn(
                'max-w-[80%] rounded-lg px-4 py-2',
                message.role === 'user'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted'
              )}
            >
              {message.parts
                .filter(part => part.type === 'text')
                .map((part, index) => (
                  <Response
                    key={index}
                    isAnimating={isLoading && message.role === 'assistant'}
                  >
                    {part.text}
                  </Response>
                ))}
            </div>
          </div>
        ))}

        {isLoading && status === 'submitted' && (
          <div className="flex justify-start">
            <div className="bg-muted rounded-lg px-4 py-2">
              <span className="animate-pulse">Thinking...</span>
            </div>
          </div>
        )}

        {error && (
          <div className="flex justify-center">
            <div className="bg-destructive/10 text-destructive rounded-lg px-4 py-2">
              <p>{error.message}</p>
              <button onClick={reload} className="underline mt-2">
                Retry
              </button>
            </div>
          </div>
        )}
      </div>

      <div className="border-t p-4">
        <form
          onSubmit={e => {
            e.preventDefault();
            if (input.trim() && !isLoading) {
              sendMessage({ text: input });
              setInput('');
            }
          }}
          className="flex gap-2"
        >
          <input
            value={input}
            onChange={e => setInput(e.target.value)}
            disabled={isLoading}
            placeholder="Type a message..."
            className="flex-1 px-4 py-2 border rounded-lg"
          />
          {isLoading ? (
            <button type="button" onClick={stop} className="px-4 py-2 border rounded-lg">
              Stop
            </button>
          ) : (
            <button type="submit" disabled={!input.trim()} className="px-4 py-2 bg-primary text-primary-foreground rounded-lg">
              Send
            </button>
          )}
        </form>
      </div>
    </div>
  );
}
```

### Server Route

```typescript
// app/api/chat/route.ts
import { convertToModelMessages, streamText, UIMessage } from 'ai';
import { openai } from '@ai-sdk/openai';

export const maxDuration = 30;

export async function POST(req: Request) {
  const { messages }: { messages: UIMessage[] } = await req.json();

  const result = streamText({
    model: openai('gpt-4o'),
    system: `You are a helpful assistant. Format your responses using markdown:
- Use **bold** for emphasis
- Use code blocks for code
- Use lists for multiple items
- Use tables when comparing things`,
    messages: convertToModelMessages(messages),
  });

  return result.toUIMessageStreamResponse({
    onError: error => {
      if (error instanceof Error) return error.message;
      return 'An error occurred';
    },
  });
}
```

## Performance Tips

1. **Use React.memo**: Wrap your Response component to prevent re-renders
2. **Key properly**: Use stable keys for message parts (index is fine within parts array)
3. **Throttle updates**: Use `experimental_throttle` on useChat for high-frequency updates
4. **Static mode for completed**: Consider switching to `mode="static"` after streaming completes
5. **Memoize configurations**: Define shikiTheme, controls, and mermaid config outside render

```tsx
// Good: Config outside component
const shikiTheme: [BundledTheme, BundledTheme] = ['github-light', 'github-dark'];
const controls = { code: true, table: true, mermaid: true };

function Chat() {
  // ...
  return <Streamdown shikiTheme={shikiTheme} controls={controls}>{content}</Streamdown>;
}

// Bad: Config inside render (creates new objects each render)
function Chat() {
  return <Streamdown shikiTheme={['github-light', 'github-dark']}>{content}</Streamdown>;
}
```
