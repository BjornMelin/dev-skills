# Tauri v2 Rust Backend

## Command Shape

Keep commands as thin adapters:

1. Deserialize DTOs.
2. Validate frontend-provided values.
3. Call an app-owned service.
4. Map domain result/error into a stable IPC response.

Avoid embedding business logic, filesystem traversal, or subprocess orchestration directly in command functions.

## IPC Contracts

- Use explicit request/response structs for anything beyond trivial commands.
- Keep field names stable and documented through TypeScript bindings or generated fixtures where the repo supports it.
- Return structured errors instead of stringly catch-all failures when the frontend branches on error kind.
- Do not leak internal paths, tokens, or raw subprocess output unless the UI explicitly needs them.

## App State

Use `tauri::State` for shared clients, stores, and services. Make ownership clear:

- immutable config
- async clients with internal pooling
- mutex-protected mutable state only when truly shared
- channels for background workers

Avoid global mutable statics. If singletons are needed, initialize them in app setup and inject them through state.

## Plugins

Prefer maintained Tauri plugins for common platform features. Before adding a plugin:

- Check current Tauri v2 compatibility.
- Review required permissions.
- Confirm mobile/desktop support if cross-platform is needed.
- Add only the plugin surface actually used.

## Frontend Bridge

Keep invoke names centralized in frontend code. For larger apps, generate or hand-maintain a small typed client module rather than scattering raw string command names.
