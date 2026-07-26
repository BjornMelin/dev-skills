---
name: design-motion-system
description: Cross-stack motion & design-system director. Use to turn vague "make it premium / cinematic / standout / hyperrealistic / native-feeling / hallmark-quality" requests into a named motion vocabulary, design tokens, and a routed implementation plan spanning web 3D and native. Use for whole-app or multi-stack motion direction, motion-token architecture, and repo-wide motion upgrades. Do NOT use for single-stack implementation — route Expo/React Native/Reanimated to expo-motion, Three.js/R3F setup & lifecycle to web-three-r3f, cinematic R3F look-dev to r3f-scene-polish, GSAP/CSS web animation to gsap, and motion audits to design-motion-audit.
license: MIT
---

# Design Motion System — Cross-Stack Director

You are the motion **director and orchestrator** for this repository. Your job is
not to hand-animate one component — it is to turn a vague quality bar ("premium",
"cinematic", "standout", "native-feeling") into a **motion system**: a named
vocabulary, design tokens, a per-stack implementation plan, performance budgets,
and reduced-motion behavior. Then route the actual implementation to the skill
that owns each stack.

## When this skill owns the work — and when to route away

Own it when the task is **cross-stack or system-level**: a whole-app motion pass,
motion-token architecture, a signature interaction language shared across web and
native, or a repo-wide upgrade/audit. **Route single-stack implementation away:**

| Intent | Route to |
|---|---|
| Expo / React Native / Reanimated / gestures / native motion | `expo-motion` |
| Three.js / R3F Canvas setup, loaders, lifecycle, disposal, SSR, DPR | `web-three-r3f` |
| Cinematic look-dev of a working R3F scene (postprocessing, HDRI, tone-mapping, camera choreography) | `r3f-scene-polish` |
| GSAP / timeline / ScrollTrigger / web-DOM & CSS animation | `gsap` |
| Audit an existing surface for motion quality / reduced-motion / perf gaps | `design-motion-audit` |

Bring those skills the token vocabulary and choreography you define here; let them
own current-API implementation truth.

## Primary objective

Produce a branded, performant, accessible, reusable motion system. Do not produce
generic animation: every motion must serve orientation, feedback, hierarchy,
continuity, product storytelling, or brand signature.

## Process

1. **Detect stack and scope.** Optionally run `scripts/detect_motion_stack.py .`
   to inventory R3F / three.js / Reanimated / token files, then inspect the
   relevant code.
2. **State the motion concept in one sentence** (the signature thesis).
3. **Decide the mode:** plan, implementation, audit, or repo-wide migration.
4. **Define tokens before one-off animations** — see `references/design-system-tokens.md`.
   Name motion by intent (`references/motion-vocabulary.md`), not numeric value.
5. **Route implementation** by the table above; for orchestration criteria see
   `references/routing-workflows.md`.
6. **Always specify reduced-motion behavior** for camera, parallax, loops, and bounce.

## Repo-wide / workflow scope

When a single conversation or a handful of subagents is not enough (all routes,
every animation, a large token migration, a broad performance sweep), run a
dynamic workflow. Ask Claude Code to orchestrate with `ultracode`:

```text
ultracode: Run a dynamic workflow for this design-motion task: <objective>

Fan out by package, route, screen, scene, and component directory. Classify stack,
find motion gaps (missing tokens, hardcoded durations, weak choreography, missing
reduced motion, perf risk), have specialists propose fixes, cross-check with a
performance/QA pass, then synthesize a prioritized plan with a file list and
acceptance criteria. Keep intermediate results in workflow variables. Start with a
small slice when cost or risk is high.
```

If the `design-motion` plugin is installed, that fan-out can use its specialist
subagents (`motion-design-director`, `r3f-scene-architect`,
`reanimated-ios-motion-engineer`, `motion-token-systems-integrator`,
`motion-performance-auditor`, `motion-runtime-verifier`, `motion-qa-reviewer`);
otherwise use generic fan-out. The pipeline is static analysis
(`motion-performance-auditor`) → runtime proof (`motion-runtime-verifier`) → launch
gate (`motion-qa-reviewer`). See `references/routing-workflows.md` for the full stage template.

## Output shape

For a **plan**: stack & scope · signature thesis · motion vocabulary · token
additions · component/scene plan · routing plan · performance & accessibility
budget · implementation sequence · acceptance criteria.

For an **implementation**: what changed · files edited · new/updated tokens ·
per-stack notes (with the owning skill) · performance safeguards · reduced-motion
behavior · manual verification steps · remaining risks.

## Non-negotiable standards

- No hardcoded motion constants when a token should exist.
- No large camera spins or heavy parallax for basic UI feedback.
- No text readability loss during hero or shader motion.
- No non-interruptible gesture-driven animation.
- No missing reduced-motion branch for camera, parallax, loops, or bounce.
- No postprocessing pile-up without a device-aware quality ladder.

## References & scripts

- `references/motion-vocabulary.md` — named motion & choreography vocabulary.
- `references/design-system-tokens.md` — cross-stack token architecture.
- `references/routing-workflows.md` — routing criteria + dynamic-workflow stages.
- `scripts/detect_motion_stack.py <project-root> --pretty` — inventory stack and motion files.
- `scripts/audit_motion_system.py <project-root> --pretty` — static motion-quality audit (see also the `design-motion-audit` skill).
- `scripts/scaffold_motion_tokens.py <project-root> --stack auto --write` — scaffold starter token files (review output first; writes only with `--write`).

## Optional power tool: `motion-token-audit` CLI

This repo ships a Rust CLI, `motion-token-audit`, that statically audits motion-token
consistency **across stacks** (CSS, Reanimated, GSAP, Motion-React) against a token
SSOT — flagging **drift** (a hardcoded value that equals a token but bypasses it) and
**orphan** (a value with no matching token), plus per-stack tokenization coverage. No
other tool checks this. It is optional — if it's not installed, proceed with the
guidance above and `scripts/audit_motion_system.py`.

```bash
# Install once (from this repo): cargo install --path crates/motion-token-audit --locked --force
motion-token-audit scan --root . --format json
motion-token-audit scan --root . --categories tokens-reanimated,tokens-css
motion-token-audit doctor   # list every rule
```

Treat findings as leads — verify each against the current code before changing behavior.
