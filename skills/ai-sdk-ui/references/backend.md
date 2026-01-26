# Backend Integration Reference

Server-side patterns for Next.js, Node.js, Fastify, and Nest.js.

## Next.js App Router

Standard pattern with toUIMessageStreamResponse:

```typescript
// app/api/chat/route.ts
import { streamText, convertToModelMessages, UIMessage, stepCountIs } from 'ai';

export const maxDuration = 30; // Allow 30 second responses

export async function POST(request: Request) {
  const { messages }: { messages: UIMessage[] } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    system: 'You are a helpful assistant.',
    messages: await convertToModelMessages(messages), // v6: now async
    tools: { /* ... */ },
    stopWhen: stepCountIs(5),
  });

  return result.toUIMessageStreamResponse();
}
```

## Node.js HTTP Server

Use pipeUIMessageStreamToResponse:

```typescript
import { createServer } from 'http';
import { streamText, convertToModelMessages, pipeUIMessageStreamToResponse } from 'ai';

const server = createServer(async (req, res) => {
  if (req.method === 'POST' && req.url === '/api/chat') {
    let body = '';
    for await (const chunk of req) body += chunk;
    const { messages } = JSON.parse(body);

    const result = streamText({
      model: 'openai/gpt-4o',
      messages: await convertToModelMessages(messages), // v6: now async
    });

    // Pipe stream directly to response
    pipeUIMessageStreamToResponse(result.toUIMessageStream(), res);
  }
});

server.listen(3000);
```

## Fastify

Set headers and pipe stream:

```typescript
import Fastify from 'fastify';
import { streamText, convertToModelMessages } from 'ai';

const fastify = Fastify();

fastify.post('/api/chat', async (request, reply) => {
  const { messages } = request.body as { messages: UIMessage[] };

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
  });

  // Set streaming headers
  reply.header('Content-Type', 'text/plain; charset=utf-8');
  reply.header('Transfer-Encoding', 'chunked');

  return reply.send(result.toUIMessageStream());
});

fastify.listen({ port: 3000 });
```

## Nest.js

Use @Res() decorator for streaming:

```typescript
import { Controller, Post, Body, Res } from '@nestjs/common';
import { Response } from 'express';
import { streamText, convertToModelMessages, pipeUIMessageStreamToResponse } from 'ai';

@Controller('api/chat')
export class ChatController {
  @Post()
  async chat(
    @Body() body: { messages: UIMessage[] },
    @Res() res: Response
  ) {
    const result = streamText({
      model: 'openai/gpt-4o',
      messages: await convertToModelMessages(body.messages), // v6: now async
    });

    pipeUIMessageStreamToResponse(result.toUIMessageStream(), res);
  }
}
```

## createUIMessageStream

Create custom streams with data:

```typescript
import { createUIMessageStream, streamText, convertToModelMessages } from 'ai';

export async function POST(request: Request) {
  const { messages } = await request.json();

  const stream = createUIMessageStream({
    async execute(writer) {
      // Write custom data
      writer.write({ type: 'custom', data: { status: 'starting' } });

      // Stream from model
      const result = streamText({
        model: 'openai/gpt-4o',
        messages: await convertToModelMessages(messages), // v6: now async
      });

      // Forward model stream
      for await (const chunk of result.fullStream) {
        writer.write(chunk);
      }

      // Write final custom data
      writer.write({ type: 'custom', data: { status: 'complete' } });
    },
  });

  return new Response(stream.readable, {
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
  });
}
```

## Streaming Custom Data

Add metadata to responses:

```typescript
// Server
const result = streamText({
  model: 'openai/gpt-4o',
  messages: await convertToModelMessages(messages), // v6: now async
});

return result.toUIMessageStreamResponse({
  // Add sources for RAG
  sources: [
    { id: 'doc-1', title: 'User Guide', url: '/docs/guide' },
    { id: 'doc-2', title: 'FAQ', url: '/docs/faq' },
  ],
});
```

```typescript
// Client
const { messages, sources } = useChat({
  transport: new DefaultChatTransport({ api: '/api/chat' }),
});

// sources is populated from server response
```

## Error Handling

```typescript
export async function POST(request: Request) {
  try {
    const { messages } = await request.json();

    const result = streamText({
      model: 'openai/gpt-4o',
      messages: await convertToModelMessages(messages), // v6: now async
    });

    return result.toUIMessageStreamResponse({
      onError: (error) => {
        // Customize error message sent to client
        if (error instanceof RateLimitError) {
          return 'Rate limit exceeded. Please try again.';
        }
        return 'An error occurred.';
      },
    });
  } catch (error) {
    return new Response('Invalid request', { status: 400 });
  }
}
```

## With Authentication

```typescript
import { auth } from '@/lib/auth';

export async function POST(request: Request) {
  const session = await auth();
  if (!session) {
    return new Response('Unauthorized', { status: 401 });
  }

  const { messages } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages), // v6: now async
    // Use user context
    system: `You are helping ${session.user.name}.
Their preferences: ${session.user.preferences}`,
  });

  return result.toUIMessageStreamResponse();
}
```

## Agent Responses

Use createAgentUIStreamResponse for ToolLoopAgent:

```typescript
import { createAgentUIStreamResponse } from 'ai';
import { myAgent } from '@/ai/agents/my-agent';

export async function POST(request: Request) {
  const { messages, options } = await request.json();

  return createAgentUIStreamResponse({
    agent: myAgent,
    messages,
    options, // Passed to agent's callOptionsSchema
    onFinish({ steps, usage }) {
      // Log completion
      console.log('Agent finished', { stepCount: steps.length });
    },
  });
}
```

## Ensuring Stream Completion

Use `onFinish` callbacks to ensure post-stream logic runs even on client abort:

```typescript
import { streamText, convertToModelMessages } from 'ai';

export async function POST(request: Request) {
  const { messages } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    messages: await convertToModelMessages(messages),
    // onFinish runs even if client disconnects
    onFinish: async ({ text, usage }) => {
      await saveToDatabase(text);
      await logUsage(usage);
    },
    onError: async (error) => {
      await logError(error);
    },
  });

  return result.toUIMessageStreamResponse();
}
```

> **Note**: v6's `toUIMessageStreamResponse()` handles stream lifecycle automatically. The deprecated `consumeSseStream()` pattern is no longer needed—use `onFinish` and `onError` callbacks instead.

## Best Practices

1. **Set maxDuration**: Increase timeout for long responses
2. **Await convertToModelMessages**: v6 requires async call
3. **Error handling**: Return user-friendly error messages
4. **Authentication**: Validate sessions before processing
5. **Logging**: Log completions via `onFinish` callback
6. **Use onFinish**: Ensure DB writes/logging complete even on abort

Keep file under 300 lines.
