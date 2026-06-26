# Disabling and Lazy Queries

Prefer declarative dependency control.

## Rules

- Use `enabled` for queries waiting on required inputs.
- Use `skipToken` for TypeScript-safe disabling when manual `refetch` is not needed.
- Remember `skipToken` cannot be manually refetched.
- Do not imperatively call queries from event handlers when a mutation or state transition better models the action.
