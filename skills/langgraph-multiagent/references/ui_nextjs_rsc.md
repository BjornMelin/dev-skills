# UI integration: Next.js App Router (RSC) + React + shadcn/ui + Tailwind + Zod

This guide covers **production-grade UI integration** for LangGraph/LangChain multi-agent systems in a modern Next.js stack.

## Recommended architecture options (pick one)

### Option A (recommended for LangGraph-based apps): LangGraph **Agent Server** + `useStream`

Use LangGraph Agent Server (local `langgraph dev`, or LangSmith Deployment) and connect from Next.js using:

- `@langchain/langgraph-sdk/react` → `useStream()` (messages, interrupts, branching, typed state)
- optional `@langchain/langgraph-sdk/react-ui` for **generative UI components** (`LoadExternalComponent`)
- optional “Agent Chat UI” for a ready-made baseline chat surface (then layer custom UI)

Why:

- Built-in thread persistence (thread IDs)
- Standard streaming API + SDKs
- Works well with multi-agent graphs, subgraphs, HITL, and long-term memory store

### Option B: Next.js route handlers + AI SDK UI (`useChat`)

Use this when your backend is **not** an Agent Server (custom FastAPI, custom infra), or you want AI SDK’s message/parts ecosystem.

Tradeoffs:

- You own thread persistence and HITL protocols unless you integrate with Agent Server.
- More custom glue when your agent runtime is Python.

For an end-to-end AI SDK v6 + Streamdown reference (and templates), see:

- `references/ui_nextjs_ai_sdk.md`

## Next.js app/router patterns (RSC-safe)

- Keep the “chat surface” as a **client component** (`"use client"`).
- Keep auth/session/user resolution in **server components** (RSC) and pass only non-sensitive IDs to the client.
- Never ship secrets to the browser. Use server-side tokens (cookies/session) or proxy through your server.

## Thread IDs (stateful conversations)

With `useStream`, you can:

- provide `threadId` to resume an existing thread
- use `onThreadId` callback to persist the created thread ID

Best practice:

- persist `threadId` in URL query params or `localStorage`
- include user/org identifiers via safe headers or server-side injection
- for optimistic routing (navigate before the thread exists), generate a UUID and pass it via `submit(..., { threadId })`

Example thread management pattern:

```tsx
const [threadId, setThreadId] = useState<string | null>(null);

const stream = useStream({
  apiUrl: process.env.NEXT_PUBLIC_LANGGRAPH_API_URL ?? "http://localhost:2024",
  assistantId: "agent",
  threadId,
  onThreadId: setThreadId,
});
```

## Zod: validate UI↔backend contracts

Use Zod for:

- validating tool-call payloads shown in UI
- validating custom streaming events (progress, interrupts)
- validating “resume” decisions for HITL

Keep Zod schemas as the single source-of-truth in the frontend and generate types from them.

## Human-in-the-loop UI

Two levels:

1. **Simple approvals**: show tool name + args + “Approve/Reject”.
2. **Editable approvals**: allow editing args (restricted fields) when `edit` is allowed.

Implementation detail:

- LangChain HITL interrupts surface as `__interrupt__` in state/updates; resumption uses a `Command(resume={decisions: [...]})` pattern on the backend.
- In Agent Server + `useStream`, use `stream.interrupt` + `stream.submit(null, { command: { resume: { decisions } } })`.

Minimal HITL resume sketch:

```tsx
const hitlRequest = stream.interrupt?.value;
await stream.submit(null, { command: { resume: { decisions: [{ type: "approve" }] } } });
```

## Multi-agent UI ergonomics

Multi-agent systems often produce outputs from different nodes/agents.

UI best practices:

- show “agent badges” using message metadata (node/agent name)
- collapse tool-call traces by default (expandable)
- stream progress updates (“custom” stream mode) into a small status bar

To label messages, use `stream.getMessageMetadata(message)` and read fields like `langgraph_node` (exact keys are versioned; consult the streaming frontend docs).

## shadcn/ui + Tailwind guidelines

- Treat the chat UI as a “log viewer” + “command input”.
- Use monospace blocks for tool arguments and structured data.
- Keep interrupts visually distinct (danger border + explicit action buttons).

## Advanced: resume after refresh, branching, and custom transport

The streaming frontend docs cover:

- `reconnectOnMount` for auto-resuming an in-flight run after refresh
- `onCreated` / `onFinish` callbacks for manual resumption and run-id persistence
- `stream.switchBranch(...)` for navigating conversation forks
- `FetchStreamTransport` for adding auth headers or shaping requests without changing UI code

Use this when you need a Next.js “BFF” route (server-side cookies/session) but still want `useStream` on the client.

## Templates shipped with this skill

- Next.js `useStream` chat page:
  - `assets/templates/nextjs/app/chat/page.tsx`
  - `assets/templates/nextjs/app/chat/ChatClient.tsx`
  - `assets/templates/nextjs/app/chat/types.ts`
  - `assets/templates/nextjs/app/chat/Response.tsx` (Streamdown renderer; install `streamdown` to use)

These templates assume you have shadcn primitives available at `@/components/ui/*`.

- Next.js AI SDK v6 chat page (alternative stack):
  - see `references/ui_nextjs_ai_sdk.md`
  - templates under `assets/templates/nextjs_ai_sdk/`

## Markdown rendering (recommended): Streamdown

Agent responses often contain Markdown (tables, lists, code blocks). Prefer Streamdown for chat UIs because it is streaming-friendly (handles incomplete Markdown as tokens stream).

Deep-dive references:

- Streamdown skill: `/home/bjorn/.codex/skills/streamdown/SKILL.md`
- Streamdown + AI SDK patterns: `/home/bjorn/.codex/skills/streamdown/references/ai-sdk-integration.md`
- Streamdown security + styling: `/home/bjorn/.codex/skills/streamdown/references/styling-security.md`
- Streamdown API reference: `/home/bjorn/.codex/skills/streamdown/references/api-reference.md`

## Cross-skill UI references (load on demand)

If you’re building a Next.js UI and want best-practice guidance for the UI layer beyond LangGraph:

- AI SDK Core: `/home/bjorn/.codex/skills/ai-sdk-core/SKILL.md`
- AI SDK UI: `/home/bjorn/.codex/skills/ai-sdk-ui/SKILL.md`
- AI SDK Agents: `/home/bjorn/.codex/skills/ai-sdk-agents/SKILL.md`
- Frontend quality + aesthetics: `/home/bjorn/.codex/skills/frontend-design/SKILL.md`
