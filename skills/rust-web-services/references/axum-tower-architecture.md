# Axum and Tower Architecture

## Router Shape

Keep router construction boring and discoverable:

- one root `Router`
- scoped route modules for feature areas
- explicit app state type
- middleware/layers applied at the narrowest sensible scope
- versioning when API compatibility matters

Avoid route handlers that create clients, parse global config, or own long-running background work.

## Handlers

Handlers should:

1. Extract typed path/query/body/state.
2. Validate boundary DTOs.
3. Call domain/service code.
4. Map domain results to response DTOs.
5. Map errors to status codes in one consistent layer.

Do not put database query strings, retry loops, or authorization policy scattered across many handlers.

## State

Use an app state struct with clone-cheap members:

- database pools
- HTTP clients
- config snapshots
- service structs
- telemetry handles

Use `Arc` where sharing is needed. Avoid `Mutex` around state that should be modeled as a database transaction, channel, cache, or service.

## Middleware and Layers

Use `tower` layers for cross-cutting concerns:

- request IDs
- tracing
- timeouts
- compression
- body limits
- auth context extraction
- CORS when needed

Keep authorization decisions close to domain requirements. Middleware can authenticate and attach identity, but route/domain code should still enforce resource-specific authorization.

## Errors

Define typed domain errors and map them to HTTP responses at the boundary. Include stable error codes when clients branch on failures. Avoid leaking internal database or upstream error text to clients.
