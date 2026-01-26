# Real-World Agent Patterns

Production-ready patterns for common agent use cases.

## RAG Agent

Retrieval-Augmented Generation for knowledge-based agents:

```typescript
import { ToolLoopAgent, tool, embed, cosineSimilarity } from 'ai';
import { openai } from '@ai-sdk/openai';
import { z } from 'zod';

// Vector store (simplified - use Pinecone, Qdrant, etc. in production)
const documentStore: { content: string; embedding: number[] }[] = [];

const ragAgent = new ToolLoopAgent({
  model: 'openai/gpt-4o',
  instructions: `You are a helpful assistant with access to a knowledge base.
Use the getInformation tool to search for relevant information before answering.
Always cite your sources.`,
  tools: {
    addResource: tool({
      description: 'Add a resource to the knowledge base',
      inputSchema: z.object({
        content: z.string().describe('The content to add'),
      }),
      execute: async ({ content }) => {
        const { embedding } = await embed({
          model: openai.embedding('text-embedding-3-small'),
          value: content,
        });
        documentStore.push({ content, embedding });
        return { success: true, message: 'Resource added to knowledge base' };
      },
    }),

    getInformation: tool({
      description: 'Search the knowledge base for relevant information',
      inputSchema: z.object({
        query: z.string().describe('The search query'),
      }),
      execute: async ({ query }) => {
        const { embedding: queryEmbedding } = await embed({
          model: openai.embedding('text-embedding-3-small'),
          value: query,
        });

        // Find most similar documents
        const scored = documentStore.map(doc => ({
          content: doc.content,
          similarity: cosineSimilarity(queryEmbedding, doc.embedding),
        }));

        const topResults = scored
          .sort((a, b) => b.similarity - a.similarity)
          .slice(0, 3);

        return { results: topResults };
      },
    }),
  },
});
```

### Two-Stage RAG with Reranking

```typescript
import { embed, embedMany, rerank } from 'ai';
import { cohere } from '@ai-sdk/cohere';

async function twoStageRetrieval(query: string, documents: string[]) {
  // Stage 1: Embedding similarity (fast, get top 20)
  const { embedding: queryEmb } = await embed({
    model: openai.embedding('text-embedding-3-small'),
    value: query,
  });

  const { embeddings: docEmbs } = await embedMany({
    model: openai.embedding('text-embedding-3-small'),
    values: documents,
  });

  const candidates = documents
    .map((doc, i) => ({ doc, score: cosineSimilarity(queryEmb, docEmbs[i]) }))
    .sort((a, b) => b.score - a.score)
    .slice(0, 20)
    .map(c => c.doc);

  // Stage 2: Reranking (accurate, get top 5)
  const { results } = await rerank({
    model: cohere.reranker('rerank-v3.5'),
    query,
    documents: candidates,
    topK: 5,
  });

  return results.map(r => r.document);
}
```

## Multi-Modal Agent

Agent that processes images and PDFs:

```typescript
import { ToolLoopAgent, tool } from 'ai';
import { z } from 'zod';
import fs from 'fs';

const multiModalAgent = new ToolLoopAgent({
  model: 'anthropic/claude-sonnet-4.5', // Supports vision
  instructions: 'You can analyze images and documents. Ask users to upload files.',
  tools: {
    analyzeImage: tool({
      description: 'Analyze an uploaded image',
      inputSchema: z.object({
        imagePath: z.string().describe('Path to the image file'),
        question: z.string().describe('What to analyze about the image'),
      }),
      execute: async ({ imagePath, question }) => {
        const imageBuffer = fs.readFileSync(imagePath);
        const base64 = imageBuffer.toString('base64');
        const mimeType = imagePath.endsWith('.png') ? 'image/png' : 'image/jpeg';

        const { text } = await generateText({
          model: 'anthropic/claude-sonnet-4.5',
          messages: [{
            role: 'user',
            content: [
              { type: 'image', image: `data:${mimeType};base64,${base64}` },
              { type: 'text', text: question },
            ],
          }],
        });

        return { analysis: text };
      },
    }),

    extractFromPDF: tool({
      description: 'Extract text and analyze a PDF document',
      inputSchema: z.object({
        pdfPath: z.string().describe('Path to the PDF file'),
      }),
      execute: async ({ pdfPath }) => {
        // Use a PDF library to extract text
        const text = await extractPDFText(pdfPath);
        return { extractedText: text.slice(0, 10000) }; // Limit size
      },
    }),
  },
});
```

## Conversational Agent (Slack Bot)

Agent with thread history and context:

```typescript
import { ToolLoopAgent, tool, stepCountIs } from 'ai';
import { z } from 'zod';

// Store conversation history per thread
const threadHistory = new Map<string, Message[]>();

const slackAgent = new ToolLoopAgent({
  model: 'anthropic/claude-sonnet-4.5',
  callOptionsSchema: z.object({
    threadId: z.string(),
    channelId: z.string(),
    userId: z.string(),
  }),
  prepareCall: async ({ options, ...settings }) => {
    // Load thread history
    const history = threadHistory.get(options.threadId) || [];

    return {
      ...settings,
      instructions: `You are a helpful Slack assistant.
Thread ID: ${options.threadId}
Channel: ${options.channelId}
User: ${options.userId}

