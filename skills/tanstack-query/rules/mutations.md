# Mutation lifecycle

- Use the object form of `useMutation` with an explicit `mutationFn` and typed variables.
- Put cache updates in `onSuccess`, invalidation in `onSettled` when both success and rollback need a refresh, and rollback logic in `onError`.
- Await invalidation when the UI must stay pending until related queries are fresh.
- Use `mutationKey` and `useMutationState` for shared pending UI across components instead of lifting duplicate local state.
- Keep navigation, toast, analytics, and cache writes in deliberate lifecycle callbacks; avoid hiding side effects inside API clients.
- Prefer server-confirmed data with `setQueryData` after success when the response contains the updated entity.
