---
name: minimalist-ui
description: Premium utilitarian minimalism: warm monochrome, strong typographic contrast, bento-style grids, flat components, desaturated pastel accents. Rejects default SaaS chrome—no Inter-by-default, no heavy shadows, no loud hero fills or gradient kitsch.
---

# Protocol: Premium Utilitarian Minimalism UI Architect

## 1. Protocol Overview

**Name:** Premium Utilitarian Minimalism & Editorial UI

**Description:** Frontend directive for refined, document-like surfaces in the spirit of high-end workspace and editorial products. The stack is warm high-contrast monochrome, deliberate type scale, large structural whitespace, bento layouts, a flat component layer, and rare muted pastels. It is explicitly **not** generic SaaS template styling.

## 2. Absolute Negative Constraints (Banned Elements)

Avoid these common defaults:

- **Fonts:** No `Inter`, `Roboto`, or `Open Sans` as the primary system.
- **Icon kits:** No thin default sets such as `Lucide`, `Feather`, or stock `Heroicons` for “generic dashboard” iconography. (See §6 for allowed directions.)
- **Shadows:** No Tailwind-heavy drops (`shadow-md`, `shadow-lg`, `shadow-xl`) unless you heavily soften them—almost no shadow, or very diffuse and low opacity (below about `0.05`).
- **Color blocks:** No loud primary-tinted **large** fields (no bright blue / green / red hero panels).
- **Effects:** No gradients for decoration, no neon, no thick glassmorphism (a light navbar blur is the exception).
- **Shape:** No `rounded-full` for **large** containers, main cards, or primary CTAs.
- **Emojis:** None in code, markup, copy, headings, or `alt` text. Use real icons or simple SVGs.
- **Copy:** No placeholder names like "John Doe" / "Acme Corp" or raw `Lorem Ipsum` where real context could exist. No buzzwords like "Elevate", "Seamless", "Unleash", "Next-Gen", "Game-changer", "Delve"—use plain, specific language.

## 3. Typographic Architecture

The UI should read **editorial**: contrast comes from type pairing and scale, not from chrome.

- **Primary sans (body, UI, buttons):** Clean geometric or system-near faces with character, e.g. `font-family: 'SF Pro Display', 'Geist Sans', 'Helvetica Neue', 'Switzer', sans-serif`.
- **Editorial serif (hero, pull quotes):** e.g. `font-family: 'Lyon Text', 'Newsreader', 'Playfair Display', 'Instrument Serif', serif` with tight tracking (`letter-spacing: -0.02em` to `-0.04em`) and line-height about `1.1` on big display lines.
- **Monospace (code, meta, keys):** `font-family: 'Geist Mono', 'SF Mono', 'JetBrains Mono', monospace`.
- **Text color:** Body is **not** pure `#000000`. Use charcoal (`#111111` or `#2F3437`) with `line-height: 1.6` for long reading. Secondary text: `#787774`.

## 4. Color Palette (Warm Monochrome + Spot Pastels)

Color is **sparse**: use for meaning, tags, and quiet accents—not for big decorative fields.

- **Canvas / background:** `#FFFFFF` or warm off-white (`#F7F6F3` / `#FBFBFA`).
- **Cards / secondary surfaces:** `#FFFFFF` or `#F9F9F8`.
- **Borders / dividers:** `#EAEAEA` or `rgba(0,0,0,0.06)`.
- **Pastel accents (tags, code chips, small icon wells only):**
  - Pale red: `#FDEBEC` (text on chip: `#9F2F2D`)
  - Pale blue: `#E1F3FE` (text: `#1F6C9F`)
  - Pale green: `#EDF3EC` (text: `#346538`)
  - Pale yellow: `#FBF3DB` (text: `#956400`)

## 5. Component Specifications

- **Bento / feature grids**
  - Asymmetrical `display: grid` layouts, not a uniform card dump.
  - Card chrome: `border: 1px solid #EAEAEA` everywhere it applies.
  - Radius cap: `8px` or `12px` on larger surfaces.
  - Internal padding: generous (`24px`–`40px` range).
- **Primary buttons**
  - Fill `#111111`, label `#FFFFFF`.
  - Radius `4px`–`6px`, no default box-shadow. Hover: shift toward `#333333` and/or `transform: scale(0.98)`.
- **Tags / status chips**
  - Pill radius (`border-radius: 9999px`), `text-xs`, uppercase, `letter-spacing: 0.05em`, on the **pastel** swatches only.
- **Accordions (FAQ)**
  - No card boxing around the whole list—separate rows with `border-bottom: 1px solid #EAEAEA`.
  - Toggle with clear `+` / `-` (or equivalent), not noisy chevrons-only clutter.
- **Keystroke hints**
  - Real `<kbd>` chips: `border: 1px solid #EAEAEA`, `border-radius: 4px`, `background: #F7F6F3`, monospace face.
- **Faux app window (optional)**
  - A slim white top bar and three small gray circles to echo macOS window chrome when you mock software UIs.

## 6. Iconography & Imagery Directives

- **Icons:** Prefer **Phosphor** (Bold or Fill) or **Radix Icons** for a slightly technical, weighty line. One stroke weight across a screen.
- **Illustration:** Monochrome line art on white, one offset shape filled with a **muted pastel** from the palette.
- **Photography:** Desaturated, warm, subtle grain overlay (e.g. `opacity: 0.04`) so images sit in the monochrome world. If you need a stand-in, `https://picsum.photos/seed/{context}/1200/800` is acceptable when assets are missing.
- **Section backgrounds:** Avoid empty flat voids. Use very low-opacity photography, a soft warm radial (`radial-gradient` around `opacity: 0.03`), or light geometric line work so sections breathe without going loud.

## 7. Subtle Motion & Micro-Animations

Motion is **felt**, not a show. Prefer restraint.

- **Scroll-in:** `translateY(12px)` from `opacity: 0` to settled over `~600ms` with `cubic-bezier(0.16, 1, 0.3, 1)`. Drive visibility with `IntersectionObserver`, not `window` scroll listeners.
- **Card hover:** Shadow moves from `0 0 0` to about `0 2px 8px rgba(0,0,0,0.04)` in `200ms`. Buttons: `scale(0.98)` on active where it fits.
- **Staggers:** `animation-delay: calc(var(--index) * 80ms)` on list/grid children so the page does not pop in as one block.
- **Ambient background (optional):** One very slow moving radial blob, `20s+` period, `opacity: 0.02`–`0.04`, on a `position: fixed; pointer-events: none` layer only—not on scroll containers.
- **Performance:** Animate `transform` and `opacity` only. Avoid animating `top` / `left` / `width` / `height` for these effects. `will-change: transform` on elements that are actively moving, not globally.

## 8. Execution Protocol

When you implement HTML / React / Tailwind / Vue (or design at the same level of detail):

1. Set **macro whitespace** first: large section padding (e.g. `py-24` / `py-32` in Tailwind where appropriate).
2. Cap main text column at roughly `max-w-4xl` or `max-w-5xl` for readability.
3. Wire in the type scale and warm monochrome **before** piling on components.
4. Keep **every** card, rail, and divider on the `1px solid #EAEAEA` language unless a single exception is intentional and documented in the spec.
5. Add scroll / enter motion to main blocks, not to every label.
6. Give sections **depth** (image, soft radial, or texture) so nothing reads as a white void—without breaking minimalism.
7. Ship code that already matches the system so it does not need a “make it not ugly” pass after the fact.
