---
name: design-motion-audit
description: "Audits a repo, route, screen, or 3D scene for motion quality, covering design-token consistency, frame-rate and draw-call risk, reduced-motion coverage, and accessibility. Use to audit, critique, or find motion gaps. Returns a prioritized punch list rather than implementing fixes."
license: MIT
---

# Design Motion Audit

Audit the target surface (a repo, route, screen, component, or 3D scene) for
motion quality and implementation safety, and return a **prioritized punch list**
with exact files, severity, reasoning, and concrete fixes. This skill diagnoses;
it routes implementation to the owning skill (`expo-motion`, `web-three-r3f`
for R3F setup and correctness, `r3f-scene-polish` for look-dev, or `gsap`). Token and motion-system work is handled here via
`references/design-system-tokens.md` and `scripts/scaffold_motion_tokens.py`.

## How to run

1. Optionally run the static analyzers to gather leads (treat findings as leads,
   verify each against the real code before reporting):

   ```bash
   python3 scripts/detect_motion_stack.py <project-root> --pretty   # what stacks/files exist
   python3 scripts/audit_motion_system.py <project-root> --pretty    # heuristic motion-quality scan
   ```

2. Read the flagged files and judge against the dimensions below.
3. Return the punch list, most-severe first.

## Audit dimensions

1. **Design tokens & consistency** — hardcoded durations/easings/springs vs
   tokenized values; naming by intent. (`references/motion-vocabulary.md`)
2. **R3F / three.js / WebGL** — `setState` in `useFrame`, missing delta-time,
   per-frame allocations, disposal, DPR, shadows, postprocessing budget.
3. **Reanimated / Expo / gestures** — JS-thread per-frame work, deprecated
   `runOnJS`/`runOnUI`, worklet misuse, interruptibility, layout-vs-transform.
4. **Interaction physicality** — velocity-aware release, cancellation-safety.
5. **Performance risk** — frame budget, draw calls, texture/asset weight, blur.
   (`references/performance-accessibility.md`)
6. **Reduced-motion coverage** — every camera move, parallax, loop, and bounce
   has a reduced-motion branch that preserves functional feedback.
7. **Accessibility & readability** — text legibility during motion.
8. **Missing hallmark opportunities** — where a signature motion would add value.

Score against `references/quality-gates.md`, and shape the output with
`references/report-template.md`. For any change with a visible motion surface, add
**runtime proof** (`references/runtime-verification.md`) — static findings alone do
not prove a scene renders or holds its frame budget; the `motion-runtime-verifier`
subagent (design-motion plugin) drives that when the browser tools are available.
For deep implementation fixes, hand each finding to the skill that owns its stack.
