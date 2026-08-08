---
name: design-review
description: "Adversarial design, frontend, and motion review for web or mobile branches. Use to inspect a dirty UI worktree, prove behavior in a real browser or device, measure motion, performance, and accessibility, and implement one canonical end state."
license: MIT
metadata:
  version: "1.0.0"
  companion-skills:
    - better-interface
    - design-motion-audit
    - improve-ui
    - wcag-audit-patterns
---

# Design Review

Review and, when requested, repair a UI branch to a high product-design bar.
This is an adversarial review and execution workflow, not a screenshot critique
or generic lint pass. Begin with the dirty worktree, trace the actual system,
prove behavior at runtime, measure relevant quality, and converge on one
canonical implementation.

## Modes

- **Review:** inspect, prove, measure, and return corrections without editing.
- **Implementation:** review, implement every confirmed in-scope finding,
  remove superseded paths, rerun proof, and align docs.

State the selected mode and scope before judging. A named route, component, or
motion system narrows the map but does not remove runtime proof.

## Doctrine Sources

Read the repository's own design contract before applying generic taste.
Search in this order:

1. Root and nested agent instructions (`AGENTS.md`, equivalent runtime guides).
2. Design authorities such as `docs/design*`, design-system references,
   product principles, brand rules, accessibility policy, and UI ADRs.
3. Token sources, theme files, global CSS, NativeWind or styling config, font
   declarations, motion tokens, and shared component packages.
4. Component registry/configuration such as `components.json`.
5. The changed components, routes, tests, stories, screenshots, and consumers.

If no design contract exists, say: `No repository design contract found;
generic design-review gates applied.` Do not invent hidden doctrine. Repository
docs own additions to refused vocabulary, brand rules, tokens, and signatures.

## Taste Gates

These are non-negotiable unless a stronger repository contract supersedes them.

### Bold, not enterprise-default

- Give each view one legible focal idea and match density to purpose.
- Reject template-shaped composition, default tokens, timid hierarchy, and
  polish that does not clarify the product. Prefer subtraction.

### Theme parity when theming exists

- If the product supports multiple themes or appearances, each must feel
  intentionally art-directed and equally complete.
- Do not repair one theme by flattening the other. Verify contrast, hierarchy,
  imagery, elevation, focus, status color, and motion in every appearance.
- If the repository has no theming, do not require it as part of this review.

### Motion with purpose

- Movement must communicate state, causality, continuity, feedback, or
  hierarchy. Decorative motion must earn its cost.
- Animate compositor-friendly `transform` and `opacity` for continuous UI
  motion. Treat layout, paint-heavy filters, and large blurred regions as risks
  that require measured justification.
- Target smooth 60fps interaction on supported hardware.
- Respect reduced-motion preferences with a settled, usable end state; never
  hide content merely because animation is disabled.
- Keep server-rendered output stable. Initialization must be SSR/hydration-safe,
  interruptible where interaction demands it, and cleaned up.
- Assign one animation owner per element. Do not let CSS, a JS library, and a
  gesture system fight over the same property.

### No synthetic design filler

- No filler, invented customers, fake metrics/certifications, or decorative
  numbers presented as product truth.
- No gradients whose only purpose is sophistication, or emoji as the primary
  icon and interaction vocabulary.
- No arbitrary glow, glass, noise, particles, or ambient movement used to make
  an unfinished hierarchy look designed.
- No generic AI-product aesthetic in place of a product-specific visual idea.

### Accessibility floor

- WCAG 2.2 AA is the minimum conformance target.
- Require keyboard access, visible focus, semantics, names, usable forms,
  sufficient contrast, zoom/reflow tolerance, and motion control.
- Review validation, errors, loading, empty data, dialogs, menus, and recovery.
  Automation never replaces keyboard and screen-reader-aware inspection.

### Refused vocabulary

The repository's own design contract owns its refused list. In its absence,
flag these common defaults unless the brief explicitly and credibly requires
them:

- mesh or lava-lamp gradients and animated gradient borders;
- generic bento or masonry collages used as a substitute for hierarchy;
- custom cursors without explicit user opt-in and an accessibility rationale;
- scroll hijacking, forced smooth scrolling, or input interception;
- glassmorphism on content surfaces where readability matters;
- the purple-to-blue AI gradient as generic brand shorthand;
- decorative-only chrome, particles, glitch effects, or floating ornaments;
- repetitive card nesting, default shadows, and arbitrary radius ladders.

If repository doctrine explicitly adopts one, judge against that contract and
note the exception rather than silently overriding it.

## Craft Bar

### Typography is architecture

