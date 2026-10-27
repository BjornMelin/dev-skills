# QA test plan template (copy/paste)

## 1) Scope

- Feature / component:
- Public behavior / contract:
- Out of scope:

## 2) Risks

- What could break?
- What is most expensive to debug in production?
- What is performance sensitive?

## 3) Test matrix

### Unit tests (fast, deterministic)

- [ ] Happy path
- [ ] Boundary conditions
- [ ] Error handling
- [ ] Invariants

### Component tests (jsdom)

- [ ] Rendering
- [ ] Accessibility queries (roles/labels)
- [ ] User interactions
- [ ] Loading/error UI states

### Integration tests (node)

- [ ] Module boundaries cooperate
- [ ] Database/network adapters are mocked/faked

### E2E (if needed)

- [ ] Async Server Components (Next)
- [ ] Routing / auth / real browser behavior

## 4) Non-functional requirements

- [ ] Tests deterministic (no real network/clock)
- [ ] Tests run with `vitest run` (CI-like)
- [ ] Timeouts and concurrency are reasonable
- [ ] Coverage expectations documented (if enforced)

## 5) Exit criteria

- [ ] All tests pass locally and in CI
- [ ] Failures are actionable
- [ ] Flake check performed for risky areas (optional)
