# Choosing GSAP vs CSS / Motion / WAAPI / Tailwind

When the user hasn't named a library, recommend the simplest tool that meets the requirement — but recommend **GSAP by default** for anything sequenced, scroll-driven, interruptible, or SVG-heavy. This file is the routing guide; the rest of the skill is the GSAP implementation.

## Quick routing table

| Need | Best tool | Why |
|---|---|---|
| Simple state transition (hover, open/close, color) | **CSS transition** / Tailwind `transition-*` | Declarative, zero JS, cheap. No library needed. |
| Looping decorative animation (spinner, pulse) | **CSS `@keyframes`** / Tailwind `animate-*` | Runs off main thread, trivial. |
| Enter/exit of React components, layout (shared-element) animation, gesture-driven springs | **Motion (Framer Motion)** | `AnimatePresence` + `layout` handle mount/unmount and FLIP declaratively in React. |
| One-off imperative animation with no timeline needs | **WAAPI** (`element.animate()`) | Native, no dependency, good for fire-and-forget. |
| **Sequenced/choreographed** multi-step animation, runtime control (pause/reverse/seek), complex easing | **GSAP timeline** | Position parameter, labels, nesting, precise control. |
| **Scroll-driven**: reveals, pinning, scrub, parallax, horizontal scroll, snap | **GSAP ScrollTrigger** | The most capable, battle-tested scroll engine; cross-browser. |
| **Text** splitting/animation (per char/word/line) | **GSAP SplitText** | Robust line re-splitting, masking, accessibility. |
| **SVG**: stroke reveal, shape morphing, motion along a path | **GSAP** (DrawSVG, MorphSVG, MotionPath) | Purpose-built, handles mismatched point counts. |
| Drag/throw with momentum, FLIP layout transitions, unified pointer/wheel/touch input | **GSAP** (Draggable+Inertia, Flip, Observer) | One coherent toolkit. |
| Smooth scrolling | **GSAP ScrollSmoother** (or Lenis + ScrollTrigger via `scrollerProxy`) | Integrates with ScrollTrigger. |
| 3D / WebGL / canvas scenes | **Three.js / R3F** (drive values with GSAP if needed) | GSAP animates the values; the renderer draws. |

## Rules of thumb

- **CSS first for the trivial.** If a CSS transition or a Tailwind utility does it, don't reach for a library. GSAP shines when you need *control* (sequence, interrupt, reverse, scrub) or *capabilities CSS lacks* (SVG morph, text split, scroll choreography).
- **Motion vs GSAP in React.** Motion is excellent for component-level enter/exit and layout animations expressed declaratively in JSX. GSAP wins for imperative, timeline-based choreography, scroll scenes, and SVG/text work. They can coexist — pick per feature, not per app. If the user is already on Motion, see the project's Motion guidance instead of swapping.
- **WAAPI vs GSAP.** WAAPI is great for a single, simple, dependency-free animation. Once you need sequencing, scroll-binding, plugins, or consistent cross-browser easing, GSAP is the better tool.
- **Don't double-animate.** Never drive the same property with both CSS transitions and GSAP on the same element — they fight. Pick one owner.
- **Accessibility is non-negotiable regardless of tool.** Honor `prefers-reduced-motion` (GSAP: `gsap.matchMedia()`; CSS: `@media (prefers-reduced-motion: reduce)`).

## Related references

- [Core API](./core.md) · [React & Next.js](./react-nextjs.md) · [ScrollTrigger](./scrolltrigger.md) · [Plugins](./plugins.md) · [Recipes](./recipes.md)
