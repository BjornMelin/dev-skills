---
name: design-taste-frontend
description: Senior UI/UX engineering skill—bias-aware layouts, type, and color; Tailwind-first; RSC/client boundaries; Framer-style motion; performance guardrails. Sets baseline dials (variance, motion, density), forbids "AI slop" patterns, and points to a high-end pattern library plus Bento 2.0 motion specs.
---

# High-Agency Frontend Skill

## 1. Active baseline configuration

* `DESIGN_VARIANCE`: **8** (1 = perfect symmetry, 10 = artsy chaos)
* `MOTION_INTENSITY`: **6** (1 = static, 10 = cinematic / heavy physics)
* `VISUAL_DENSITY`: **4** (1 = gallery / airy, 10 = cockpit / dense data)

**How to use:** Default generation assumes **(8, 6, 4)**. Do not ask the user to edit this file for that. If the user **explicitly** changes variance / motion / density in chat, treat those as overrides and feed them into Sections 3–7. Otherwise keep these as globals.

## 2. Default architecture & conventions

Unless the user names a different stack, follow this skeleton:

* **Dependencies [mandatory]:** Before **any** new import (e.g. `framer-motion`, `lucide-react`, `zustand`), read `package.json`. If the package is missing, print the install command (e.g. `npm install package-name`) before code. Never assume a dep exists.
* **Framework:** React or Next.js; prefer Server Components (RSC). Global state **only** in Client Components; put providers in a `"use client"` wrapper. If Sections 4 or 7 (motion / liquid glass) apply, the **interactive** UI must be a **leaf** client component with `'use client'` at the top; RSCs stay for static structure.
* **State:** `useState` / `useReducer` for local UI; global state only to kill deep prop drilling.
* **Styling:** Tailwind v3 or v4 for ~90% of styles. **Lock:** Read `package.json` for version—no v4-only syntax in v3 trees. **v4 PostCSS:** use `@tailwindcss/postcss` or the Vite plugin, not the old `tailwindcss` plugin in `postcss.config.js` alone.
* **No emojis** in code, markup, user-facing text, or `alt`. Use Radix, Phosphor, or small SVGs instead.
* **Layout & spacing:** Use `sm` / `md` / `lg` / `xl` consistently. Center pages with `max-w-[1400px] mx-auto` or `max-w-7xl`. **Hero height:** use `min-h-[100dvh]`, not `h-screen` (iOS / mobile URL bar). **Grids:** prefer `grid` + gaps (e.g. `grid grid-cols-1 md:grid-cols-3 gap-6`); avoid brittle flex + `calc` width tricks like `w-[calc(33%-1rem)]`.
* **Icons:** Import from `@phosphor-icons/react` or `@radix-ui/react-icons` only (match what is installed). One global `strokeWidth` (e.g. `1.5` or `2.0`).

## 3. Design engineering directives (bias correction)

LLMs lean on the same UI clichés. Offset that with these rules.

**Rule 1 — Typography**

* **Display:** Default `text-4xl md:text-6xl tracking-tighter leading-none`. Avoid `Inter` for “premium / creative” looks; use `Geist`, `Outfit`, `Cabinet Grotesk`, or `Satoshi` for character.
* **Dashboard / product UI:** **No serifs** on dense software UIs. Use sans + mono (e.g. `Geist` + `Geist Mono`, or `Satoshi` + `JetBrains Mono`).
* **Body:** `text-base text-gray-600 leading-relaxed max-w-[65ch]`.

**Rule 2 — Color**

* At most **one** accent; keep saturation under **~80%**.
* **Lila / “AI purple” ban:** no purple/blue glow kitsch, no neon purple gradients. Build on neutrals (Zinc / Slate) + one strong accent (e.g. Emerald, electric blue, deep rose).
* **One** gray family per project (do not mix warm and cool grays at random).

**Rule 3 — Layout**

* **Anti-center:** If `DESIGN_VARIANCE > 4`, do **not** default to a centered hero only. Use split, asymmetric, or L/R hero patterns.

**Rule 4 — Materiality & cards**

* If `VISUAL_DENSITY > 7`, avoid generic boxed cards; separate regions with `border-t`, `divide-y`, or space. Use real cards only when z-order / focus needs it. Shadows should pick up the **background** hue, not pure gray slabs.

**Rule 5 — States**

* Ship full cycles: **loading** (skeletons that match layout, not a lone spinner), **empty** (composed and instructive), **error** (inline, specific). On `:active`, use small press motion (`-translate-y-[1px]` or `scale-[0.98]`).

**Rule 6 — Forms**

* Label above field; optional helper; error under field. `gap-2` between stacked field groups.

## 4. Creative proactivity (anti-slop)

Baseline “premium” behaviors:

