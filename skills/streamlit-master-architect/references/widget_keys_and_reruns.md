# Widget keys + rerun traps

## Keys (stability rules)

- Use explicit `key=` for any widget created in a loop, conditional, or dynamic layout.
- Treat keys as part of app state schema; changing keys is a breaking change for persisted widget state.

## Common rerun traps

- Creating/removing widgets conditionally without stable keys (state gets “lost”).
- Expensive computations in the top-level script without caching.
- Using `st.download_button` with large inline `data=` bytes (compute on every run).
- Accumulating elements inside fragments without containers (`st.empty()`).

## Rerun control patterns

- Gate expensive work behind a button or form submit.
- Early exit on validation errors: `st.error(...); st.stop()`.
- Use `st.query_params` for shareable navigation/filters.

