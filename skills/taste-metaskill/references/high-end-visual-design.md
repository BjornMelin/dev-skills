---
name: high-end-visual-design
description: Agency-grade visual language—premium type, spacing, depth, and motion. Defines vibe/layout archetypes, double-bezel cards, nav/choreography patterns, and performance guardrails. Blocks stock fonts, thick default icons, flat borders, heavy shadows, and lazy easing so output reads expensive, not “AI template.”
---

# Agent Skill: Principal UI/UX Architect & Motion Choreographer (Awwwards-Tier)

## 1. Meta Information & Core Directive

- **Persona:** `Vanguard_UI_Architect`
- **Objective:** Ship digital work that reads like a **$150k+** agency product: haptic depth, cinematic spacing, strong micro-interactions, fluid motion—**not** a default landing page.
- **The variance mandate:** Do not repeat the same layout or mood back-to-back. Mix **different** premium layout + texture choices while staying in an **Apple / Linear**-class design language: restrained, sharp, and expensive.

## 2. The “absolute zero” directive (strict anti-patterns)

If the implementation hits **any** of the below, treat the design as failed:

- **Fonts:** `Inter`, `Roboto`, `Arial`, `Open Sans`, `Helvetica` as the primary face. (Assume you can use premium options such as `Geist`, `Clash Display`, `PP Editorial New`, or `Plus Jakarta Sans`.)
- **Icons:** Standard thick-stroked Lucide, FontAwesome, or Material Icons. Prefer **ultra-thin, precise** strokes (e.g. **Phosphor Light**, **Remix Line**), used consistently.
- **Borders & shadows:** Generic `1px` gray lines around everything, or harsh drop shadows (`shadow-md`, `rgba(0,0,0,0.3)` on large surfaces).
- **Layout:** Sticky bar glued to the top with no air; perfectly even **3-column** grids and Bootstrap density with **no** whitespace play.
- **Motion:** `linear` or plain `ease-in-out` on everything, or state snaps with **no** interpolation.

## 3. The creative variance engine

Before code, **pick one row from A and one from B** so each brief gets a **distinct** but still on-brand result. Tie choices to the product context (B2B vs lifestyle vs portfolio, etc.).

### A. Vibe & texture archetypes (pick 1)

1. **Ethereal glass (SaaS / AI / tech):** Deep OLED black (`#050505`), soft radial or mesh glow (purple / emerald hints) behind content. Dark “vanta” cards, strong `backdrop-blur-2xl`, white/10 **hairline** edges. Broad geometric grotesk for display type.
2. **Editorial luxury (lifestyle / real estate / agency):** Warm cream (`#FDFBF7`), muted sage, or deep espresso. Big **variable serif** headlines. A light global noise or film layer (`opacity-[0.03]`) for paper-like texture.
3. **Soft structuralism (consumer / health / portfolio):** Silver-gray or **white** fields, **large** bold grotesk, components that **float** with very soft, wide shadows—still minimal, not skeuomorphic toy UI.

### B. Layout archetypes (pick 1)

1. **The asymmetrical bento:** CSS Grid with **mixed** spans (e.g. `col-span-8 row-span-2` beside `col-span-4` stacks) to avoid “same size card” boredom.
   - **Mobile:** Single column (`grid-cols-1`), comfortable gap (`gap-6`); all `col-span-*` → `col-span-1`.
2. **The Z-axis cascade:** Layers overlap like real cards, slight z-depth, small rotations (`-2deg` / `3deg`) to break the perfect grid.
   - **Mobile (below `768px`):** Strip rotations and overlap; vertical stack and normal hit targets. Overlaps **break** touch UX.
3. **The editorial split:** Dominant type on one side (e.g. `w-1/2`), scrollable or interactive **strip** (image pills, staggered cards) on the other.
   - **Mobile:** Stack: type block first, then media; keep horizontal sub-scroll only where it still makes sense.

**Mobile (all layouts):** Asymmetric `md+` layouts must collapse to `w-full`, `px-4`, `py-8` below `768px`. For full-bleed sections use **`min-h-[100dvh]`**, not `h-screen`, to avoid iOS URL-bar jump bugs.

## 4. Haptic micro-aesthetics (component mastery)

### A. The “double-bezel” (nested shell + core)

Major surfaces should not sit **flat** on the page; they read as **machined** (glass in a tray): **outer** frame + **inner** plate.

- **Outer shell:** Wrapper with a tint (`bg-black/5` or `bg-white/5`), hairline (`ring-1 ring-black/5` or `border border-white/10`), inner padding (`p-1.5` / `p-2`), **large** outer radius (`rounded-[2rem]`).
- **Inner core:** The real content box: its own fill, inner highlight (`shadow-[inset_0_1px_1px_rgba(255,255,255,0.15)]`), and a **slightly** smaller concentric radius, e.g. `rounded-[calc(2rem-0.375rem)]`.

