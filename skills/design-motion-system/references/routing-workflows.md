# Routing and Workflow Reference

## Route by request type

- Creative concept, hierarchy, choreography: `motion-design-director`.
- R3F, three.js, Drei, GLB, WebGL: `r3f-scene-architect`.
- Expo, iOS, Reanimated, gestures: `reanimated-ios-motion-engineer`.
- Tokens, constants, CSS variables, Tailwind: `motion-token-systems-integrator`.
- Performance, FPS, reduced motion, accessibility: `motion-performance-auditor`.
- Final acceptance criteria: `motion-qa-reviewer`.

## Use normal skill execution when

- One component, route, screen, or scene is involved.
- The task needs a plan or implementation in a focused area.
- The agent can inspect relevant files directly.

## Use forked subagents when

- Research would clutter the main conversation.
- Specialist judgement is needed.
- The task needs independent review.
- The user asks for audit or deep plan.

## Use dynamic workflows when

- The user says repo-wide, all routes, every animation, complete audit, large migration, or many components.
- The task should fan out across directories or packages.
- Independent agents should cross-check each other.
- You need repeatable orchestration in a script.
- Cost is acceptable or the task can begin with a small slice.

## Workflow stage template

1. Inventory directories and classify stack.
2. Fan out audits by package, route, screen, and scene.
3. Summarize findings by stack.
4. Have specialists propose fixes.
5. Have performance auditor challenge expensive or unsafe ideas.
6. Have QA reviewer create acceptance criteria.
7. Implement in small slices.
8. Verify after each slice.
9. Produce final report.
