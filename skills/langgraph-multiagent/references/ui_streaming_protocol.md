# UI streaming protocol (SSE/Web) for multi-agent systems

This file defines a **simple, typed event protocol** for streaming agent execution to UIs (Next.js, Streamlit, etc.).

Goal: represent **tokens**, **state updates**, **custom progress**, and **interrupts** consistently.

## Event types

All events are JSON objects with a `type` discriminator.

### `meta`

Sent once at the start.

- includes `threadId` and optional model/assistant identifiers
- optionally includes `runId` if the backend creates runs

### `token`

Streaming token chunks from `stream_mode="messages"`.

- fields: `content` (string), optional `node`, optional `metadata`

### `update`

State updates from `stream_mode="updates"`.

- fields: `data` (object), optional `node`
- interrupts may show up as `{"__interrupt__": ...}` inside `data`

### `interrupt`

Explicit interrupt event (recommended to emit separately).

- fields: `interrupt` (object)

### `custom`

User-defined progress events emitted by tools/nodes using a writer.

- fields: `event` (object)

### `done` / `error`

Stream termination.

## TypeScript (Zod) schema sketch

```ts
import { z } from 'zod';

export const MetaEvent = z.object({
  type: z.literal('meta'),
  threadId: z.string(),
  runId: z.string().optional(),
});

export const TokenEvent = z.object({
  type: z.literal('token'),
  content: z.string(),
  node: z.string().optional(),
  metadata: z.unknown().optional(),
});

export const UpdateEvent = z.object({
  type: z.literal('update'),
  data: z.record(z.unknown()),
});

export const InterruptEvent = z.object({
  type: z.literal('interrupt'),
  interrupt: z.unknown(),
});

export const CustomEvent = z.object({
  type: z.literal('custom'),
  event: z.unknown(),
});

export const DoneEvent = z.object({ type: z.literal('done') });
export const ErrorEvent = z.object({ type: z.literal('error'), message: z.string() });

export const StreamEvent = z.discriminatedUnion('type', [
  MetaEvent,
  TokenEvent,
  UpdateEvent,
  InterruptEvent,
  CustomEvent,
  DoneEvent,
  ErrorEvent,
]);
```

## SSE framing

Each event is sent as (minimal):

```
data: {"type":"token","content":"..."}

```

No SSE event name required; the `type` field drives handling.

### Optional: resumable SSE (recommended when supported)

If your infra supports SSE reconnection/resumption, include an event ID:

```
id: 42
data: {"type":"token","content":"..."}

```

On reconnect, browsers may send `Last-Event-ID`. Agent Server supports this for resumable streams when a run was created with `stream_resumable=true`.

## Mapping from LangGraph stream modes (Python)

- `messages` → emit `token` events
- `updates` → emit `update` events; if interrupt present, also emit `interrupt`
- `custom` → emit `custom` events

## HITL resume payloads

When HITL interrupts occur, the UI typically collects decisions:

- `approve`
- `reject` (with optional explanation message)
- `edit` (with edited tool args)

Resumption is backend-specific:

- Agent Server/SDK supports resuming runs/threads via standard APIs.
- In-process FastAPI should expose a resume endpoint or accept a resume payload in the stream endpoint.