Type scale, measure, weight, tracking, line height, and wrapping establish the
hierarchy before decoration. Remove orphans, clipping, unstable font loading,
false button hierarchy, and dense copy. Check localization and Dynamic Type.

### White space is a tool

Use space to group, pace, and direct attention. Distinguish intentional density
from crowding. Prefer alignment and rhythm over card borders for grouping.

### Easing must be authored

- Do not ship CSS `ease` or `linear` for ordinary UI transitions. Use repository
  motion tokens or an intentional custom curve/spring.
- `linear` remains valid for genuinely constant-rate progress, rotation, or
  other continuous physical behavior, not stateful UI easing.
- Duration, delay, stagger, and easing must fit interaction frequency.

### Signature moments are scarce

A flagship surface may have one memorable interaction that expresses product
character. Do not turn every section into one; clarity precedes spectacle.

### Complete micro-states

Interactive controls need six intentional states where applicable:

1. default;
2. hover;
3. focus-visible;
4. active/pressed;
5. disabled;
6. loading/busy.

Loading needs status feedback; focus-visible must differ from hover; disabled
controls cannot be the only explanation for unavailable actions.

### Geometry and state surfaces

- Nested rounded shapes should use concentric radii rather than identical,
  visually misaligned radii.
- Empty, error, offline, denied, and loading states are product surfaces. Give
  each a clear next action when one exists.
- Maintain layout stability between skeleton, loaded, and error states.

## Discover Named Gates

Never infer validation commands from habit. Inspect:

- package scripts, task-runner, workspace, and toolchain config;
- CI workflows, validation manifests, and package guidance;
- browser, simulator, screenshot, bundle, accessibility, and performance tools.

Record the exact gates that apply to the touched files. Prefer repository
scripts over raw binaries. A gate not run is `UNVERIFIED`, never passed.

### Optional power tools

Use these only when installed or when installation is appropriate and allowed:

```bash
gsap-audit scan --root <path>
motion-token-audit scan --root <path>
expo-motion-audit scan --root <path>
npx motionscore <url> --agent --no-upload
```

The three audit commands are Rust CLIs included in this repository. From a
trusted checkout they can be installed with:

```bash
cargo install --path crates/<name> --locked --force
```

Also discover repository Lighthouse, Core Web Vitals, bundle, screenshot,
visual-regression, and accessibility gates. Retain MotionScore `--no-upload`
unless external upload is approved.

## Review Lanes

Run lanes independently. Use `references/subagent-playbook.md` only when bounded
fan-out is justified.

### 1. Taste and slop

Inspect hierarchy, originality, tokens, composition, theme parity, refused
vocabulary, and decoration covering weak product structure.

### 2. Product truth and simplicity

- State the surface's **One Thing** in one sentence. Count steps to first value;
  prefer three or fewer when natural. For each addition, ask what can be removed.
- Inspect the back of the fence: settings, email, help, loading, empty, error,
  permission, destructive, and recovery states deserve the same care as demos.
- Reject fabricated proof and dark patterns.

### 3. Accessibility and metadata

Check WCAG 2.2 AA, keyboard order, focus management, names, semantics, forms,
contrast, target sizes, reduced motion, title, description, canonical URL,
social metadata, structured data, and robots behavior where applicable.

### 4. Motion

Inspect animation ownership, curves, springs, timing, origins, interruption,
enter/exit pairing, gestures, scroll coupling, reduced-motion end states, SSR,
cleanup, and runtime smoothness. Measure rather than guessing.

### 5. Performance and architecture

Inspect first-load bytes, assets, fonts, rendering, hydration, layout shift,
long tasks, repeated work, Core Web Vitals, and budgets. Separate measured
regression from theoretical micro-optimization.

### 6. Code entropy

Find duplicate tokens, parallel components, dead styles, compatibility props,
fallback variants, aliases, stale assets, and multiple owners of one behavior.
Require one canonical source unless an external boundary is proven.

### 7. Data layer, only when touched

Inspect loading/error/empty ownership, over-fetching, request waterfalls,
subscription breadth, cache invalidation, optimistic rollback, authorization
boundaries, and accidental client trust. Do not broaden into a backend audit if
the UI change does not touch data behavior.

### 8. Native, only for React Native or Expo

Verify on a real device or simulator: safe areas, home indicators, keyboard,
44px targets, Dynamic Type, screen-reader semantics, both appearances when
supported, platform navigation, gestures, haptics, motion, and list smoothness.
Prefer platform-native behavior over fragile reimplementation.

## Evidence Discipline

A confirmed finding requires all three:

1. **Contract:** a cited repository decision or an explicitly named generic
   gate from this skill.
2. **Runtime:** proof that the cited code/value reaches the affected surface,
   ideally with browser/device evidence or a traced ownership path.
