# Motion React Field Guide

Use this before loading the large scraped docs when the task needs current API
shape or implementation heuristics.

## Package Snapshot

- Current pinned source/registry snapshot: `motion@12.40.0`.
- Official installation docs say Motion is compatible with React `18.2` and
  higher. The package peer range is `react` / `react-dom`
  `^18.0.0 || ^19.0.0`; report this distinction for version-sensitive work.
- Package exports include `motion/react`, `motion/react-client`,
  `motion/react-m`, and `motion/react-mini`. Prefer source-ledger/provenance
  for exact pins before installing or migrating imports.

## Import Surfaces

- `motion/react`: default React entrypoint for `motion`, `AnimatePresence`,
  `LayoutGroup`, `MotionConfig`, hooks, `LazyMotion`, `domAnimation`, and
  `domMax`.
- `motion/react-client`: Next.js App Router option for animated DOM components
  rendered from a server component without converting the importing module to a
  client component. Do not use it for Motion hooks, refs, scroll, gestures,
  layout measurement, presence hooks, or browser reads.
- `motion/react-m`: slim `m` components for `LazyMotion`. Pair with
  `LazyMotion` and feature bundles; do not mix regular `<motion.*>` inside the
  same strict subtree.
- `motion/react-mini`: mini React export exists in the package, but prefer
  official docs and target repo precedent before using it.

Motion `12.40.0` exports all four surfaces above and declares React/React DOM
peer support for `^18.0.0 || ^19.0.0`.

## SSR And Client Boundaries

- Next App Router files that import `motion/react` and use Motion hooks,
  gestures, refs, layout measurement, scroll, or browser reads need a client
  boundary.
- Prefer a small animated leaf client component over converting a whole route
  or data-loading surface to a client component.
- Use `motion/react-client` only for simple declarative animated DOM wrappers
  that can stay server-rendered. It is not a hook entrypoint.

## Presence

- `AnimatePresence` tracks direct children as they leave the React tree. Direct
  children need stable, unique keys; avoid array indexes in reordering lists.
- Keep `AnimatePresence` mounted and put the conditional inside it.
- `mode="sync"` is the default. It has no sequencing opinion, so resolve
  layout overlap yourself or use another mode.
- `mode="wait"` sequences exit before enter and supports one child at a time.
- `mode="popLayout"` removes exiting children from layout immediately. Check
  that the parent is positioned, transformed ancestors are intentional, and
  custom immediate children forward the provided ref to the DOM node.
- Use `propagate` for nested `AnimatePresence` when child exits should fire
  because a parent presence boundary is removed.

## Layout

- Use `layout` for size/position changes caused by React renders and
  `layoutId` for shared-element transitions.
- Change layout through normal style/class/render state; do not duplicate the
  same size/position change through `animate`.
- Use `LayoutGroup` when separately rendering siblings need coordinated layout
  measurement or when layout and presence interact.
- Add `layoutScroll` to scrollable containers and `layoutRoot` to fixed
  containers so measurements account for scroll offsets.
- Reserve media dimensions and test text wrapping. Layout animations use
  transforms for performance, but unstable content can still look stretched or
  jumpy.

## Scroll And Motion Values

- `useScroll` returns motion values. Bind them directly to style or compose
  with `useTransform`/`useSpring` instead of setting React state every frame.
- Prefer GPU-friendly outputs (`transform`, `opacity`, `clipPath`, `filter`)
  for scroll-linked effects.
- Use `container`, `target`, and `offset` explicitly for element scroll
  tracking. Enable `trackContentSize` only when dynamic content size changes
  must update scroll ranges.

## Reduced Motion

- Use `useReducedMotion()` for local branches and `MotionConfig` for subtree
  policy.
- `MotionConfig reducedMotion="user"` respects the device setting. When reduced
  motion is active, transform and layout animations are disabled while opacity
  and color can still animate.
- Replace long travel, parallax, background autoplay, and looping spatial
  effects with opacity, color, instant state changes, or user-triggered motion.

## Bundle Size

- Default to `motion/react` unless bundle analysis or route budget justifies
  extra complexity.
- For `LazyMotion`, use `domAnimation` for animations, variants, exit
  animations, and hover/tap/focus gestures. Use `domMax` for those plus
  pan/drag gestures and layout animations.
- Set `LazyMotion strict` when adopting `m` components across a subtree so
  accidental `<motion.*>` imports fail loudly.

## Review Heuristics

- Confirm every animated surface has a clear driver: state, presence, layout,
  gesture, scroll, or imperative sequence.
- Confirm the import surface matches the framework boundary and bundle goal.
- Toggle state quickly, resize, navigate away during exits, test keyboard
  focus, and emulate reduced motion.
- Treat animation as behavior. Run the repo's focused typecheck/lint/tests and
  browser-check the exact interaction.