Previous messages in thread:
${history.map(m => `${m.role}: ${m.content}`).join('\n')}`,
    };
  },
  tools: {
    searchSlack: tool({
      description: 'Search Slack messages',
      inputSchema: z.object({
        query: z.string(),
        channel: z.string().optional(),
      }),
      execute: async ({ query, channel }) => {
        // Call Slack API
        const results = await slackClient.search.messages({ query, channel });
        return { messages: results.messages.matches.slice(0, 5) };
      },
    }),

    postMessage: tool({
      description: 'Post a message to a Slack channel',
      inputSchema: z.object({
        channel: z.string(),
        text: z.string(),
      }),
      execute: async ({ channel, text }) => {
        await slackClient.chat.postMessage({ channel, text });
        return { success: true };
      },
    }),
  },
  stopWhen: stepCountIs(10),
});

// Handle Slack event
async function handleSlackMessage(event: SlackEvent) {
  const result = await slackAgent.generate({
    prompt: event.text,
    options: {
      threadId: event.thread_ts || event.ts,
      channelId: event.channel,
      userId: event.user,
    },
  });

  // Update thread history
  const history = threadHistory.get(event.thread_ts || event.ts) || [];
  history.push({ role: 'user', content: event.text });
  history.push({ role: 'assistant', content: result.text });
  threadHistory.set(event.thread_ts || event.ts, history);

  return result.text;
}
```

## Customer Support Agent

Agent with ticketing and escalation:

```typescript
const supportAgent = new ToolLoopAgent({
  model: 'anthropic/claude-sonnet-4.5',
  callOptionsSchema: z.object({
    customerId: z.string(),
    tier: z.enum(['basic', 'premium', 'enterprise']),
  }),
  prepareCall: async ({ options, ...settings }) => {
    // Fetch customer history
    const customer = await db.customers.findOne({ id: options.customerId });
    const tickets = await db.tickets.find({ customerId: options.customerId }).limit(5);

    return {
      ...settings,
      instructions: `You are a customer support agent.

Customer: ${customer.name} (${options.tier} tier)
Previous tickets: ${tickets.map(t => t.summary).join(', ')}

For ${options.tier} tier:
${options.tier === 'enterprise' ? '- Prioritize their requests\n- Offer phone callback option' : ''}
${options.tier === 'basic' ? '- Suggest upgrade for advanced features' : ''}`,
    };
  },
  tools: {
    createTicket: tool({
      description: 'Create a support ticket',
      inputSchema: z.object({
        summary: z.string(),
        priority: z.enum(['low', 'medium', 'high']),
      }),
      execute: async ({ summary, priority }) => {
        const ticket = await db.tickets.create({ summary, priority });
        return { ticketId: ticket.id };
      },
    }),

    escalateToHuman: tool({
      description: 'Escalate to human agent',
      inputSchema: z.object({
        reason: z.string(),
      }),
      execute: async ({ reason }) => {
        await notifyHumanAgents(reason);
        return { escalated: true };
      },
    }),
  },
});
```

## MCP Agent (Model Context Protocol)

Connect agents to external tools and services via the standardized Model Context Protocol. MCP allows your agents to use tools from any MCP-compatible server.

### Basic MCP Integration

```typescript
import { ToolLoopAgent, stepCountIs } from 'ai';
import { createMCPClient } from '@ai-sdk/mcp';
import { openai } from '@ai-sdk/openai';

// Create MCP client with HTTP transport
const mcpClient = createMCPClient({
  transport: {
    type: 'http',
    url: 'https://mcp-server.example.com',
  },
});

// Use MCP tools in your agent
const mcpAgent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  instructions: 'You have access to external tools via MCP.',
  tools: await mcpClient.tools(),
  stopWhen: stepCountIs(20),
});

// Generate with MCP tools
const result = await mcpAgent.generate({
  prompt: 'Use the available tools to complete this task.',
});
```

### SSE Transport with OAuth

```typescript
import { createMCPClient } from '@ai-sdk/mcp';

// SSE transport with authentication
const mcpClient = createMCPClient({
  transport: {
    type: 'sse',
    url: 'https://mcp-server.example.com/sse',
    headers: {
      Authorization: `Bearer ${accessToken}`,
    },
  },
});

const agent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  tools: await mcpClient.tools(),
});
```

### Combining MCP with Local Tools

```typescript
import { ToolLoopAgent, tool } from 'ai';
import { createMCPClient } from '@ai-sdk/mcp';

const mcpClient = createMCPClient({
  transport: { type: 'http', url: 'https://mcp-server.example.com' },
});

const localTools = {
  saveToDatabase: tool({
    description: 'Save data to local database',
    inputSchema: z.object({ data: z.any() }),
    execute: async ({ data }) => {
      await db.insert(data);
      return { saved: true };
    },
  }),
};

// Merge MCP tools with local tools
const agent = new ToolLoopAgent({
  model: openai('gpt-4o'),
  instructions: 'You can use both MCP and local tools.',
  tools: {
    ...localTools,
    ...(await mcpClient.tools()),
  },
});
```

### MCP with Resources and Prompts

```typescript
import { createMCPClient } from '@ai-sdk/mcp';

const mcpClient = createMCPClient({
  transport: { type: 'http', url: 'https://mcp-server.example.com' },
});

// Access MCP resources (data sources)
const resources = await mcpClient.resources();

// Access MCP prompts (pre-defined prompt templates)
const prompts = await mcpClient.prompts();

// Use a specific prompt
const promptResult = await mcpClient.getPrompt('analyze-document', {
  document: 'path/to/document.pdf',
});
```

## Best Practices

1. **Persist context**: Store conversation history for continuity
2. **Rate limiting**: Implement per-user rate limits
3. **Graceful degradation**: Handle API failures with fallbacks
4. **Monitoring**: Log all agent interactions for debugging
5. **Testing**: Mock external services for unit tests