3. **Correction:** one required change tied directly to the evidence.

Use `path/to/file:line` citations. Attach screenshots, accessibility snapshots,
console/network output, traces, and measurements. Search hits and stylistic
differences are candidates, not findings.

Mark unsupported claims `UNVERIFIED`. If browser, native, accessibility,
motion, or performance evidence is unavailable, mark that lane `Degraded` and
the final verdict `Inconclusive`. A compile pass is not visual proof; browser
proof is not native proof; automated accessibility is not manual coverage.

## Severity

- **HIGH:** blocks a core task, violates WCAG AA, breaks supported appearance or
  viewport behavior, causes destructive confusion, ships fabricated product
  truth, or creates a measured severe performance/motion regression.
- **MEDIUM:** materially weakens hierarchy, comprehension, interaction quality,
  state completeness, maintainability, or a non-core performance path.
- **LOW:** localized craft or consistency issue with a clear correction and low
  user impact. Do not inflate preferences into findings.

Use `Block` for hard failures even if a numeric or aggregate score looks good.

## Companion Skill Routing

Load only companions present in this repository and relevant to the lane:

- `motion`, `gsap`, or `expo-motion` for implementation-specific motion.
- `design-motion-audit` for a read-only cross-stack motion punch list.
- `r3f-scene-polish` for existing Three.js/R3F cinematic look development.
- `improve-ui` for documented design-system drift and Contract/Runtime/
  Correction evidence.
- `wcag-audit-patterns` and `web-interface-guidelines` for web conformance.
- `better-ui`, `better-colors`, `better-typography`, `better-layout`,
  `better-accessibility`, and `better-writing` for focused craft ownership.
- `better-interface` for a cross-discipline interface review/build workflow.
- `autoreview` or `multi-model-review` for independent code-review support.

## Workflow

1. **Scope the dirty worktree.** Read repository instructions and design
   doctrine, inspect status and diff, identify unrelated changes, and do not
   overwrite work outside scope.
2. **Build a system map.** Trace changed routes/components to tokens, shared
   primitives, assets, motion owners, data producers, tests, docs, and native
   surfaces. Identify canonical and superseded paths.
3. **Discover gates.** Read scripts and CI; list applicable commands before
   running them. Never invent a gate.
4. **Run review lanes.** Cover every applicable lane and record `Complete`,
   `Degraded`, `Not in scope`, or `Not reviewed`.
5. **Prove and measure.** For web, use a real browser in every supported theme
   at desktop and a mobile viewport, with console/network inspection. For
   native, use a real device or simulator. Exercise reduced motion. Run
   applicable accessibility, performance, motion, and bundle measurements.
6. **Synthesize findings.** Require Contract + Runtime + Correction, cite
   files and evidence, deduplicate by root cause, and rank by severity.
7. **Implement every confirmed finding when requested.** Make the smallest
   complete correction, collapse duplicate ownership, and delete superseded
   compatibility/fallback paths unless an external boundary is named.
8. **Re-verify.** Repeat the affected browser/device flows and measurements,
   then run the repository gates implied by touched files.
9. **Align docs.** Update design, component, testing, or contributor contracts
   only when behavior or ownership changed.

## Verdict Ladder

- **Approve:** every applicable lane is complete, no confirmed issue remains,
  runtime evidence is adequate, and required gates pass.
- **Needs changes:** correctable findings remain; provide an ordered required
  action list.
- **Block:** a core UX, accessibility, truth, security, destructive-action,
  console/runtime, or severe measured performance failure prevents approval.
- **Inconclusive:** required evidence or lane coverage is degraded. Never use
  `Approve` with an unverified required lane.

## Review Output Format

Return:

1. **Scope and mode** — dirty diff, surfaces, themes/viewports, and exclusions.
2. **Doctrine and gates** — authorities read, generic fallback if needed, gates
   run, gates skipped, and `UNVERIFIED` items.
3. **Coverage** — status for every applicable review lane.
4. **Findings** — ordered HIGH, MEDIUM, LOW; each includes Contract, Runtime,
   Correction, `file:line`, and before evidence.
5. **Implemented fixes** — changed files, canonical path retained, and legacy
   paths removed, or `Report-only`.
6. **After proof** — browser/device captures, console, motion, accessibility,
   performance, bundle, and named gate results.
7. **Verdict** — one value from the ladder with rationale.
8. **Considered but rejected** — one to three plausible candidates rejected as
   unsupported, out of scope, or contrary to doctrine.
9. **Residual gaps** — explicit `UNVERIFIED` risks and the command or evidence
   needed to close each.

Never claim a visual, native, motion, accessibility, or performance pass from a
different kind of evidence. Evidence quality is part of the design quality.
