# Context personalization (Agents SDK): state + memory notes

This reference summarizes a practical “context engineering” approach for personalization using the OpenAI Agents SDK:

- keep a **structured state object** outside the model (your source of truth),
- distill candidate memories during a run into **session notes**,
- consolidate session notes into **long-term notes** at the end (dedupe + conflict rules),
- inject only the **relevant slice** of state into the model at the start of each run.

Use it when you want an agent to feel consistent across sessions without turning the conversation transcript into your only memory store.

## Two kinds of state

1. **Session state (short-lived)**
   - Captures “useful for this run” context.
   - Can be re-injected when you trim/summarize history.
   - Should be small and aggressively curated.

2. **Long-term memory (durable notes + profile)**
   - Stable preferences and constraints (diet, locale, tone, accessibility needs).
   - Things that remain true across sessions, with clear recency rules.
   - Stored in your DB (SQLite is usually enough).

## The lifecycle (distill → consolidate → inject)

1. **Inject (start of run)**
   - Build system instructions that include:
     - a structured profile block (small, stable fields)
     - a short memory note list (top-K by relevance + recency)
   - Precedence rule (recommended): **current user input > session context > long-term memory**.

2. **Distill (during run)**
   - Use a dedicated tool (e.g., `save_memory_note`) to capture candidate notes.
   - Only store durable preferences/constraints; avoid transient facts.
   - Save to a session-scoped buffer first (so you can reject/curate).

3. **Consolidate (end of run)**
   - Merge session notes into long-term memory with:
     - deduplication
     - conflict resolution (often “latest wins”)
     - optional TTL/forgetting
   - Clear the session buffer after commit.

## Agents SDK primitives to use

- **Python**: `RunContextWrapper[T]` lets tools and glue code access your state object. It is not passed to the LLM directly.
- **Python sessions**: `SQLiteSession` persists conversation history across runs.
- **TypeScript sessions**: `OpenAIConversationsSession` (durable) or `MemorySession` (local dev); `sessionInputCallback` for deterministic trimming/merging.

## Guardrails (treat memory as an attack surface)

Memories are effectively “instructions-adjacent” once injected, so treat the pipeline as high risk.

Distillation checks (write-time):

- reject sensitive strings (SSNs, payment details, secrets)
- reject instruction-shaped content (“ignore previous instructions…”, “store this policy…”)
- constrain the note schema to allow only approved fields

Consolidation checks (merge-time):

- “no invention”: do not add facts not present in session notes
- resolve conflicts explicitly (document the rule)
- dedupe aggressively

Injection checks (read-time):

- wrap memory in explicit delimiters (e.g. `<memories>…</memories>`)
- enforce precedence: user intent overrides memory
- keep token budget small (top-K, short notes, summarize old notes)

## Suggested note schema (minimal)

Store each memory note as a small structured object:

- `type`: `"preference"` | `"constraint"` | `"fact"` (avoid `"instruction"`)
- `key`: stable identifier (e.g. `"diet"`, `"seat_preference"`)
- `value`: short string or small JSON
- `confidence`: 0..1
- `source`: `"user"` | `"system"` | `"inferred"` (prefer `"user"`)
- `created_at`: ISO timestamp

In SQLite, keep the source-of-truth in your DB; only inject what’s needed per run.

## Evals and logging (don’t guess)

Track whether personalization helps without harming correctness:

- memory_write_rate (too high usually means noisy capture)
- blocked_write_rate (signals adversarial/sensitive writes)
- conflict_rate (how often user overrides memory)
- time_to_personalization (turns until correct preference is applied)

Use tracing to correlate outcomes with memory injection and tool calls.

