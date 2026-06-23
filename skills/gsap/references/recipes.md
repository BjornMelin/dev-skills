# GSAP Recipes for Next.js (App Router, TypeScript)

Copy-paste, production-minded recipes. All assume a client component and a shared registration module:

```ts
"use client";
// lib/gsap.ts
import { gsap } from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
gsap.registerPlugin(useGSAP, ScrollTrigger); // add SplitText, Flip, Draggable, etc. as needed
export { gsap, useGSAP, ScrollTrigger };
```

Conventions used throughout:
- `"use client"` at the top — GSAP never runs during SSR.
- `useGSAP(() => { ... }, { scope: root })` — selectors are scoped to `root`; cleanup is automatic on unmount (tweens, ScrollTriggers, SplitText, Draggable all reverted).
- Every recipe includes a **reduced-motion** path via `gsap.matchMedia()`. Wrap motion in `mm.add({...}, ...)` and provide a static fallback when `prefers-reduced-motion: reduce`.

## Table of contents

1. [Hero text reveal (SplitText + timeline)](#1-hero-text-reveal)
2. [Pinned, scrubbed section](#2-pinned-scrubbed-section)
3. [Fake horizontal scroll](#3-fake-horizontal-scroll)
4. [Parallax on scroll](#4-parallax-on-scroll)
5. [Scroll progress bar](#5-scroll-progress-bar)
6. [Staggered grid reveal (batch)](#6-staggered-grid-reveal)
7. [Magnetic button / cursor follower (quickTo)](#7-magnetic-button)
8. [Flip layout transition](#8-flip-layout-transition)
9. [SVG logo: DrawSVG + MorphSVG](#9-svg-logo)
10. [App Router page transitions + ScrollTrigger cleanup](#10-app-router-page-transitions)
11. [Smooth scrolling with ScrollSmoother](#11-scrollsmoother)

---

## 1. Hero text reveal

Reveal a headline line-by-line on mount. `SplitText` re-splits on resize via `autoSplit`/`onSplit`; the `mask` option clips each line for a clean wipe.

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";
import { SplitText } from "gsap/SplitText";
gsap.registerPlugin(SplitText);

export function Hero({ text }: { text: string }) {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(() => {
    const mm = gsap.matchMedia();
    mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
      const split = SplitText.create(".hero-title", { type: "lines", mask: "lines", autoSplit: true,
        onSplit: (self) => gsap.from(self.lines, { yPercent: 100, autoAlpha: 0, stagger: 0.12, ease: "power3.out", duration: 0.8 }),
      });
      return () => split.revert();
    });
  }, { scope: root });

  return <div ref={root}><h1 className="hero-title">{text}</h1></div>;
}
```

Reduced-motion: the `mm.add` block only runs when motion is allowed; the heading renders normally otherwise.

---

## 2. Pinned, scrubbed section

Pin a section and scrub a timeline tied to scroll distance. Use `scrub` (not `toggleActions`) and `ease: "none"` inside scrubbed tweens.

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";

export function PinnedSection() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(() => {
    const mm = gsap.matchMedia();
    mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
      const tl = gsap.timeline({
        defaults: { ease: "none" },
        scrollTrigger: { trigger: ".pin", start: "top top", end: "+=1500", pin: true, scrub: 1 },
      });
      tl.from(".pin-card", { autoAlpha: 0, yPercent: 30, stagger: 0.2 })
        .to(".pin-bg", { scale: 1.2 }, 0);
    });
  }, { scope: root });

  return <div ref={root}><section className="pin">{/* .pin-bg, .pin-card[] */}</section></div>;
}
```

Reduced-motion: skip the pin entirely so content scrolls normally (do not pin when reduce is set — pinning changes layout).

---

## 3. Fake horizontal scroll

Translate a wide track horizontally as the user scrolls vertically. The pin length equals the horizontal distance.

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";

export function HorizontalScroll() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(() => {
    const mm = gsap.matchMedia();
    mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
      const track = root.current!.querySelector<HTMLElement>(".track")!;
      const getDistance = () => track.scrollWidth - root.current!.querySelector<HTMLElement>(".viewport")!.offsetWidth;
      gsap.to(track, {
        x: () => -getDistance(),
        ease: "none",
        scrollTrigger: { trigger: ".viewport", pin: true, scrub: 1, end: () => "+=" + getDistance(), invalidateOnRefresh: true },
      });
    });
  }, { scope: root });

  return (
    <div ref={root}>
      <div className="viewport" style={{ overflow: "hidden" }}>
        <div className="track" style={{ display: "flex", width: "max-content" }}>{/* panels */}</div>
      </div>
    </div>
  );
}
```

Function-based `x` + function-based `end` + `invalidateOnRefresh: true` re-measure the distance (from the live `.viewport` `offsetWidth`, not a stale `window.innerWidth`) on every refresh/resize, so the scene stays correct after layout changes. Reduced-motion: render the panels in a normal vertical/grid layout.

---

## 4. Parallax on scroll

Move layers at different rates between `scrollTrigger` start/end with `scrub`.

```tsx
useGSAP(() => {
  const mm = gsap.matchMedia();
  mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
    gsap.utils.toArray<HTMLElement>(".layer").forEach((layer) => {
      const depth = Number(layer.dataset.depth ?? 0.2);
      gsap.to(layer, {
        yPercent: -depth * 100,
        ease: "none",
        scrollTrigger: { trigger: layer, start: "top bottom", end: "bottom top", scrub: true },
      });
    });
  });
}, { scope: root });
```

Reduced-motion: leave layers static.

---

## 5. Scroll progress bar

Drive a fixed bar's `scaleX` from page scroll progress.

```tsx
useGSAP(() => {
  gsap.set(".progress", { transformOrigin: "left center", scaleX: 0 });
  gsap.to(".progress", {
    scaleX: 1,
    ease: "none",
    scrollTrigger: { start: 0, end: "max", scrub: 0.3 },
  });
}, { scope: root });
```

`start: 0`/`end: "max"` track the whole document. A progress indicator is informational, so it's generally safe under reduced motion, but you can swap to a non-animated indicator if desired.

---

## 6. Staggered grid reveal

`ScrollTrigger.batch()` groups elements entering the viewport together so they stagger in batches instead of firing one ScrollTrigger each (cheaper, nicer).

```tsx
useGSAP(() => {
  const mm = gsap.matchMedia();
  mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
    gsap.set(".card", { autoAlpha: 0, y: 40 });
    ScrollTrigger.batch(".card", {
      start: "top 85%",
      onEnter: (els) => gsap.to(els, { autoAlpha: 1, y: 0, stagger: 0.08, overwrite: true }),
      onLeaveBack: (els) => gsap.set(els, { autoAlpha: 0, y: 40, overwrite: true }),
    });
  });
  // reduce: show all cards immediately
  mm.add({ reduce: "(prefers-reduced-motion: reduce)" }, () => gsap.set(".card", { autoAlpha: 1, y: 0 }));
}, { scope: root });
```

---

## 7. Magnetic button

`quickTo` creates one reusable tween for high-frequency pointer updates (far cheaper than a new `gsap.to` per `mousemove`). Wrap handlers in `contextSafe` so they're cleaned up.

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";

export function MagneticButton({ children }: { children: React.ReactNode }) {
  const root = useRef<HTMLButtonElement>(null);
  const reduce = useRef(false);
  const { contextSafe } = useGSAP(() => {
    reduce.current = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  }, { scope: root });

  const onMove = contextSafe((e: React.MouseEvent) => {
    if (reduce.current) return; // reduced-motion: don't follow the pointer
    const el = root.current!; const r = el.getBoundingClientRect();
    const x = e.clientX - (r.left + r.width / 2);
    const y = e.clientY - (r.top + r.height / 2);
    gsap.to(el, { x: x * 0.3, y: y * 0.3, duration: 0.4, ease: "power3.out" });
  });
  const onLeave = contextSafe(() => {
    if (reduce.current) return;
    gsap.to(root.current!, { x: 0, y: 0, duration: 0.5, ease: "elastic.out(1,0.4)" });
  });

  return <button ref={root} onMouseMove={onMove} onMouseLeave={onLeave}>{children}</button>;
}
```

Reduced-motion: the handlers early-return when `prefers-reduced-motion: reduce`, so the button never chases the pointer for users who opt out.

---

## 8. Flip layout transition

`Flip` records positions, you mutate the DOM (reorder, add a class, change grid), then `Flip.from` animates the delta — perfect for layout changes React causes on state updates.

```tsx
"use client";
import { useRef, useState } from "react";
import { gsap, useGSAP } from "@/lib/gsap";
import { Flip } from "gsap/Flip";
gsap.registerPlugin(Flip);

export function FlipGrid({ items }: { items: string[] }) {
  const root = useRef<HTMLUListElement>(null);
  const lastState = useRef<Flip.FlipState | null>(null);
  const [expanded, setExpanded] = useState<string | null>(null);

  useGSAP(() => {
    // Runs AFTER React commits the new DOM; animate from the pre-mutation snapshot.
    if (lastState.current) {
      Flip.from(lastState.current, { duration: 0.5, ease: "power2.inOut", absolute: true, stagger: 0.03 });
    }
  }, { dependencies: [expanded], scope: root });

  return (
    <ul ref={root}>
      {items.map((it) => (
        <li
          key={it}
          className={`item ${expanded === it ? "is-expanded" : ""}`}
          onClick={() => {
            // Capture BEFORE the state change so the DOM is still in its old layout.
            lastState.current = Flip.getState(".item");
            setExpanded(it);
          }}
        >{it}</li>
      ))}
    </ul>
  );
}
```

Always capture `Flip.getState` *before* the layout-driving change — never after. In React, take the snapshot in the event handler immediately before `setState`, then run `Flip.from` in a `useGSAP` keyed on that state so it fires after React commits the new DOM. Capturing inside the post-commit effect would record the already-mutated layout, leaving an empty delta and no animation. Reduced-motion: set `duration: 0`.

---

## 9. SVG logo: DrawSVG + MorphSVG

Draw a stroked path on, then morph one shape into another.

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { MorphSVGPlugin } from "gsap/MorphSVGPlugin";
gsap.registerPlugin(DrawSVGPlugin, MorphSVGPlugin);

export function AnimatedLogo() {
  const root = useRef<SVGSVGElement>(null);
  useGSAP(() => {
    const mm = gsap.matchMedia();
    mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
      const tl = gsap.timeline();
      tl.from("#stroke", { drawSVG: "0%", duration: 1.2, ease: "power2.inOut" })
        .to("#shape", { morphSVG: "#target", duration: 0.8, ease: "power2.inOut" }, "-=0.2");
    });
  }, { scope: root });
  return <svg ref={root} viewBox="0 0 100 100">{/* #stroke path, #shape + hidden #target */}</svg>;
}
```

Reduced-motion: render the final logo state directly (`gsap.set("#stroke", { drawSVG: "100%" })`).

---

## 10. App Router page transitions

ScrollTrigger positions go stale across Next.js route changes, so refresh on navigation and animate route content on mount. A simple, robust approach: a client transition wrapper keyed on `usePathname()`.

```tsx
"use client";
import { useRef } from "react";
import { usePathname } from "next/navigation";
import { gsap, useGSAP, ScrollTrigger } from "@/lib/gsap";