* **Liquid glass:** Beyond `backdrop-blur`, add `border-white/10` and `shadow-[inset_0_1px_0_rgba(255,255,255,0.1)]` for a believable edge.
* **Magnetic buttons (`MOTION_INTENSITY > 5`):** Do **not** drive pointer tracking with `useState`. Use Framer `useMotionValue` + `useTransform` so motion stays off the main render path.
* **Perpetual micro-motion (`MOTION_INTENSITY > 5`):** Pulse, typewriter, float, shimmer, carousels in normal components. Springs: `type: "spring", stiffness: 100, damping: 20`—**not** raw linear ease on interactions.
* **Layout transitions:** Framer `layout` / `layoutId` for reorder, resize, shared elements.
* **Staggered mounts:** `staggerChildren` or `animation-delay: calc(var(--index) * 100ms)`. For `staggerChildren`, parent `variants` and children must live in the **same** client tree; async data → pass props into one motion parent.

## 5. Performance guardrails

* **Grain / noise:** Only on `fixed`, `pointer-events-none` layers (e.g. `fixed inset-0 z-50 pointer-events-none`), never on scrolling panels.
* **Animation:** `transform` + `opacity` only; not `top` / `left` / `width` / `height` for these effects.
* **Z-index:** Systematic layers (nav, overlay, modal)—not random `z-50` / `z-[9999]` everywhere.

## 6. Technical reference (dial definitions)

### `DESIGN_VARIANCE` (1–10)

* **1–3:** Centered flex, 12-col symmetry, even padding.
* **4–7:** Overlap (`margin-top: -2rem`), mixed image ratios (4:3 next to 16:9), type left / content grid offset.
* **8–10:** Masonry, uneven tracks (`grid-template-columns: 2fr 1fr 1fr`), big empty side bands (`padding-left: 20vw` style).
* **Mobile:** For 4–10, asymmetric `md+` layouts must collapse to single column: `w-full`, `px-4`, `py-8` below `768px` (no broken horizontal scroll).

### `MOTION_INTENSITY` (1–10)

* **1–3:** Only `:hover` / `:active`; no always-on motion.
* **4–7:** CSS transitions, e.g. `transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1)`, staggered delays, `transform` + `opacity`, `will-change: transform` sparingly.
* **8–10:** Scroll-linked / parallax / Framer-level choreography. **No** `window.addEventListener('scroll')` for per-frame work.

### `VISUAL_DENSITY` (1–10)

* **1–3:** Airy, large gaps, “gallery” feel.
* **4–7:** Normal product spacing.
* **8–10:** Tight, line-separated data, minimal boxes; **mono for all numbers** (`font-mono`).

## 7. AI tells (forbidden unless asked)

**Visual & CSS:** No default outer glow; no `#000000` (use off-black / Zinc-950); desaturate accents; no huge gradient text headers; no custom cursor graphics.

**Typography:** No `Inter` (use `Geist`, `Outfit`, `Cabinet Grotesk`, `Satoshi`). No H1s that only scale to “shout.” Serifs only for editorial; **never** on clean dashboards.

**Layout:** Tidy math on spacing. **No** “three equal cards in a row” feature strip—use zig-zag, asymmetric grid, or horizontal scroll.

**Content:** No “John Doe” / “Sarah Chan” / “Jack Su” / “Acme” / “Nexus” / “SmartFlow.” No egg avatars or generic user glyphs—use believable photos or styled placeholders. No fake round numbers only (`99.99%`, `50%`, `1234567`); use messier, real-looking values (`47.2%`, `+1 (312) 847-1928`). No “Elevate / Seamless / Unleash / Next-Gen” filler.

**Assets:** No brittle Unsplash URLs. Use `https://picsum.photos/seed/{random_string}/800/600` or SVG / UI avatars. **shadcn/ui** is OK only after you retune radii, color, shadow to match the project.

**Quality bar:** Code stays clean, intentional, and memorable—not boilerplate with a new font.

## 8. The creative arsenal (inspiration)

Do not default to generic UI. Pull from this list when you need a memorable direction. For **scroll-linked storytelling** or **3D / canvas** work, consider **GSAP (ScrollTrigger / parallax)** or **Three.js / WebGL** instead of only CSS. **Do not** mount GSAP or Three in the same component tree as Framer Motion. Use Framer for UI and bento-style motion; use GSAP/Three only in **isolated** full-page or canvas regions with strict `useEffect` cleanup.

### The standard hero paradigm

* Move past centered type on a stock dark image. Try asymmetric heroes: type aligned L or R; background image is high quality, on-brand, and graded into the page (lighten or darken into the base bg for light / dark mode).

### Navigation & menus

* **Mac OS dock magnification:** Bar at the edge; icons scale on hover.
* **Magnetic button:** CTA subtly tracks the pointer.
* **Gooey menu:** Sub-items peel off the main control like viscous liquid.
* **Dynamic island:** Pill UI that morphs for status / alerts.
* **Contextual radial menu:** Radial options spawn at the click point.
* **Floating speed dial:** FAB that opens a curved secondary action row.
* **Mega menu reveal:** Full-screen drop with staggered, rich content.

### Layout & grids

* **Bento grid:** Asymmetric tile layout (e.g. Control Center style).
* **Masonry layout:** Staggered columns without fixed row heights (e.g. Pinterest).
* **Chroma grid:** Tile or border with slow, subtle animated color.
* **Split screen scroll:** Two halves move in opposite directions on scroll.
* **Curtain reveal:** Hero splits down the middle like a curtain.

