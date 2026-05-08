# Runtime, Config, and Data

## Tokio

- Spawn tasks deliberately and keep cancellation paths.
- Use `JoinSet` or tracked task handles for groups of background workers.
- Apply timeouts to external calls.
- Avoid blocking CPU or filesystem-heavy work on async worker threads; use `spawn_blocking` or a dedicated worker model when necessary.

## Configuration

Parse config once at startup into typed structs. Keep required settings explicit and fail fast. Redact secrets when logging effective config.

Use environment variables for deploy-time settings and config files only when the deployment model needs them. Do not let handlers read env vars per request.

## Database

For `sqlx`:

- Own one pool in app state.
- Use transactions for multi-step mutations.
- Prefer compile-time checked queries where repo workflow supports the required database metadata.
- Keep migrations reviewed and tested.

For background jobs:

- Make jobs idempotent.
- Use leases or unique constraints for duplicate prevention.
- Add retry/backoff policy instead of unbounded loops.

## Graceful Shutdown

Shutdown should:

- stop accepting new work
- signal background tasks
- wait with a timeout
- flush traces/logs when practical
- close resources cleanly

Test shutdown-adjacent code when adding long-running tasks or worker pools.
