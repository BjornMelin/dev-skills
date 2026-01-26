# Guardrails + human-in-the-loop (HITL)

## Design goal

Minimize irreversible failures and unsafe behavior by combining:

- deterministic checks (fast, cheap, predictable)
- model-based checks (semantic, slower)
- human approvals for high-stakes actions

## Where to apply guardrails

- Before model calls (prompt shaping, policy constraints, context filtering)
- After model calls (output validation, structured extraction checks)
- Around tool calls (argument validation, allowlists, approval gates, rate limits)

## HITL rules of thumb

Require approval for:

- destructive writes (delete/update production data)
- outbound communication (email, SMS, Slack, social)
- payments/transfers
- privileged system actions (shell, infra changes)

Prefer middleware-based HITL when available (clean separation of policy from business logic). Use graph `interrupt` when you need explicit pause/resume at a specific node boundary.