### Cards & containers

* **Parallax tilt card:** 3D tilt from pointer position.
* **Spotlight border card:** Edge highlight follows the cursor.
* **Glassmorphism panel:** Frosted glass with inner refraction.
* **Holographic foil card:** Iridescent sheen on hover.
* **Tinder swipe stack:** Physical stack the user can swipe.
* **Morphing modal:** A control expands into a full-screen dialog.

### Scroll animations

* **Sticky scroll stack:** Cards stack and cover as you scroll.
* **Horizontal scroll hijack:** Vertical scroll drives a horizontal track.
* **Locomotive scroll sequence:** Scrub video or 3D with scroll position.
* **Zoom parallax:** Hero bg zooms with scroll.
* **Scroll progress path:** SVG path draws with scroll.
* **Liquid swipe transition:** Page turns over like a viscous sheet.

### Galleries & media

* **Dome gallery:** Panoramic 3D photo ring.
* **Coverflow carousel:** Center card forward, sides recede.
* **Drag-to-pan grid:** Large grid panned with drag.
* **Accordion image slider:** Strips that expand on hover.
* **Hover image trail:** Cursor leaves fading image stamps.
* **Glitch effect image:** Short RGB / channel glitch on hover.

### Typography & text

* **Kinetic marquee:** Scrolling type that reacts to scroll / speed.
* **Text mask reveal:** Type cuts out a video layer.
* **Text scramble effect:** Characters decode (matrix style).
* **Circular text path:** Type on a rotating circle.
* **Gradient stroke animation:** Outline stroke with moving gradient.
* **Kinetic typography grid:** Letters react to pointer position.

### Micro-interactions & effects

* **Particle explosion button:** Success breaks into particles.
* **Liquid pull-to-refresh:** Pull gesture looks like detaching water.
* **Skeleton shimmer:** Moving sheen on placeholders.
* **Directional hover aware button:** Fill enters from the pointer side.
* **Ripple click effect:** Ripples from exact click point.
* **Animated SVG line drawing:** Strokes self-draw.
* **Mesh gradient background:** Soft animated color blobs.
* **Lens blur depth:** Blur back layers to focus a foreground action.

## 9. The “motion-engine” Bento paradigm

For modern SaaS dashboards and feature walls, use **Bento 2.0**: calm Vercel-like surfaces with always-on, tasteful motion.

### A. Core philosophy

* **Look:** Minimal, high-end, functional.
* **Palette:** Page `#f9fafb`, cards `#ffffff`, hairline `border-slate-200/50`.
* **Shape:** Big radius `rounded-[2.5rem]` on major surfaces; “diffusion” shadow e.g. `shadow-[0_20px_40px_-15px_rgba(0,0,0,0.05)]`.
* **Type:** `Geist`, `Satoshi`, or `Cabinet Grotesk` + `tracking-tight` on titles.
* **Layout:** Titles / body for each tile **below** the card (gallery feel). Inner padding `p-8`–`p-10`.

### B. Animation engine (perpetual motion)

* **Spring:** `type: "spring", stiffness: 100, damping: 20` on interactive motion.
* **Layout / shared:** `layout` + `layoutId` for rearranging content.
* **Loops:** Every card should expose an “active” loop (pulse, typewriter, float, carousel) so the dashboard feels alive. **Critical:** isolate perpetual motion in **memoized** client leaf components; never re-render the whole layout from a loop.
* **Lists / presence:** Use `<AnimatePresence>` for list diff; keep motion at 60fps.

### C. Five card archetypes (example set)

For Bento grids such as: Row 1: three columns; Row 2: two columns split ~70/30.

1. **The intelligent list:** Vertical stack with an auto-sort loop; items swap with `layoutId` as if tasks were reprioritized live.
2. **The command input:** Search / AI field with a multi-line typewriter pass, blinking cursor, and a “processing” shimmer gradient state.
3. **The live status:** Scheduling / queue UI with breathing dots; a badge can pop in with an overshoot spring, stay ~3s, then leave.
4. **The wide data stream:** Horizontal infinite strip of mini metric cards; seamless X loop e.g. `x: ["0%", "-100%"]` at a calm speed.
5. **The contextual UI (focus mode):** Document-style block: stagger a highlight on text, then float in a small action toolbar with micro-icons.

## 10. Final pre-flight

* [ ] Global state used to avoid real prop drilling—not sprinkled arbitrarily.
* [ ] High-variance layouts collapse on small screens (`w-full`, `px-4`, `max-w-7xl mx-auto` or equivalent) without horizontal scroll breakage.
* [ ] Full-bleed sections use `min-h-[100dvh]`, not `h-screen`.
* [ ] `useEffect` motion hooks unregister / cleanup.
* [ ] Loading, empty, and error states exist.
* [ ] Spacing / lines replace gratuitous card stacks where density allows.
* [ ] Heavy infinite loops sit in isolated, memoized client components.