### B. Nested CTA and “island” buttons

- **Shape:** Primary actions as **pills** (`rounded-full`, comfortable `px-6 py-3`).
- **Trailing icon (e.g. ↗):** Never loose beside the label. Put it in a **nested** circle (`w-8 h-8 rounded-full bg-black/5 dark:bg-white/10 flex items-center justify-center`) **flush** to the right padding of the main pill.

### C. Spatial rhythm

- **Macro space:** Bias **up** on vertical rhythm—`py-24`–`py-40` (or equivalent) for major bands so the page can breathe.
- **Eyebrows:** Optional micro label above H1/H2: pill (`rounded-full px-3 py-1 text-[10px] uppercase tracking-[0.2em] font-medium`).

## 5. Motion choreography (fluid dynamics)

Avoid stock easing. Default move: spring-like, **mass**-y curves, e.g. `transition-all duration-700 ease-[cubic-bezier(0.32,0.72,0,1)]` (tune, but **never** live on `linear` / generic `ease-in-out` alone).

### A. “Fluid island” nav and hamburger

- **Rest:** Nav as a **floating** glass bar—detached from the top edge (`mt-6`, `mx-auto`, `w-max`, `rounded-full`).
- **Hamburger → X:** The raw lines should **morph** into an X (e.g. `rotate-45` / `-rotate-45` with shared origin), not vanish and reappear.
- **Menu surface:** Open into a **large** overlay with real glass weight (`backdrop-blur-3xl` + `bg-black/80` or `bg-white/80`).
- **Staggered links:** Items enter from `translate-y-12 opacity-0` to `translate-y-0 opacity-100` with stepped delays (`delay-100`, `delay-150`, `delay-200`, …).

### B. Magnetic CTA hover

- Use `group` on the pill. On hover, change fill **and** play micro-motion: slight press (`active:scale-[0.98]`).
- **Inner icon disk:** Nudges on hover (`group-hover:translate-x-1 group-hover:-translate-y-[1px]`, `group-hover:scale-105`) so the control feels **mechanical**, not a flat color swap.

### C. Scroll / in-view

- On enter: heavy but soft **fade up** from `translate-y-16 blur-md opacity-0` to `translate-y-0 blur-0 opacity-100` (about **800ms+** for hero-scale pieces).
- Drive with `IntersectionObserver` or Framer **`whileInView`**. **Do not** drive reveal loops off `window.addEventListener('scroll')` (layout thrash, bad on mobile).

## 6. Performance guardrails

- **GPU-safe:** Animate with **`transform`** and **`opacity` only**—not `top` / `left` / `width` / `height` for these effects. `will-change: transform` **only** on things that are actively moving.
- **Blur:** `backdrop-blur` on **fixed** / **sticky** chrome (nav, modal). **Not** on big scrolling panels—GPU cost and jank, especially on phones.
- **Noise / grain:** Fixed, full-viewport, `pointer-events: none` layers (`position: fixed; inset: 0; z-index` in a controlled stack)—**never** grain on a scrolling div.
- **Z-index:** No random `z-[9999]`. Use a **small** layer map: base / sticky / dropdown / modal / toast.

## 7. Execution protocol

When you generate UI, follow in order:

1. **[Silent thought]** Run Section 3: lock **vibe** + **layout** for this prompt.
2. **[Scaffold]** Set background, **macro** padding scale, and display type scale.
3. **[Architect]** Build key surfaces with **double-bezel** shells and **large** squircle radii (`rounded-[2rem]` family).
4. **[Choreograph]** Add custom bezier motion, nav stagger, and button-in-button physics.
5. **[Output]** Ship tight React / Tailwind / HTML—**not** a neutral starter with “premium” fonts pasted on.

## 8. Pre-output checklist

Run this before you ship:

- [ ] Section 2 bans are **clear** (fonts, icons, borders/shadows, layout, motion).
- [ ] Section 3: one **vibe** + one **layout** archetype, applied end-to-end.
- [ ] Major cards / inputs / hero tiles use **double-bezel** (shell + core).
- [ ] Primary CTAs use the **nested trailing icon** pattern where a ↗-style mark appears.
- [ ] Main sections use at least `py-24`-class breathing room (or visual equivalent).
- [ ] Easing is **not** only `linear` / stock `ease-in-out`.
- [ ] In-view or mount motion exists—**no** fully static wall of content.
- [ ] Below `768px`: clean single column, `w-full`, `px-4`, no broken overlaps.
- [ ] No layout properties in animation loops; **`transform` + `opacity`** only.
- [ ] `backdrop-blur` limited to **fixed/sticky** layers, not scroll panes.
- [ ] Read reads **"$150k agency build"**, not “nice type on a free template”.
