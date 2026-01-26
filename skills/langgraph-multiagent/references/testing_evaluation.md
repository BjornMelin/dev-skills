# Testing + evaluation for multi-agent systems

Goal: keep migrations safe and prevent regressions as models/versions change.

## Test pyramid

### 1) Tool unit tests (fast, offline)

- Validate schemas and edge cases.
- Ensure idempotency where required.
- Mock external IO; record deterministic fixtures.

### 2) Graph/agent integration tests (offline-first)

- Run the compiled graph with a stub model (or mocked model interface).
- Assert:
  - correct tool selection / ordering
  - correct state transitions
  - correct handling of tool failures
  - correct interrupt behavior (HITL)

### 3) End-to-end tests (selective)

- Only for critical paths.
- Capture traces and verify invariants (latency, tool count, stop reason).

## Regression evaluation (LangSmith-style)

Recommended:

- maintain a dataset of canonical prompts and expected tool actions
- run evaluations on every dependency upgrade
- keep “golden traces” for critical flows

## Migration safety protocol

1. Add tests before refactoring orchestration.
2. Migrate in slices:
   - tools → state → routing → memory → deployment
3. Keep observability on during rollout.

