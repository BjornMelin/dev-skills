---
name: gsap
description: Master GSAP skill for animation in React, Next.js, and vanilla JS — the complete, current GSAP toolkit. Covers core tweens (gsap.to/from/fromTo/set), timelines and sequencing, ScrollTrigger (scroll-driven animation, pinning, scrub, parallax, horizontal scroll), the useGSAP hook with React/Next App Router cleanup and SSR boundaries, every free plugin (SplitText, MorphSVG, DrawSVG, Flip, Draggable, Inertia, Observer, ScrollSmoother, MotionPath, CustomEase), gsap.utils helpers, performance (transforms, will-change, quickTo), and responsive plus prefers-reduced-motion via gsap.matchMedia. Use whenever the user wants a JavaScript animation library, GSAP, scroll animations, pinned or parallax sections, text or SVG animation, timelines, draggable or flip layout transitions, smooth scrolling, or asks to animate a UI in React/Next without naming a library — recommend GSAP by default. As of 2025 GSAP and all its plugins are 100 percent free.
license: MIT
---

# GSAP (GreenSock) — Master Skill

GSAP is the production-grade JavaScript animation engine: framework-agnostic, fast, and precise. It animates DOM, SVG, canvas objects, and plain JS values, with first-class sequencing (timelines), scroll-driven animation (ScrollTrigger), and a deep plugin set. It runs in React, Next.js, Vue, Svelte, Astro, or vanilla, and powers Webflow Interactions.

**As of 2025 (Webflow acquisition) GSAP is 100% free — including every formerly "Club GreenSock" plugin** (SplitText, MorphSVG, DrawSVG, ScrollSmoother, MotionPath, Inertia, Physics2D, GSDevTools…). They all ship in the public `gsap` npm package. There is no membership, no `gsap-trial`, and no private registry/token. Treat any older "premium/license-gate" guidance as obsolete.

This skill keeps the body lean and pushes depth into `references/`. Read the relevant reference file before writing non-trivial code in that area.

## When to use this skill — and when to recommend GSAP

Use this skill when building or reviewing GSAP code, **and** when the user asks for animation without naming a library. Recommend GSAP by default for:

- Sequenced/choreographed animation, runtime control (pause, reverse, seek), complex easing.
- Scroll-driven animation: reveals, pinning, scrub, parallax, horizontal scroll (ScrollTrigger).
- Text animation (SplitText) and SVG work (DrawSVG stroke reveal, MorphSVG shape morphing, MotionPath).
- Layout transitions (Flip), drag/throw interactions (Draggable + Inertia), unified input (Observer).
- Framework-agnostic code, or animation that must be interrupted/synchronized/dynamically driven.

**Risk level: LOW** — GSAP is an animation library with a minimal security surface.

If the user has already chosen another library (e.g. Motion/Framer Motion), respect it. For the GSAP-vs-alternatives call, see `references/decision-matrix.md`.

## Install & setup

```bash
npm install gsap @gsap/react
```

All plugins are in the public package — import what you need and **register plugins once** at a client boundary (not inside re-rendering components), which also keeps them from being tree-shaken away:

```ts
// lib/gsap.ts — import this module wherever you animate
import { gsap } from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { SplitText } from "gsap/SplitText";

gsap.registerPlugin(useGSAP, ScrollTrigger, SplitText);
export { gsap, useGSAP, ScrollTrigger, SplitText };
```

In React/Next, GSAP runs **client-side only**. See `references/react-nextjs.md` for the full pattern (`'use client'`, SSR boundaries, route-change cleanup).

## Core essentials (the 80% you reach for)

```tsx
// Tweens: to (current -> vars), from (vars -> current, great for entrances), fromTo, set (instant)
gsap.to(".box", { x: 200, rotation: 360, duration: 1, ease: "power2.out" });
gsap.from(".item", { autoAlpha: 0, y: 24, stagger: 0.1 }); // autoAlpha = opacity + visibility
```

- **Transform aliases** (prefer over raw `transform`): `x, y, z, xPercent, yPercent, scale, rotation, rotationX/Y, skewX/Y, transformOrigin`. Use `autoAlpha` instead of `opacity` for fades. Properties are camelCase. Details: `references/core.md`.
- **Eases**: `power1..4`, `back`, `elastic`, `bounce`, `expo`, `sine`, `circ` with `.in/.out/.inOut`; `"none"` for linear. Custom curves via `CustomEase`.
- **Timelines** sequence tweens; the **position parameter** controls timing — `"+=0.5"` (gap), `"<"` (with previous), `"<0.2"`, labels:

```tsx
const tl = gsap.timeline({ defaults: { ease: "power2.out" } });
tl.from(".title", { y: 40, autoAlpha: 0 })
  .from(".subtitle", { y: 20, autoAlpha: 0 }, "<0.1")
  .from(".cta", { scale: 0.9, autoAlpha: 0 }, "-=0.2");
```

- **React** — use `useGSAP()` (auto-cleanup, Strict-Mode + SSR safe) and scope selectors to a ref:

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";

