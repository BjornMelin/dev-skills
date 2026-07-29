# Events, Async, and Terminal Safety

## Event Loop

Use one clear owner for terminal setup and teardown:

- enter alternate screen
- enable raw mode
- install panic/error restoration if needed
- run loop
- restore terminal on every exit path

Avoid early returns between setup and cleanup unless guarded by RAII.

## Async Work

Do not block the input/render loop. For async applications:

- Use `tokio` tasks for network or disk-heavy operations.
- Send task results back as typed events.
- Use bounded channels to avoid unbounded memory growth.
- Use cancellation tokens for work tied to a screen or query.
- Debounce high-frequency inputs such as search.

## Ticks and Rendering

Render on meaningful changes where possible. If using ticks, keep the interval intentional and cheap.

Handle resize events explicitly. Recompute layouts and clamp focus/scroll state after size changes.

## Error Handling

Terminal restoration is more important than pretty errors. Restore first, then report.

Use domain errors for recoverable failures and keep fatal terminal/runtime errors rare and well explained.
