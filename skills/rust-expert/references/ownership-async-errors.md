# Ownership, Async, And Errors

Use this reference when the task involves borrow checker errors, lifetimes,
mutability, async `Send` failures, task lifecycle, error boundaries, or
recovery behavior.

## Borrow Checker Repair Order

1. Reproduce the exact compiler diagnostic.
2. Read the owning type, caller, and intended invariant.
3. Narrow borrow scopes before changing types.
4. Move behavior onto the owner when mutation authority belongs there.
5. Split data structures when independent fields are being borrowed together.
6. Borrow or move deliberately; clone only when shared ownership or retained
   values are semantically required.

Avoid:

- `.clone()` just to silence E0382.
- `Arc<Mutex<_>>` as a generic Send/Sync fix.
- `RefCell` where ordinary ownership works.
- Public fields that let callers violate invariants.
- `unsafe` to bypass ownership errors.

## Async Design

Ask what kind of work this is:

| Work | Prefer |
| --- | --- |
| I/O-bound concurrency | Tokio when repo already uses it or networking requires it |
| CPU-bound parallelism | `rayon`, dedicated threads, or `spawn_blocking` |
| One owner, many messages | channels or actor/task owner |
| Shared immutable config | `Arc<T>` |
| Shared mutable state | redesign first, then narrow locks if needed |

Rules:

- Do not block the executor with blocking I/O, sleeps, or CPU-heavy loops.
- Do not hold lock guards, borrowed refs, or non-Send values across `.await`
  unless the architecture explicitly supports it.
- Prefer bounded channels for backpressure.
- Use `JoinSet` or owned task groups for related spawned work.
- Use `tokio::select!` for cancellation, shutdown, timeouts, and races.
- Treat dropped channels as expected shutdown signals when appropriate.

## Error Boundaries

| Context | Default |
| --- | --- |
| Public library/module API | typed `thiserror` enum/struct |
| Application internals | `anyhow` or `miette`/`color-eyre` context |
| CLI/user boundary | stable exit code, kind, operation, message, cause chain |
| Service boundary | typed status/error envelope, no internal leakage |
| Bug/invariant | `panic!`, `assert!`, or precise `expect` message |

Do not use `anyhow` to erase errors that drive retry behavior, CLI exit codes,
JSON contracts, public APIs, or operator remediation.

## Recovery Rules

- Invalid input: no retry; return an actionable error.
- Timeout, 503, rate limit: bounded retry with backoff and jitter.
- Invalid config: fail fast.
- Data corruption: stop and surface high-severity error.
- Dependency unavailable: timeout/circuit/fallback only if product semantics
  allow degraded behavior.
