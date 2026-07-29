---
name: rust-web-services
description: Implicit Rust web service skill. Use for Axum, Tokio, Tower, Hyper, REST APIs, middleware/layers, request extractors, typed errors, tracing, config, graceful shutdown, SQLx, background jobs, service tests, and production Rust HTTP architecture.
license: MIT
metadata:
  category: rust
  domains:
    - web-services
    - axum
    - tokio
    - tower
---

# Rust Web Services

Build Rust HTTP services with explicit state, typed boundaries, observable behavior, and production-grade runtime contracts.

## Operating Model

1. Read the existing service shape first: router setup, app state, middleware layers, config, database access, tests, deployment scripts, and observability.
2. Prefer `axum` plus `tower` layers for new HTTP services unless the repo already standardizes on another framework.
3. Keep handlers thin. Extract inputs, call domain/service code, map results into HTTP responses.
4. Make app state explicit and clone-cheap. Avoid hidden globals and ad hoc connection creation per request.
5. Treat errors, tracing, timeouts, request size limits, and shutdown as part of the feature, not afterthoughts.

## Reference Map

- `references/axum-tower-architecture.md` for router structure, state, extractors, middleware, errors, and versioned APIs.
- `references/runtime-config-data.md` for Tokio runtime concerns, config, database access, background jobs, and graceful shutdown.
- `references/testing-observability.md` for integration tests, tracing, metrics, contract tests, and production readiness.

## Defaults

- Use `tokio` with only required features unless the repo uses full feature sets consistently.
- Use `serde` DTOs at HTTP boundaries and domain types internally.
- Use `thiserror` for domain errors and typed response mapping at the HTTP edge.
- Use `tracing`/`tracing-subscriber` with structured spans and request IDs.
- Use `sqlx` for database-backed services when compile-time query checking and async pooling are useful.

## Verification

For service changes:

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

Add integration tests for new routes, error status mapping, auth/authorization branches, config parsing, database behavior, and graceful shutdown or background work when touched.
