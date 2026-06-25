# Optimistic Updates

Choose optimistic strategy based on scope.

## Rules

- Use UI-level optimistic `variables` for simple local pending rows.
- Use `onMutate` cache updates when multiple components must reflect the optimistic state.
- Snapshot previous cache data and return rollback context.
- Roll back on error and invalidate on settle when server truth may differ.
- Avoid optimistic updates for security-sensitive state transitions unless conflicts are well-defined.
