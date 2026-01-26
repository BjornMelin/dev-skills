# AI SDK UI v6 Migration Guide

Complete guide for migrating to AI SDK v6 stable release.

## Automated Migration

### Step 1: Run Codemod

```bash
npx @ai-sdk/codemod v6
```

The codemod automatically handles most breaking changes.

### Step 2: Update Packages

```bash
pnpm add ai@^6.0.3 @ai-sdk/react@^3.0.3
```

### Step 3: Manual Fixes

#### Async convertToModelMessages

The `convertToModelMessages` function is now async and must be awaited:

```typescript
// Before
const messages = convertToModelMessages(uiMessages);

// After
const messages = await convertToModelMessages(uiMessages);
```

#### Helper Function Renames

Static tool helpers have been renamed:

```typescript
// Before
if (isToolUIPart(part)) {
  const name = getToolName(part);
}

// After (for static tools only)
if (isStaticToolUIPart(part)) {
  const name = getStaticToolName(part);
}

// Or for both static and dynamic tools:
// (renamed from isToolOrDynamicToolUIPart/getToolOrDynamicToolName)
if (isToolUIPart(part)) {
  const name = getToolName(part);
}
```

### Step 4: Type Check

```bash
pnpm type-check
```

## Breaking Changes Reference

| Before | After |
|--------|-------|
| `convertToModelMessages()` | `await convertToModelMessages()` |
| `isToolUIPart()` | `isStaticToolUIPart()` |
| `isToolOrDynamicToolUIPart()` | `isToolUIPart()` |
| `getToolName()` | `getStaticToolName()` |
| `getToolOrDynamicToolName()` | `getToolName()` |

## New Features in v6

### Tool Approval

Require user approval before executing sensitive tools:

#### Server Configuration

```typescript
const result = streamText({
  model: 'openai/gpt-4o',
  messages: await convertToModelMessages(messages),
  tools: {
    deleteFile: {
      description: 'Delete a file',
      inputSchema: z.object({ path: z.string() }),
      needsApproval: true, // New: requires user approval
      execute: async ({ path }) => {
        await fs.unlink(path);
        return { deleted: path };
      },
    },
  },
});
```

#### Client Approval Handling

```typescript
const { messages, addToolApprovalResponse } = useChat({
  transport: new DefaultChatTransport({ api: '/api/chat' }),
});

// Handle approval-requested state
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
  }
})}
```

### New Tool Part State

The `approval-requested` state is now available for tool parts:

| State | Description |
|-------|-------------|
| `input-streaming` | Tool input being streamed |
| `input-available` | Tool input complete, waiting for execution |
| `approval-requested` | Tool requires user approval |
| `output-available` | Tool execution complete with output |
| `output-error` | Tool execution failed |

### Type Helpers

New type inference helpers:

```typescript
import { InferUITools, InferAgentUIMessage, ToolSet, UIMessage } from 'ai';

const tools = { /* ... */ } satisfies ToolSet;

// Infer tool types from tool definitions
type MyUITools = InferUITools<typeof tools>;

// Infer message type from agent
type MyAgentUIMessage = InferAgentUIMessage<typeof myAgent>;
```

### Stream Completion with onFinish

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

> **Note**: v6's `toUIMessageStreamResponse()` handles stream lifecycle automatically. No additional helper needed.

### AI Elements

Pre-built shadcn/ui components for chat UIs:

- https://ai-sdk.dev/elements

## Migration Checklist

- [ ] Run `npx @ai-sdk/codemod v6`
- [ ] Update packages to v6 versions
- [ ] Add `await` to all `convertToModelMessages()` calls
- [ ] Rename helper functions if used directly
- [ ] Add `approval-requested` state handling for tools with `needsApproval`
- [ ] Run `pnpm type-check` to verify
- [ ] Test all chat functionality

## Common Issues

### "convertToModelMessages is not a function"

Ensure you're importing from `ai`:

```typescript
import { convertToModelMessages } from 'ai';
```

### Tool approval not working

1. Ensure `needsApproval: true` is set on server-side tool definition
2. Handle `approval-requested` state in client rendering
3. Call `addToolApprovalResponse` with correct `id` from `part.approval.id`

### Type errors after migration

Run the codemod first, then check for:

1. Missing `await` on `convertToModelMessages`
2. Renamed helper functions
3. New tool part states in switch statements
