# Branch Finalization Review Framework

Use this reference for broad pre-merge, branch-completion, hard-cut,
dependency-native simplification, or merge-readiness work.

## Scoring Method

1. Apply hard gates from `SKILL.md` before scoring.
2. Select every profile materially touched by the branch.
3. Score each criterion from 0.0 to 10.0.
4. Multiply by the percentage weight and sum to a 0.0-10.0 lane score.
5. Keep lane scores separate. Add an overall readiness statement only after
   exposing weak lanes and blockers.
6. Change weights only for unusual evidence; state the replacement weights and
   rationale.

Action tiers:

- 9.0-10.0: preferred.
- 8.0-8.9: acceptable with explicit tradeoffs.
- 7.0-7.9: narrow, ask, defer, or research.
- Below 7.0: reject, redesign, or simplify further.

## Scoring Profiles

### Branch-Wide Product

| Criterion | Weight |
| --- | ---: |
| Workflow and user/operator value | 25 |
| Canonical architecture and deletion leverage | 20 |
| Security, data, and release safety | 20 |
| Cross-surface coherence | 15 |
| Verification confidence | 10 |
| Delivery simplicity | 10 |

### Backend

| Criterion | Weight |
| --- | ---: |
| Authorization, isolation, and data integrity | 25 |
| Canonical schema and API boundaries | 20 |
| Framework/database-native alignment | 20 |
| Performance and cost shape | 15 |
| Testability and release verification | 10 |
| Workflow value | 10 |

### Auth and Security

| Criterion | Weight |
| --- | ---: |
| Security, authorization, and isolation | 35 |
| Identity/session/data correctness | 20 |
| UX continuity and recovery safety | 15 |
| Platform and provider fit | 15 |
| Verification confidence | 10 |
| Simplicity and deletion | 5 |

### Web

| Criterion | Weight |
| --- | ---: |
| User/operator workflow value | 25 |
| Server boundary and authorization correctness | 20 |
| Framework/runtime alignment | 20 |
| UX and accessibility | 15 |
| Performance and cache behavior | 10 |
| Test/build verification | 10 |

### Mobile and Native

| Criterion | Weight |
| --- | ---: |
| Native workflow value | 25 |
| iOS/Android/framework correctness | 25 |
| Auth, session, link, and storage safety | 15 |
| UX, accessibility, and responsiveness | 15 |
| Build/release/update verification | 10 |
| Runtime-boundary simplicity | 10 |

### UI and UX

| Criterion | Weight |
| --- | ---: |
| Task clarity and workflow ergonomics | 30 |
| Product truth and copy | 20 |
| Accessibility and interaction quality | 20 |
| Design-system consistency and simplicity | 15 |
| Cross-surface coherence | 10 |
| Implementation risk | 5 |

### Integrations

| Criterion | Weight |
| --- | ---: |
| Idempotency, durability, and repairability | 25 |
| Security, trust boundaries, and compliance | 20 |
| Provider-native leverage and current API fit | 20 |
| Observability and operational recovery | 15 |
| Product value | 10 |
| Testability without live providers | 10 |

### QA and Testing

| Criterion | Weight |
| --- | ---: |
| Regression-detection value | 30 |
| Determinism and flake resistance | 20 |
| Repository validation and CI fit | 20 |
| Feedback speed | 15 |
| Test maintainability | 10 |
| Honest coverage | 5 |

### Docs and Operations

| Criterion | Weight |
| --- | ---: |
| Contract and source-of-truth accuracy | 25 |
| Release/operator safety | 25 |
| Reproducibility | 20 |
| Minimal durable documentation | 15 |
| Automation leverage | 10 |
| Churn containment | 5 |

## Lane Checklists

### Hard-Cut Shape

- One schema, API, route, helper, export, config owner, fixture, and test lane
  exists per concept.
- Producers, consumers, docs, generated output, snapshots, and tests use the
  canonical shape.
- No wrapper, shim, alias, fallback, dual-write/read, coercion, stale export,
  dead feature flag, or obsolete test remains without a named external boundary.
- Dependency-native capabilities replace equivalent custom infrastructure when
  behavior and ownership improve.
- Removed paths are deleted, not left deprecated indefinitely.

### Product Completeness

- The branch closes a coherent task, not merely internal plumbing.
- Primary, empty, loading, error, denied, destructive, recovery, and operator
  states are accounted for where relevant.
- Copy describes product concepts rather than implementation details.
- Missing work is classified as blocker, explicit follow-up, or intentionally
  excluded scope with rationale.
- No fabricated proof, silent failure, or inaccessible recovery path ships.

