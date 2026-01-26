# Production UI Patterns

Best practices for production chat UIs with AI SDK.

## Error Handling

### Hook-Level Errors

```typescript
const { messages, error, sendMessage } = useChat({
  transport: new DefaultChatTransport({ api: '/api/chat' }),
  onError: (error) => {
    // Log to monitoring
    console.error('Chat error:', error);
    // Show toast notification
    toast.error('Failed to send message. Please try again.');
  },
});

// Render error state
{error && (
  <div className="bg-red-50 p-4 rounded">
    <p>Something went wrong.</p>
    <button onClick={() => sendMessage(lastMessage)}>
      Retry
    </button>
  </div>
)}
```

### Masked Errors

Server errors are masked by default. Customize:

```typescript
// Server
return result.toUIMessageStreamResponse({
  onError: (error) => {
    if (error instanceof RateLimitError) {
      return 'Rate limit exceeded. Please wait a moment.';
    }
    if (error instanceof InvalidInputError) {
      return 'Invalid input. Please check your message.';
    }
    // Return generic message for other errors
    return 'Something went wrong. Please try again.';
  },
});
```

### Warnings

Handle non-fatal issues:

```typescript
const { messages, warnings } = useChat({ /* ... */ });

{warnings.length > 0 && (
  <div className="bg-yellow-50 p-2 text-sm">
    {warnings.map((warning, i) => (
      <p key={i}>{warning}</p>
    ))}
  </div>
)}
```

## Message Metadata

Add custom data to messages:

```typescript
// Server
import { createUIMessageStream } from 'ai';

const stream = createUIMessageStream({
  async execute(writer) {
    const result = streamText({
      model: 'openai/gpt-4o',
      messages: await convertToModelMessages(messages), // v6: now async
    });

    // Add metadata to assistant message
    writer.writeMessageMetadata({
      model: 'gpt-4o',
      timestamp: Date.now(),
      cost: calculateCost(result.usage),
    });

    for await (const chunk of result.fullStream) {
      writer.write(chunk);
    }
  },
});
```

```typescript
// Client - access metadata
{messages.map(m => (
  <div key={m.id}>
    <p>{m.content}</p>
    {m.metadata && (
      <span className="text-xs text-gray-500">
        Model: {m.metadata.model} | Cost: ${m.metadata.cost}
      </span>
    )}
  </div>
))}
```

## Custom Transport

Customize request/response handling:

```typescript
import { ChatTransport } from 'ai';

class CustomTransport implements ChatTransport {
  async send(options: { messages: UIMessage[]; abortSignal?: AbortSignal }) {
    const response = await fetch('/api/chat', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Request-ID': crypto.randomUUID(),
      },
      body: JSON.stringify({
        messages: options.messages,
        timestamp: Date.now(),
      }),
      signal: options.abortSignal,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    return response;
  }
}

const { messages } = useChat({
  transport: new CustomTransport(),
});
```

## Sources for RAG

Display document sources:

```typescript
// Server
return result.toUIMessageStreamResponse({
  sources: retrievedDocuments.map(doc => ({
    id: doc.id,
    title: doc.title,
    url: doc.url,
    snippet: doc.content.slice(0, 200),
  })),
});
```

```typescript
// Client
const { messages, sources } = useChat({ /* ... */ });

{sources.length > 0 && (
  <div className="border-t mt-4 pt-4">
    <h4 className="font-bold">Sources</h4>
    {sources.map(source => (
      <a key={source.id} href={source.url} className="block">
        {source.title}
      </a>
    ))}
  </div>
)}
```

## Type Safety

Use typed messages throughout:

```typescript
import { UIMessage } from 'ai';

// Define custom metadata schema
const metadataSchema = z.object({
  model: z.string(),
  cost: z.number(),
  timestamp: z.number(),
});

type MyMessage = UIMessage & {
  metadata?: z.infer<typeof metadataSchema>;
};

// Type-safe message component
function Message({ message }: { message: MyMessage }) {
  return (
    <div>
      {message.parts.map((part, i) => {
        switch (part.type) {
          case 'text':
            return <p key={i}>{part.text}</p>;
          case 'tool-weather':
            return <WeatherCard key={i} {...part.output} />;
        }
      })}
    </div>
  );
}
```

## Optimistic Updates

Show messages immediately:

```typescript
const { messages, sendMessage, status } = useChat({ /* ... */ });

const handleSend = async (text: string) => {
  // Message appears immediately
  sendMessage({ text });
  // Clear input right away
  setInput('');
};

// Show streaming indicator
{status === 'streaming' && (
  <div className="animate-pulse">AI is typing...</div>
)}
```

## Rate Limiting

Prevent spam:

```typescript
const [lastSent, setLastSent] = useState(0);
const MIN_INTERVAL = 1000; // 1 second

const handleSend = () => {
  const now = Date.now();
  if (now - lastSent < MIN_INTERVAL) {
    toast.warning('Please wait before sending another message');
    return;
  }

  setLastSent(now);
  sendMessage({ text: input });
};
```

## Accessibility

```typescript
<div role="log" aria-live="polite" aria-label="Chat messages">
  {messages.map(m => (
    <div
      key={m.id}
      role="article"
      aria-label={`${m.role} message`}
    >
      {/* ... */}
    </div>
  ))}
</div>

{status === 'streaming' && (
  <div aria-live="assertive" className="sr-only">
    AI is responding
  </div>
)}
```

## Best Practices

1. **Error boundaries**: Wrap chat in error boundary
2. **Loading states**: Show clear streaming indicators
3. **Retry logic**: Allow retrying failed messages
4. **Accessibility**: Use proper ARIA attributes
5. **Rate limiting**: Prevent message spam
6. **Monitoring**: Log errors to observability service
7. **Type safety**: Use typed messages and parts
