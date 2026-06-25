# Input Validation

Every network boundary needs runtime validation. Prefer Zod v4 schemas when a schema is already part of the app contract; otherwise use a narrow function validator.

## Rules

- Validate every `data` payload passed to server functions.
- Reject or strip fields the client must not control, such as role, owner id, or spend limits.
- Treat parsed IDs as shape-valid only; re-check ownership against the session principal before querying or mutating.
- Share schemas with forms only when the same trust boundary and field set are correct.
- Use `FormData` validators for form submissions instead of asserting object shapes.
