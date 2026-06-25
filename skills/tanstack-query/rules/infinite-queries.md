# Infinite Queries

Use infinite queries only for paginated lists with clear page params.

## Rules

- Provide `initialPageParam`.
- Implement `getNextPageParam`.
- Guard `fetchNextPage` with `hasNextPage` and `isFetchingNextPage`.
- Consider `maxPages` for large lists.
- Keep finite and infinite query keys distinct.
