# useChat Fundamentals

Complete reference for the `useChat` hook API, state management, and configuration patterns.

## Core API

### Basic Setup

```tsx
'use client';

import { useChat } from '@ai-sdk/react';
import { DefaultChatTransport } from 'ai';
import { useState } from 'react';

export default function Chat() {
  const { messages, sendMessage, status } = useChat({
    transport: new DefaultChatTransport({
      api: '/api/chat',
    }),
  });
  const [input, setInput] = useState('');

  return (
    <>
      {messages.map(message => (
        <div key={message.id}>
          {message.role === 'user' ? 'User: ' : 'AI: '}
          {message.parts.map((part, index) =>
            part.type === 'text' ? <span key={index}>{part.text}</span> : null,
          )}
        </div>
      ))}

      <form
        onSubmit={e => {
          e.preventDefault();
          if (input.trim()) {
            sendMessage({ text: input });
            setInput('');
          }
        }}
      >
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          disabled={status !== 'ready'}
          placeholder="Say something..."
        />
        <button type="submit" disabled={status !== 'ready'}>
          Submit
        </button>
      </form>
    </>
  );
}
```

### Hook Return Values

```tsx
const {
  // Message state
  messages,           // UIMessage[] - Current conversation
  setMessages,        // (messages: UIMessage[]) => void - Update messages

  // Input helpers (optional - can use manual state)
  sendMessage,        // (message: { text: string; files?: FileList }) => void

  // Status tracking
  status,            // 'ready' | 'submitted' | 'streaming' | 'error'

  // Control methods
  stop,              // () => void - Abort current request
  regenerate,        // () => void - Regenerate last response
  reload,            // () => void - Retry after error

  // Error state
  error,             // Error | undefined

  // Tool handling
  addToolOutput,     // Add tool execution results
  addToolApprovalResponse, // v6: Respond to tool approval requests
} = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
  }),
});
```

## Status Management

### Status States

```tsx
// Status values and their meanings:
const statusInfo = {
  'ready': 'Idle - can accept new messages',
  'submitted': 'Request sent - awaiting stream start',
  'streaming': 'Actively receiving response chunks',
  'error': 'Request failed - see error object',
};

// Using status for UI control
const { status, stop } = useChat();

return (
  <>
    {/* Show loading spinner */}
    {(status === 'submitted' || status === 'streaming') && <Spinner />}

    {/* Show stop button */}
    {(status === 'submitted' || status === 'streaming') && (
      <button onClick={() => stop()}>Stop</button>
    )}

    {/* Disable input during processing */}
    <input disabled={status !== 'ready'} />
  </>
);
```

### Cancellation and Regeneration

```tsx
const { stop, regenerate, status } = useChat();

return (
  <>
    {/* Stop current generation */}
    <button
      onClick={stop}
      disabled={!(status === 'streaming' || status === 'submitted')}
    >
      Stop
    </button>

    {/* Regenerate last message */}
    <button
      onClick={regenerate}
      disabled={!(status === 'ready' || status === 'error')}
    >
      Regenerate
    </button>
  </>
);
```

## Message State Management

### Direct Message Manipulation

```tsx
const { messages, setMessages } = useChat();

// Delete a message
const handleDelete = (id: string) => {
  setMessages(messages.filter(message => message.id !== id));
};

// Edit a message
const handleEdit = (id: string, newText: string) => {
  setMessages(messages.map(message =>
    message.id === id
      ? {
          ...message,
          parts: [{ type: 'text', text: newText }]
        }
      : message
  ));
};

return (
  <>
    {messages.map(message => (
      <div key={message.id}>
        {message.parts.map((part, index) =>
          part.type === 'text' ? <span key={index}>{part.text}</span> : null,
        )}
        <button onClick={() => handleDelete(message.id)}>Delete</button>
      </div>
    ))}
  </>
);
```

### Initial Messages

```tsx
// Load existing conversation
const { messages } = useChat({
  id: chatId,
  messages: initialMessages, // UIMessage[]
  transport: new DefaultChatTransport({
    api: '/api/chat',
  }),
});
```

