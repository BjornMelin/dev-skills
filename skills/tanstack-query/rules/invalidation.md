# Invalidation

Use targeted invalidation after mutations that affect cached data.

## Rules

- Invalidate the smallest stable key that covers changed data.
- Return or await invalidation promises when mutation pending state should include the refetch.
- Prefer `setQueryData` for precise local cache updates when the mutation response contains complete replacement data.
- Avoid invalidating all queries unless the mutation truly affects global state.
