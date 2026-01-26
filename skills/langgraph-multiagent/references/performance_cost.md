# Performance + cost optimization

## 1) Context discipline

- Prefer retrieval over large static prompts.
- Summarize aggressively at stable checkpoints.
- Use subagents as context isolation boundaries.

## 2) Cache the right things

- Deterministic tool results: cache by (tool name, args hash, user scope).
- Retrieval: cache embeddings + vector results when acceptable.
- Long-term memory: store normalized facts/preferences, not transcripts.

## 3) Model routing

- Small model for routing/validation; large model for synthesis.
- Enforce max tool calls / max steps / recursion limits.

## 4) Parallelism safety

- Parallelize only when state updates can be merged deterministically.
- Use reducers to avoid concurrent update errors.
- Prefer “fanout results key” + aggregation.

## 5) Reliability tactics

- Timeouts + retries with jitter for flaky dependencies.
- Circuit breakers for repeated tool failures.
- “Fail open” only for non-critical features (e.g., optional rerank).

