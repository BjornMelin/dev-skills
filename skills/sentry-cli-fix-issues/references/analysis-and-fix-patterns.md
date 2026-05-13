# Analysis And Fix Patterns

Use this reference after initial evidence collection. The goal is to convert
Sentry signals into a verified code fix, not to summarize the issue forever.

## Universal Root-Cause Checklist

- Which release, environment, project, and first/last seen window are involved?
- What is the highest-confidence application frame?
- Is the selected frame from the deployed release or from stale/missing mapping?
- Are affected users, event count, and frequency still increasing?
- Do traces or logs show an upstream dependency, queue, database, cache, or
  timeout that explains the symptom?
- Did a recent commit, config change, dependency upgrade, deploy, feature flag,
  or migration touch the same path?
- Can the failure be reproduced with a unit, integration, or smoke test?

## Frontend And JavaScript

Common causes:

- Missing guards around optional data from loaders, API responses, or auth state.
- Hydration and lifecycle races after navigation or suspense boundaries.
- Source maps missing, mismatched, or uploaded under the wrong release/dist.
- Browser-only APIs accessed during server rendering.
- Error boundaries swallowing details without enough context.

Fix pattern:

1. Verify mapped source frames before editing.
2. Add the narrow guard, state transition, or error boundary behavior needed.
3. Add a test for the failing state, not just the happy path.
4. If source maps are wrong, fix release and upload workflow first.

## Backend And Jobs

Common causes:

- Invalid assumptions about nullable external payload fields.
- Non-idempotent retries and partial writes.
- Missing timeout, cancellation, or backoff boundaries.
- Queue workers acknowledging before durable work completes.
- Error wrapping that hides the actionable cause.

Fix pattern:

1. Reconstruct the failing input shape from redacted event context.
2. Check whether the operation is safe to retry.
3. Add validation or canonical normalization at the boundary.
4. Preserve useful error context without leaking secrets.

## Performance Regressions

Common causes:

- N+1 queries, unbounded fanout, missing indexes, slow external calls.
- High-cardinality span names or tags that explode observability cost.
- Retrying latency-sensitive calls without budget or cancellation.
- Logging too much per request or per token.

Fix pattern:

1. Identify the slowest repeated span or top transaction in traces.
2. Confirm whether latency is app-owned or upstream-owned.
3. Fix the algorithm/query/cache/batch boundary.
4. Add a regression test, query plan check, or focused benchmark where the repo
   has a pattern for it.

## Noisy Groups

Do not archive or filter before asking why the events are noisy.

Appropriate fixes:

- Better boundary validation so expected user errors are not reported as crashes.
- More precise grouping when distinct root causes are merged.
- Dropping low-value breadcrumbs/logs at the instrumentation layer.
- Sampling only after correctness and alerting impact are understood.

Inappropriate fixes:

- Hiding a real production defect with archive forever.
- Turning off capture for an entire subsystem because one issue is noisy.
- Filtering by broad exception class without a specific expected condition.

## AI And LLM Telemetry

Check both correctness and observability:

- Root spans must be sampled for child `gen_ai` spans to appear.
- Token and cost attributes may be absent for streaming unless usage reporting is
  enabled by the model provider or SDK call.
- Prompt and completion capture can contain sensitive data. Treat captured text
  as private production data.
- Tool-call loops and retries can multiply spend. Check retry counts, model
  names, tool spans, and timeout boundaries.

Fix pattern:

1. Verify tracing is enabled and sampled for the AI route or worker.
2. Confirm model, provider, tokens, cost, and tool call attributes exist.
3. Add instrumentation around app-owned agent steps when automatic integration
   does not cover them.
4. Avoid enabling prompt/output capture by default in code changes.

## Release And Mapping Failures

If Sentry points to minified or missing frames:

- Compare event `release` and `dist` to SDK initialization and deploy metadata.
- Check whether debug IDs were injected into the deployed bundle.
- Check whether artifacts were uploaded for the same release and URL prefix.
- Use source-map debug API output as evidence before changing upload scripts.

Fix pattern:

1. Fix deterministic release naming.
2. Inject debug IDs before upload where supported.
3. Upload artifacts from the exact build output used for deployment.
4. Add CI validation or release script checks to prevent drift.

## Error Handling Quality Bar

- Capture exceptions where the app intentionally catches and continues.
- Add context with low-cardinality tags and safe extras.
- Do not attach raw secrets, headers, cookies, prompts, or customer payloads.
- Prefer typed/domain errors over string matching.
- Preserve stack traces when wrapping errors.