export function PageTransition({ children }: { children: React.ReactNode }) {
  const root = useRef<HTMLDivElement>(null);
  const pathname = usePathname();

  useGSAP(() => {
    // re-run on every route change
    const mm = gsap.matchMedia();
    mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
      gsap.from(root.current, { autoAlpha: 0, y: 16, duration: 0.4, ease: "power2.out" });
    });
    ScrollTrigger.refresh(true); // recalc after new content mounts
  }, { dependencies: [pathname], scope: root });

  return <div ref={root}>{children}</div>;
}
```

Do **not** call `ScrollTrigger.getAll().forEach((t) => t.kill())` here: it destroys triggers owned by other live/persistent components (shared layouts, smooth scrollers, sidebars), not just the unmounted page's. Each component's own `useGSAP` already reverts the ScrollTriggers it created when its page unmounts, so the only thing this wrapper needs is `ScrollTrigger.refresh()` once the new content has mounted. For exit animations, coordinate with the App Router's `template.tsx` (re-renders on navigation) or a view-transition approach. Reduced-motion: skip the enter tween.

---

## 11. Smooth scrolling with ScrollSmoother

`ScrollSmoother` wraps the page and integrates with ScrollTrigger. It needs a fixed wrapper/content structure and must be created client-side. In Next.js App Router, set it up in a top-level client component.

```tsx
"use client";
import { useRef } from "react";
import { gsap, useGSAP } from "@/lib/gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { ScrollSmoother } from "gsap/ScrollSmoother";
gsap.registerPlugin(ScrollTrigger, ScrollSmoother);

export function SmoothScroll({ children }: { children: React.ReactNode }) {
  const wrapper = useRef<HTMLDivElement>(null);
  const content = useRef<HTMLDivElement>(null);
  useGSAP(() => {
    const mm = gsap.matchMedia();
    mm.add({ motion: "(prefers-reduced-motion: no-preference)" }, () => {
      const smoother = ScrollSmoother.create({ wrapper: wrapper.current!, content: content.current!, smooth: 1.2, effects: true });
      return () => smoother.kill();
    });
  }, { scope: wrapper });

  return (
    <div id="smooth-wrapper" ref={wrapper}>
      <div id="smooth-content" ref={content}>{children}</div>
    </div>
  );
}
```

Reduced-motion: don't create the smoother (native scrolling). Note ScrollSmoother only works on the main page scroller, not nested scroll containers — for those, or for Lenis/Locomotive, use `ScrollTrigger.scrollerProxy` (see [ScrollTrigger](./scrolltrigger.md)).

## Related references

- [Core API](./core.md) · [React & Next.js](./react-nextjs.md) · [ScrollTrigger](./scrolltrigger.md) · [Timeline](./timeline.md) · [Plugins](./plugins.md) · [Performance](./performance.md) · [Utilities](./utils.md)
