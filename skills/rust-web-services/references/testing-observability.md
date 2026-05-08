# Testing and Observability

## Integration Tests

Use in-process service tests where possible:

- construct router with test state
- send HTTP requests through `tower::ServiceExt`
- assert status, headers, and response body
- use temporary databases or test containers only when the repo already supports them

Test:

- happy paths
- validation failures
- authorization failures
- not found/conflict semantics
- database transaction behavior
- timeout/upstream failure mapping

## Observability

Use structured tracing:

- request ID
- route/method/status
- authenticated principal or tenant when safe
- latency
- upstream call names
- error kind

Do not log secrets, bearer tokens, raw cookies, sensitive headers, or large request bodies by default.

## Readiness

Production services should have:

- health/readiness endpoints if deployed behind orchestration
- request body limits
- timeouts
- CORS policy when browser-facing
- metrics or traces wired to the deployment platform
- clear config failure messages

Keep readiness checks cheap and avoid using them to run expensive database migrations.
