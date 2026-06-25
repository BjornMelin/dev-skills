# Mutation Invalidation

Coordinate mutations with Query and Router after writes.

## Rules

- For Query-owned data, invalidate targeted query keys after successful mutations.
- Await invalidation if pending UI should last through refetch.
- If route loader deps changed, navigate with updated params/search instead of manual cache surgery.
- For Convex mutations, prefer Convex reactive updates over duplicate Query invalidation unless using the Convex Query adapter.
