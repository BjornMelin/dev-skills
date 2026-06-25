# Query Keys

Query keys identify cached server state.

## Rules

- Use top-level arrays.
- Include every variable that changes the query result.
- Keep keys JSON-serializable.
- Organize hierarchically from domain to entity to filters.
- Do not reuse the same key shape for finite and infinite queries.
