# Security threat model (multi-agent + tool calling)

Use this when designing production agent systems or migrating a legacy stack.

## Primary threats

### Prompt injection / data exfiltration

- Malicious content attempts to override instructions and trigger unsafe tools.
- Retrieval sources can contain adversarial strings that look like tool directives.

Mitigations:

- strict tool allowlists, permissions, and argument validation
- isolate retrieval content; never treat it as instructions
- use middleware guardrails + HITL for sensitive actions

### SSRF / untrusted network access

- “fetch URL” tools can leak internal network data.

Mitigations:

- domain/prefix allowlists
- block loopback/private IP ranges
- enforce HTTPS and size limits

### Secrets leakage

- The model should never receive raw secrets; tool outputs may contain secrets.

Mitigations:

- inject secrets via runtime context (server-side DI)
- redact tool results before returning to model

### Destructive writes

- Agents can perform irreversible actions quickly.

Mitigations:

- human-in-the-loop for side effects
- least privilege credentials
- idempotent APIs + dry-run modes

## Security-by-default checklist

- Every tool has: schema, timeout, retries, and explicit side-effect declaration.
- Every side-effectful tool has: auth gating + HITL (unless explicitly exempt).
- Logs/traces: redaction + sampling + per-tenant isolation.

