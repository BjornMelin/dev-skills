# Message Persistence Reference

Storing and loading chat messages with AI SDK UI.

## Basic Pattern

```typescript
'use client';
import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport } from 'ai';

export default function Chat({ chatId }: { chatId: string }) {
  const [initialMessages, setInitialMessages] = useState<UIMessage[]>([]);

  // Load chat history on mount
  useEffect(() => {
    async function loadChat() {
      const messages = await db.messages.findMany({
        where: { chatId },
        orderBy: { createdAt: 'asc' },
      });
      setInitialMessages(messages);
    }
    loadChat();
  }, [chatId]);

  const { messages, sendMessage } = useChat({
    transport: new DefaultChatTransport({ api: '/api/chat' }),
    initialMessages,
  });

  return (/* ... */);
}
```

## Server-Side Persistence

Save messages in onFinish callback:

```typescript
// app/api/chat/route.ts
import { streamText, convertToModelMessages, UIMessage } from 'ai';

export async function POST(request: Request) {
  const { messages, chatId }: { messages: UIMessage[]; chatId: string } =
    await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
    onFinish: async ({ text, usage }) => {
      // Save assistant message to database
      await db.messages.create({
        data: {
          chatId,
          role: 'assistant',
          content: text,
          tokenUsage: usage?.totalTokens,
        },
      });
    },
  });

  return result.toUIMessageStreamResponse();
}
```

## Server-Side ID Generation

Generate message IDs on server for consistency:

```typescript
// app/api/chat/route.ts
import { generateId } from 'ai';

export async function POST(request: Request) {
  const { messages, chatId } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
  });

  // Generate ID before streaming
  const messageId = generateId();

  return result.toUIMessageStreamResponse({
    messageId, // Client will use this ID
  });
}
```

## Validate UIMessages

Ensure loaded messages are valid:

```typescript
import { validateUIMessages } from 'ai';

async function loadChat(chatId: string) {
  const rawMessages = await db.messages.findMany({
    where: { chatId },
  });

  // Validate structure and types
  const validMessages = validateUIMessages(rawMessages);

  return validMessages;
}
```

## Stream Resumption

Resume interrupted streams after client disconnect:

### Server Setup

```typescript
// app/api/chat/route.ts
export async function POST(request: Request) {
  const { messages, chatId } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
  });

  const messageId = generateId();

  // Store stream for potential resumption
  await storeStream(chatId, messageId, result);

  return result.toUIMessageStreamResponse({ messageId });
}

// GET endpoint for resumption
export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const chatId = searchParams.get('chatId');
  const messageId = searchParams.get('messageId');

  const storedStream = await getStoredStream(chatId, messageId);
  if (!storedStream) {
    return new Response('Stream not found', { status: 404 });
  }

  return storedStream.toUIMessageStreamResponse({ messageId });
}
```

### Client Setup

```typescript
const { messages, sendMessage } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
    resume: true, // Enable stream resumption
  }),
});
```

## Ensuring Persistence on Client Disconnect

Use `onFinish` callbacks to ensure messages are saved even if the client disconnects:

```typescript
// app/api/chat/route.ts
import { streamText, convertToModelMessages } from 'ai';

export async function POST(request: Request) {
  const { messages, chatId } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
    // onFinish runs even if client disconnects
    onFinish: async ({ text, usage }) => {
      await db.messages.create({
        data: { chatId, role: 'assistant', content: text },
      });
      await logUsage(chatId, usage);
    },
    onError: async (error) => {
      await logError(chatId, error);
    },
  });

  return result.toUIMessageStreamResponse();
}
```

> **Note**: v6's `toUIMessageStreamResponse()` handles stream lifecycle automatically. The deprecated `consumeSseStream()` pattern is no longer needed—use `onFinish` and `onError` callbacks instead.

## Creating New Chats

```typescript
// app/api/chats/route.ts
export async function POST(request: Request) {
  const { title } = await request.json();

  const chat = await db.chats.create({
    data: {
      id: generateId(),
      title,
      createdAt: new Date(),
    },
  });

  return Response.json({ chatId: chat.id });
}
```

```typescript
// Client
async function createNewChat() {
  const res = await fetch('/api/chats', {
    method: 'POST',
    body: JSON.stringify({ title: 'New Chat' }),
  });
  const { chatId } = await res.json();
  router.push(`/chat/${chatId}`);
}
```

## Full Example

```typescript
'use client';
import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport, validateUIMessages, UIMessage } from 'ai';
import { useEffect, useState } from 'react';

export default function Chat({ chatId }: { chatId: string }) {
  const [initialMessages, setInitialMessages] = useState<UIMessage[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function loadChat() {
      try {
        const res = await fetch(`/api/chats/${chatId}/messages`);
        const messages = await res.json();
        setInitialMessages(validateUIMessages(messages));
      } finally {
        setLoading(false);
      }
    }
    loadChat();
  }, [chatId]);

  const { messages, sendMessage, status } = useChat({
    transport: new DefaultChatTransport({
      api: '/api/chat',
      body: { chatId }, // Include chatId in requests
      resume: true,
    }),
    initialMessages,
    onFinish: async (message) => {
      // Save user message
      await fetch(`/api/chats/${chatId}/messages`, {
        method: 'POST',
        body: JSON.stringify({ role: 'user', content: message.content }),
      });
    },
  });

  if (loading) return <div>Loading chat...</div>;

  return (
    <div>
      {messages.map(m => (
        <Message key={m.id} message={m} />
      ))}
      {/* ... input form */}
    </div>
  );
}
```

## Best Practices

1. **Validate on load**: Always validate stored messages
2. **Server-side IDs**: Generate IDs on server for consistency
3. **Use onFinish**: Ensure DB persistence even on client disconnect
4. **Resume support**: Enable for long-running responses
5. **Include chatId**: Pass chatId in all API requests
6. **Error handling**: Handle load/save failures gracefully
7. **Await convertToModelMessages**: v6 requires async call