## Event Callbacks

### onFinish Callback

```tsx
const { messages } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
  }),
  onFinish: ({ message, messages, isAbort, isDisconnect, isError }) => {
    // message: The new assistant message
    // messages: All messages including the new one
    // isAbort: User called stop()
    // isDisconnect: Network disconnection
    // isError: Error occurred

    if (!isError && !isAbort) {
      console.log('Generation completed successfully');
      // Save to database, update analytics, etc.
    }
  },
});
```

### onError Callback

```tsx
const { messages } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
  }),
  onError: (error) => {
    console.error('Chat error:', error);
    // Show toast notification
    // Log to error tracking service
    // Custom error handling
  },
});
```

### onData Callback

```tsx
const { messages } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
  }),
  onData: (data) => {
    console.log('Received data part:', data);

    // Handle different data types
    if (data.type === 'data-notification') {
      showToast(data.data.message);
    }

    // Can abort by throwing error
    if (data.type === 'data-error') {
      throw new Error('Aborting due to error data');
    }
  },
});
```

## Request Configuration

### Hook-Level Configuration (All Requests)

```tsx
const { messages, sendMessage } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/custom-chat',
    headers: {
      Authorization: 'Bearer token',
      'X-Custom-Header': 'value',
    },
    body: {
      user_id: '123',
      preferences: { theme: 'dark' },
    },
    credentials: 'same-origin',
  }),
});
```

### Dynamic Hook-Level Configuration

```tsx
const { messages, sendMessage } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
    // Functions are called on each request
    headers: () => ({
      Authorization: `Bearer ${getAuthToken()}`,
      'X-User-ID': getCurrentUserId(),
    }),
    body: () => ({
      sessionId: getCurrentSessionId(),
      preferences: getUserPreferences(),
    }),
    credentials: () => 'include',
  }),
});
```

**Note**: For component state that changes over time, use `useRef` to store the current value and reference `ref.current` in your configuration function, or use request-level options.

### Request-Level Configuration (Recommended)

```tsx
// Pass options as second parameter to sendMessage
sendMessage(
  { text: input },
  {
    headers: {
      Authorization: 'Bearer token123',
      'X-Custom-Header': 'custom-value',
    },
    body: {
      temperature: 0.7,
      max_tokens: 100,
      user_id: '123',
    },
    metadata: {
      userId: 'user123',
      sessionId: 'session456',
    },
  },
);
```

**Best Practice**: Request-level options take precedence over hook-level options and provide better flexibility.

### Custom Body Fields Per Request

```tsx
// Client
sendMessage(
  { text: input },
  {
    body: {
      customKey: 'customValue',
      temperature: 0.8,
    },
  },
);

// Server - retrieve custom fields
export async function POST(req: Request) {
  const { messages, customKey, temperature } = await req.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages),
    temperature,
  });

  return result.toUIMessageStreamResponse();
}
```

## Throttling UI Updates

```tsx
const { messages } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
  }),
  // Throttle updates to 50ms (React only)
  // Note: Still experimental in v6
  experimental_throttle: 50,
});
```

**Effect**: Reduces render frequency during streaming. Default is to render on every chunk.

## Transport Options

```tsx
const { messages } = useChat({
  transport: new DefaultChatTransport({
    api: '/api/chat',
    // Request configuration
    headers: { Authorization: 'Bearer token' },
    body: { customField: 'value' },
    credentials: 'include',
    // Stream resumption
    resume: true,
  }),
});
```

## Error State Handling

### Display Error Message

```tsx
const { messages, error, reload } = useChat();

return (
  <div>
    {messages.map(m => (
      <div key={m.id}>
        {m.role}:{' '}
        {m.parts.map((part, index) =>
          part.type === 'text' ? <span key={index}>{part.text}</span> : null,
        )}
      </div>
    ))}

    {error && (
      <>
        <div>An error occurred.</div>
        <button onClick={() => reload()}>Retry</button>
      </>
    )}

    <form onSubmit={handleSubmit}>
      <input disabled={error != null} />
    </form>
  </div>
);
```

