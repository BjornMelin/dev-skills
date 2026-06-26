# Error Handling

Return predictable errors at the right layer.

## Rules

- Throw `redirect` or `notFound` from Router/Start APIs when navigation semantics are intended.
- Convert sensitive server errors into safe client-facing messages.
- Log with request context on the server, not in client components.
- Keep validation errors actionable and field-specific where user input caused the failure.
- Do not expose provider secrets, raw webhook payloads, or internal stack traces.
