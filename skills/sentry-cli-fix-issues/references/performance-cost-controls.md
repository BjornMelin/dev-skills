# Performance And Cost Controls

Use this reference when Sentry evidence indicates high event volume, trace
volume, replay volume, log volume, AI token spend, or noisy alerts.

## First Principles

- Do not reduce cost by hiding an unresolved correctness bug.
- Fix high-cardinality names and tags at the instrumentation boundary.
- Tune sampling with a clear observability objective.
- Keep enough data to debug regressions and prove recovery.

## Error Volume

High event volume usually means one of:

- a real production defect in a hot path
- an expected user/input condition reported at the wrong severity
- a retry loop or worker loop amplifying one failure
- duplicate capture in framework and app code
- overly broad logging or exception wrapping

Fix the owner first. Use filters or archive actions only after the root cause is
known and the signal is truly low value.

## Traces And Spans

Check:

- `traces_sample_rate` or sampler behavior
- transaction naming cardinality
- span operation and description cardinality
- root span sampling for important child spans
- whether logs/replays/traces are linked by trace IDs

Prefer:

- route templates over raw URLs
- stable operation names over dynamic strings
- low-cardinality tags over arbitrary payload fields
- targeted sampling for critical flows

## Replays

Replay can be high value and high volume.

Use replay evidence when issue context includes rage clicks, dead clicks, console
errors, or user path confusion. Do not enable broad replay capture as part of a
bug fix unless the user asked for instrumentation changes and privacy settings
are explicit.

## Logs

Logs should answer "what happened around this trace?" without duplicating full
payloads.

Reduce cost by:

- lowering noisy expected states to debug/info
- sampling repetitive logs
- adding stable event IDs instead of dumping objects
- linking logs to traces rather than copying context into every line

## AI Token And Model Cost

For AI systems, inspect:

- model/provider names
- input/output token attributes
- retry counts
- tool-call and handoff spans
- timeout and cancellation behavior
- streaming usage reporting

Useful fixes:

- add token/cost capture where missing and privacy-safe
- cap retries and tool-loop iterations
- add cheaper fallback models only when product quality allows it
- sample traces for high-value AI routes intentionally

Do not enable prompt or completion capture by default. If it is already enabled,
handle captured text as sensitive production data.

## Alerting And Grouping

Before changing alerting or grouping:

- confirm who consumes the alert
- confirm expected event/user thresholds
- check whether issue grouping merges unrelated root causes
- document the reason in the final report

Good outcomes:

- fewer duplicate reports
- stable, actionable issue groups
- enough traces/logs/replays to debug the next regression
- no broad blind spots in critical flows