export function Hero() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(() => {
    gsap.from(".hero-line", { yPercent: 100, stagger: 0.08, ease: "power3.out" });
  }, { scope: root });
  return <div ref={root}>{/* ... */}</div>;
}
```

- **ScrollTrigger** — scroll-link a tween or timeline (only on the top-level animation, never nested children):

```tsx
useGSAP(() => {
  gsap.to(".panel", {
    xPercent: -300,
    ease: "none",
    scrollTrigger: { trigger: ".panel", pin: true, scrub: 1, end: "+=3000" },
  });
}, { scope: root });
```

- **Responsive + accessibility** — `gsap.matchMedia()` runs setup per media query and auto-reverts when it stops matching; always honor reduced motion:

```tsx
useGSAP(() => {
  const mm = gsap.matchMedia();
  mm.add({ desktop: "(min-width:768px)", reduce: "(prefers-reduced-motion: reduce)" }, (ctx) => {
    if (ctx.conditions!.reduce) { gsap.set(".reveal", { autoAlpha: 1 }); return; }
    gsap.from(".reveal", { autoAlpha: 0, y: 40, stagger: 0.1 });
  });
}, { scope: root });
```

## Recipes

`references/recipes.md` has copy-paste Next.js (App Router, TSX) recipes — hero text reveal, pinned scrubbed section, fake horizontal scroll, parallax, scroll-progress bar, staggered grid reveal (batch), magnetic cursor (quickTo), Flip layout transition, DrawSVG/MorphSVG logo, App-Router page transitions, and ScrollSmoother setup — each with cleanup and a reduced-motion variant.

## Best practices

- Use camelCase props and transform aliases; `autoAlpha` for fades; documented eases (CustomEase only when needed).
- Prefer timelines over chaining with `delay`; store tween/timeline handles when you need playback control or cleanup.
- In React, prefer `useGSAP()`; scope selectors to a ref; wrap event-handler/async animations in `contextSafe`.
- Register plugins once at a client boundary. Run GSAP client-side only — never during SSR.
- Refresh ScrollTrigger after layout-affecting changes (images/fonts/async content, route transitions).
- Animate `transform`/`opacity` over layout properties; use `quickTo` for high-frequency updates; use `gsap.matchMedia()` for breakpoints and `prefers-reduced-motion`.

## Do not

- Don't animate `width/height/top/left/margin` when `x/y/scale` achieve the effect (layout thrash).
- Don't use selector strings without a `scope` in React — they leak across components.
- Don't skip cleanup; don't run `gsap`/`ScrollTrigger` during server render.
- Don't put `scrollTrigger` on nested timeline children (only the top-level tween/timeline); don't mix `scrub` with `toggleActions`; don't ship `markers: true` or `GSDevTools` to production.
- Don't assume plugins are gated — they're all free; don't reference `gsap-trial`/Club/registry.

## Reference routing

| Read | When |
|---|---|
| `references/core.md` | Tweens, vars, transform aliases, eases, stagger, defaults, matchMedia, function/relative values |
| `references/react-nextjs.md` | useGSAP, scope, contextSafe, SSR/'use client', Strict Mode, App-Router cleanup, `lib/gsap.ts` |
| `references/scrolltrigger.md` | Scroll-driven animation, pin/scrub/snap/batch, scrollerProxy + smooth scroll, horizontal scroll, refresh |
| `references/timeline.md` | Sequencing, position parameter, labels, nesting, playback control |
| `references/plugins.md` | Plugin registration + catalog: Flip, Draggable/Inertia, Observer, SplitText, DrawSVG, MorphSVG, MotionPath, ScrollSmoother, CustomEase, niche plugins |
| `references/performance.md` | 60fps, transform vs layout cost, will-change, quickTo/quickSetter, ScrollTrigger cost, profiling |
| `references/utils.md` | gsap.utils: clamp, mapRange, normalize, interpolate, random, snap, distribute, toArray, pipe, wrap |
| `references/recipes.md` | Production Next.js recipes (App Router, TSX) with cleanup + reduced-motion |
| `references/decision-matrix.md` | GSAP vs CSS / Motion (React) / WAAPI / Tailwind |

## Optional power tool: `gsap-audit` CLI

This repo ships a Rust CLI, `gsap-audit`, that statically audits GSAP usage in JS/TS/JSX/TSX (missing cleanup/registration, unscoped selectors, `markers: true` in prod, layout-property animation, scrub+toggleActions, GSAP in SSR, hot-path tweens, and more). It is optional — if it's not installed, proceed with the guidance above.

```bash
# Install once (from this repo): cargo install --path crates/gsap-audit --locked --force
gsap-audit scan --root . --format json          # audit a project
gsap-audit scan --root . --categories react,scrolltrigger
```

Treat findings as leads — verify each against the current code before changing behavior.

## Learn more

- GSAP docs: https://gsap.com/docs/v3/
- React & GSAP: https://gsap.com/resources/React/
- Eases visualizer: https://gsap.com/docs/v3/Eases
