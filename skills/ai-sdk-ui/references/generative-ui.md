# Generative UI Reference

Building dynamic UI components from LLM tool outputs.

## How It Works

1. Model receives prompt + available tools
2. Model calls a tool based on context
3. Tool executes and returns data
4. Data is passed to React component
5. Component renders in chat UI

## Basic Setup

### 1. Define Tool

```typescript
// ai/tools.ts
import { tool } from 'ai';
import { z } from 'zod';

export const weatherTool = tool({
  description: 'Display weather for a location',
  inputSchema: z.object({
    location: z.string().describe('The location to get weather for'),
  }),
  execute: async ({ location }) => {
    // Simulate API call
    await new Promise(r => setTimeout(r, 1000));
    return { weather: 'Sunny', temperature: 75, location };
  },
});

export const tools = { displayWeather: weatherTool };
```

### 2. Add to API Route

```typescript
// app/api/chat/route.ts
import { streamText, convertToModelMessages, UIMessage, stepCountIs } from 'ai';
import { tools } from '@/ai/tools';

export async function POST(request: Request) {
  const { messages }: { messages: UIMessage[] } = await request.json();

  const result = streamText({
    model: 'openai/gpt-4o',
    system: 'You are a helpful assistant!',
    messages: await convertToModelMessages(messages), // v6: now async
    tools,
    stopWhen: stepCountIs(5),
  });

  return result.toUIMessageStreamResponse();
}
```

### 3. Create UI Component

```typescript
// components/weather.tsx
type WeatherProps = {
  temperature: number;
  weather: string;
  location: string;
};

export const Weather = ({ temperature, weather, location }: WeatherProps) => (
  <div className="p-4 border rounded-lg bg-blue-50">
    <h2 className="font-bold">Weather in {location}</h2>
    <p>Condition: {weather}</p>
    <p>Temperature: {temperature}°F</p>
  </div>
);
```

### 4. Render in Chat

```typescript
// app/page.tsx
'use client';
import { useChat } from '@ai-sdk/react';
import { Weather } from '@/components/weather';

export default function Chat() {
  const [input, setInput] = useState('');
  const { messages, sendMessage } = useChat();

  return (
    <div>
      {messages.map(message => (
        <div key={message.id}>
          <strong>{message.role}:</strong>
          {message.parts.map((part, index) => {
            if (part.type === 'text') {
              return <span key={index}>{part.text}</span>;
            }

            // Typed tool part: tool-${toolName}
            if (part.type === 'tool-displayWeather') {
              switch (part.state) {
                case 'input-available':
                  return <div key={index}>Loading weather...</div>;
                case 'output-available':
                  return <Weather key={index} {...part.output} />;
                case 'output-error':
                  return <div key={index}>Error: {part.errorText}</div>;
              }
            }

            return null;
          })}
        </div>
      ))}

      <form onSubmit={e => {
        e.preventDefault();
        sendMessage({ text: input });
        setInput('');
      }}>
        <input value={input} onChange={e => setInput(e.target.value)} />
        <button type="submit">Send</button>
      </form>
    </div>
  );
}
```

## Adding Multiple Tools

```typescript
// ai/tools.ts
export const stockTool = tool({
  description: 'Get stock price',
  inputSchema: z.object({
    symbol: z.string().describe('Stock symbol'),
  }),
  execute: async ({ symbol }) => {
    return { symbol, price: 150.25, change: +2.5 };
  },
});

export const tools = {
  displayWeather: weatherTool,
  getStockPrice: stockTool,
};
```

```typescript
// Render both tools
{message.parts.map((part, index) => {
  switch (part.type) {
    case 'text':
      return <span key={index}>{part.text}</span>;

    case 'tool-displayWeather':
      return part.state === 'output-available'
        ? <Weather key={index} {...part.output} />
        : <Skeleton key={index} />;

    case 'tool-getStockPrice':
      return part.state === 'output-available'
        ? <Stock key={index} {...part.output} />
        : <Skeleton key={index} />;
  }
})}
```

## State-Aware Rendering

Handle all tool states for best UX:

```typescript
function ToolRenderer({ part }: { part: ToolPart }) {
  switch (part.state) {
    case 'input-streaming':
      // Tool input being streamed
      return <StreamingIndicator args={part.input} />;

    case 'input-available':
      // Input complete, executing
      return <LoadingSpinner label={`Getting ${part.input.location}...`} />;

    case 'output-available':
      // Success - render component
      return <Weather {...part.output} />;

    case 'output-error':
      // Error state
      return <ErrorCard message={part.errorText} />;
  }
}
```

## Progressive Loading

Show partial data as it streams:

```typescript
case 'input-streaming':
  // Show what we know so far
  return (
    <div className="animate-pulse">
      <p>Searching for: {part.input?.location || '...'}</p>
    </div>
  );
```

## Charts and Visualizations

```typescript
const chartTool = tool({
  description: 'Display a chart',
  inputSchema: z.object({
    type: z.enum(['bar', 'line', 'pie']),
    data: z.array(z.object({
      label: z.string(),
      value: z.number(),
    })),
    title: z.string(),
  }),
  execute: async (input) => input, // Pass through
});

// Render
case 'tool-displayChart':
  if (part.state === 'output-available') {
    return <Chart type={part.output.type} data={part.output.data} />;
  }
```

## Best Practices

1. **State handling**: Always handle all 5 tool states (including `approval-requested`)
2. **Loading states**: Show meaningful loading indicators
3. **Error handling**: Display user-friendly error messages
4. **Type safety**: Use typed tool parts (`tool-${name}`)
5. **Accessibility**: Ensure generated UI is accessible
6. **Streaming**: Leverage input-streaming for progressive UX
7. **AI Elements**: Consider using https://ai-sdk.dev/elements for pre-built components