**Best Practice**: Show generic error messages to avoid leaking server information.

### Custom Error Handling with Message Replacement

```tsx
const { sendMessage, error, messages, setMessages } = useChat();

function customSubmit(event: React.FormEvent) {
  event.preventDefault();

  if (error != null) {
    // Remove failed message before retry
    setMessages(messages.slice(0, -1));
  }

  sendMessage({ text: input });
  setInput('');
}
```

## File Attachments

### Using FileList

```tsx
const { messages, sendMessage, status } = useChat();
const [input, setInput] = useState('');
const [files, setFiles] = useState<FileList | undefined>();
const fileInputRef = useRef<HTMLInputElement>(null);

return (
  <>
    <form
      onSubmit={e => {
        e.preventDefault();
        if (input.trim()) {
          sendMessage({ text: input, files });
          setInput('');
          setFiles(undefined);
          if (fileInputRef.current) {
            fileInputRef.current.value = '';
          }
        }
      }}
    >
      <input
        type="file"
        onChange={e => {
          if (e.target.files) {
            setFiles(e.target.files);
          }
        }}
        multiple
        ref={fileInputRef}
      />
      <input
        value={input}
        onChange={e => setInput(e.target.value)}
        disabled={status !== 'ready'}
      />
    </form>

    {/* Render file parts */}
    {messages.map(message => (
      <div key={message.id}>
        {message.parts.map((part, index) => {
          if (part.type === 'text') {
            return <span key={index}>{part.text}</span>;
          }
          if (part.type === 'file' && part.mediaType?.startsWith('image/')) {
            return <img key={index} src={part.url} alt={part.filename} />;
          }
          return null;
        })}
      </div>
    ))}
  </>
);
```

**Note**: Only `image/*` and `text/*` content types are automatically converted to multi-modal content parts.

### Using File Objects

```tsx
import { FileUIPart } from 'ai';

const [files] = useState<FileUIPart[]>([
  {
    type: 'file',
    filename: 'earth.png',
    mediaType: 'image/png',
    url: 'https://example.com/earth.png',
  },
  {
    type: 'file',
    filename: 'data.png',
    mediaType: 'image/png',
    url: 'data:image/png;base64,iVBORw0KGgo...',
  },
]);

sendMessage({ text: input, files });
```

## Type Safety

### Type Inference for Tools

```tsx
import { InferUITools, ToolSet } from 'ai';
import { z } from 'zod';

const tools = {
  weather: {
    description: 'Get weather',
    inputSchema: z.object({
      location: z.string(),
    }),
    execute: async ({ location }) => {
      return `Weather in ${location}: sunny`;
    },
  },
} satisfies ToolSet;

type MyUITools = InferUITools<typeof tools>;
type MyUIMessage = UIMessage<never, UIDataTypes, MyUITools>;

// Use with useChat
const { messages } = useChat<MyUIMessage>();
```

## Common Patterns

### Loading State with Disable

```tsx
const { status, sendMessage } = useChat();
const isLoading = status === 'submitted' || status === 'streaming';

return (
  <>
    {isLoading && <Spinner />}
    <button disabled={isLoading}>Send</button>
  </>
);
```

### Conditional Rendering Based on Role

```tsx
{messages.map(message => (
  <div key={message.id} className={message.role}>
    {message.role === 'user' && <UserAvatar />}
    {message.role === 'assistant' && <BotAvatar />}
    {message.parts.map((part, index) =>
      part.type === 'text' ? part.text : null,
    )}
  </div>
))}
```

### Auto-scroll to Bottom

```tsx
const messagesEndRef = useRef<HTMLDivElement>(null);

useEffect(() => {
  messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
}, [messages]);

return (
  <>
    {messages.map(m => <MessageBubble key={m.id} message={m} />)}
    <div ref={messagesEndRef} />
  </>
);
```