### Backend

- Inputs, outputs, and persisted data have one validated shape.
- Server-side authorization covers every protected read and write.
- Tenant/account scope cannot be supplied or widened by an untrusted client.
- Reads are indexed/bounded; writes are atomic where required; transitions and
  retries are idempotent.
- External side effects live at the proper boundary and have replay/repair
  behavior.
- Migrations, generated clients, and old readers/writers agree on the final
  contract or the external compatibility boundary is documented.

### Auth and Security

- Unauthenticated, authorized, denied-role, cross-tenant, expired-session, and
  recovery behavior are proven where applicable.
- Redirects and deep links do not leak tokens or bypass checks.
- Secrets remain outside code, client bundles, logs, fixtures, and artifacts.
- Webhooks verify authenticity before processing and deduplicate trusted event
  identifiers.
- Privacy-sensitive logs and analytics are minimized.
- Security gates cannot be waived by a high aggregate score.

### Web

- Routing, server/client boundaries, rendering mode, caching, and invalidation
  follow the actual framework contract.
- User-specific responses are not publicly cached.
- Hydration and error boundaries preserve a usable task.
- Accessibility, metadata, responsive behavior, and supported themes are
  verified in a browser.
- Assets, fonts, code splitting, and data fetching fit repository budgets.
- Production build and relevant E2E/browser lanes pass.

### Mobile and Native

- Navigation, links, safe areas, keyboard behavior, storage, permissions, and
  lifecycle transitions follow platform conventions.
- Touch targets, Dynamic Type, screen readers, appearance, and reduced motion
  are verified on a simulator or device.
- Native config, entitlements, plugins, build profiles, and update channels
  match the intended release path.
- Web proof is not substituted for native proof.
- Platform-specific behavior is isolated without duplicating domain logic.

### UI and UX

- The primary task and next action are obvious.
- Forms preserve values, explain errors, and recover safely.
- Loading, empty, error, offline, and denied states are intentional.
- Design tokens and shared components have one owner.
- Interaction state, accessibility, responsive layout, and themes are complete.
- Visual additions earn their complexity and do not hide weak information
  architecture.

### Integrations

- Current official provider docs/source support the API shape.
- Requests have safe idempotency, timeouts, retries, and failure classification.
- Provider identifiers, signatures, pagination, and event ordering are handled
  at the correct trust boundary.
- Tests use fakes/fixtures rather than live provider mutation.
- Operational repair, replay, reconciliation, and observability are defined.
- Billing, security, or provider configuration mutations require explicit user
  approval.

### Performance and Cost

- Common flows minimize network round trips and repeated server/provider calls.
- Reads are bounded and summarized rather than broad or per-row.
- Cache ownership and invalidation are explicit.
- Rendering, serialization, assets, and bundles avoid measured regressions.
- Background work has bounded concurrency, retries, and cost.
- Optimization claims include measurements or are marked `UNVERIFIED`.

### QA and Testing

- A narrow, deterministic proof covers each changed behavior and denied path.
- Tests assert behavior rather than implementation trivia.
- Obsolete compatibility tests are deleted with obsolete behavior.
- No wall-clock sleeps, live providers, shared mutable leakage, or test-only
  production branches are introduced.
- The narrow lane passes before wider repository/CI gates.
- Skipped gates and degraded environment prerequisites are explicit.

### Docs and Operations

- README, API/schema docs, ADRs/specs, setup, runbooks, and contributor guidance
  match changed behavior and ownership.
- Docs do not preserve deleted commands, paths, or compatibility promises.
- Environment and provider ownership is explicit; secrets are never copied into
  docs or logs.
- Release and rollback instructions match actual scripts.
- Generated references are regenerated through their owner, not hand-edited.
- Documentation churn is limited to durable contract changes.

## Decision Questions

Inspect repository evidence before asking the user. Ask only when the choice
changes product scope, public contracts, data shape, security posture, provider
commitment, release risk, or architecture ownership and user intent is required.

When needed:

- ask one to three independent questions;
- put the recommended option first;
- make options mutually exclusive;
- include active-profile scores and tradeoffs;
- offer a pre-mortem for an irreversible decision.

## Synthesis

Order closeout actions as follows:

1. hard-gate blockers;
2. coherent workflow completion;
3. canonical architecture and deletion;
4. performance/cost and dependency-native simplification;
5. narrow tests and wider validation;
6. docs/operations alignment;
7. optional low-risk polish.

Do not turn every observation into mandatory scope. A follow-up is justified
only when it has evidence, an owner, and a reason it should not block this
branch.
