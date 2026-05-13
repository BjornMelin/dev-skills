# UI integration: Next.js App Router + AI SDK v6 + Streamdown (React 19+)

This reference covers building a **production-grade chat UI** in **Next.js App Router** using:

- **Vercel AI SDK v6** (`ai`, `@ai-sdk/react`) for message/parts streaming and tool UIs
- **Streamdown** for streaming-safe Markdown rendering
- **Zod** for runtime validation and type inference
- **shadcn/ui + Tailwind** for UI components

Use this when your agent runtime is in **TypeScript/Node** (AI SDK core functions or ToolLoopAgent).

If your agent runtime is **Python** (LangGraph/LangChain in-process), prefer:

- **LangGraph Agent Server + `useStream`** (see `references/ui_nextjs_rsc.md`) to avoid writing a custom “protocol bridge”.

## Ground-truth rules (always)

1. Treat API surfaces as versioned; don’t code from memory.
2. Before implementing UI plumbing, verify:
   - Installed AI SDK versions (lockfile)
   - Current AI SDK stream protocol + hook API (Context7 + official docs)
3. If you must understand under-the-hood behavior, snapshot sources (read-only):
   - `npx opensrc ai@<VERSION> --modify=false`
   - `npx opensrc @ai-sdk/react@<VERSION> --modify=false`
   - `npx opensrc streamdown@<VERSION> --modify=false`

## Minimal architecture (recommended)

- `app/chat/page.tsx` (RSC): resolves auth/session and renders the client chat component
- `app/chat/ChatClient.tsx` (`"use client"`): `useChat` UI state + rendering
- `app/api/chat/route.ts` (server): `streamText(...)` and `toUIMessageStreamResponse()`

Why:

- Aligns with AI SDK’s **UI message/parts** model (tools, files, metadata)
- Keeps secrets server-side (route handler)
- Easy to add persistence and auth

## Server route handler pattern (AI SDK v6)

Core pattern:

- Parse `UIMessage[]` from request
- Convert to model messages with `await convertToModelMessages(messages)`
- Stream with `streamText(...)`
- Return `result.toUIMessageStreamResponse(...)`

Authoritative example snippet (AI SDK repo/docs) uses:

- `convertToModelMessages` (async in v6)
- `toUIMessageStreamResponse` for UI streaming responses

## Client pattern (AI SDK UI)

Use `useChat` with a `DefaultChatTransport`:

- chat surface is a client component
- render `message.parts` (text/tool results/files)
- use `status` for loading state and Streamdown `isAnimating`

## Tool calling + UI rendering (Zod-first)

Prefer Zod schemas for tools and (when feasible) for server→client custom events:

- Zod becomes the single source-of-truth for validation and TypeScript types
- Keep tool inputs narrow (principle of least privilege)

For tool UI patterns (invocations, results, approvals), load:

- `/home/bjorn/.codex/skills/ai-sdk-ui/references/tool-integration.md`
- `/home/bjorn/.codex/skills/ai-sdk-core/references/tool-calling.md`

## Message persistence + stream resumption

AI SDK UI is flexible, but **persistence is your responsibility**:

- Persist chat transcripts in your DB keyed by session/thread
- Restore messages on load (`initialMessages`) or via a fetch on mount
- Decide whether to persist **tool traces** or only user/assistant text

References:

- `/home/bjorn/.codex/skills/ai-sdk-ui/references/persistence.md`
- `/home/bjorn/.codex/skills/ai-sdk-ui/references/backend.md`
- `/home/bjorn/.codex/skills/ai-sdk-ui/references/production.md`

## Markdown rendering (Streamdown)

Streamdown is a streaming-optimized replacement for `react-markdown`.

Best practice:

- Render assistant `text` parts with Streamdown
- Set `isAnimating={status === "streaming"}` during generation
- Harden untrusted markdown/HTML with `rehype-harden` when you allow links/images

References:

- Streamdown skill: `/home/bjorn/.codex/skills/streamdown/SKILL.md`
- AI SDK integration: `/home/bjorn/.codex/skills/streamdown/references/ai-sdk-integration.md`
- Styling/security: `/home/bjorn/.codex/skills/streamdown/references/styling-security.md`

Tailwind integration notes live in the Streamdown skill:

- Tailwind v4: `@source "../node_modules/streamdown/dist/*.js";` in `globals.css`
- Tailwind v3: add `./node_modules/streamdown/dist/*.js` to `content`

## UI quality bar (shadcn/ui + Tailwind)

For polished, production-grade UI patterns (beyond a basic chat box), load:

- `/home/bjorn/.codex/skills/frontend-design/SKILL.md`

For AI SDK Agents workflow/UI patterns:

- `/home/bjorn/.codex/skills/ai-sdk-agents/SKILL.md`
- `/home/bjorn/.codex/skills/ai-sdk-agents/references/production.md`

## Bridging to a Python agent runtime (FastAPI / LangGraph)

If your agent runtime is Python, prefer **Agent Server + `useStream`**.

If you must use AI SDK UI anyway:

- Implement a Next.js “BFF” route that:
  1. Calls your Python backend
  2. Translates backend streaming events into AI SDK’s UI stream protocol

This is non-trivial and easy to get wrong (interrupts/resume, tool calls, threading).

For custom SSE event schemas, see:

- `references/ui_streaming_protocol.md`
- `references/ui_fastapi_backend.md`

## Templates shipped with this skill

- AI SDK v6 + Streamdown Next.js templates:
  - `assets/templates/nextjs_ai_sdk/app/api/chat/route.ts`
  - `assets/templates/nextjs_ai_sdk/app/chat/page.tsx`
  - `assets/templates/nextjs_ai_sdk/app/chat/ChatClient.tsx`
  - `assets/templates/nextjs_ai_sdk/app/chat/Response.tsx`

