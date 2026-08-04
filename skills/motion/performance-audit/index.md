# MotionScore performance audit

Use the procedure in this skill to find & fix animation performance issues either by analysing code or, if requested, runtime profiling.

1. Audit: Classify every animation by its render-pipeline cost. Identify fixes and upgrade paths.
2. Report: Present findings to the user. Request which to fix (or all), unless prompt already asks to fix ("/motion fix animation performance in Modal.tsx" etc).
3. Fix: Implement requested fixes and summarise.

### Scoring overview

MotionScore ranks animations based on their render pipeline cost using a tier grade between S-F:

| Tier  | Thread              | Cost                                                          | Example                                                                 |
| ----- | ------------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------- |
| **S** | Compositor          | Near-zero — no main-thread work                               | `transform`, `opacity` via CSS/WAAPI                                    |
| **A** | Main → Compositor   | Low — compositor values set from JS                           | `element.style.transform` via rAF/GSAP                                  |
| **B** | Main (setup) → S/A  | One-time DOM read then S/A animation                          | FLIP technique, Motion `layout` prop                                    |
| **C** | Main (paint)        | Medium — repaint affected layers each frame                   | `background-color`, `color`, `border-radius`, CSS variable animations   |
| **D** | Main (layout+paint) | High — layout recalc + paint + composite each frame           | `width`, `height`, `margin`, `top`/`left`, `scrollTop` polling          |
| **F** | Main (thrash)       | Catastrophic — forced synchronous layout per read/write cycle | Interleaved DOM reads/writes, CSS variable inheritance bombs on `:root` |

[resources/tier-reference.md] provides a detailed breakdown of exactly which kinds of values and patterns fall into which tier. Always read this file first.

## Step 1: Audit

### 1. Scope

The prompt determines audit scope. For example, "/motion audit all animation in src/components" - code audit all files in the directory.

Alternatively, the user might request to audit specific things: "audit use of will-change".

If requesting a "runtime" audit, or providing a specific URL, we should perform a runtime audit using `npx motionscore`. It could in this instance you also perform code analysis in subagents to triangulate.

### 1a. Runtime audit (when a URL is available)

If the prompt contains a URL (eg `http://localhost:3000`, `myhomepage.com`, `localhost:5173`, a public origin) or the word "runtime", additionally the runtime audit:

```
npx motionscore <url> --agent
```

`--agent` runs a local Puppeteer audit and returns a report keyed by selector with library hints and grep targets. Use it when the user asks to audit "the homepage", "this page" (if you know the dev server URL), or any specific URL.

How to use the output:

-   The report's **Findings** section lists every sub-S animation with its selector, animated properties, detected library, and an explicit `Source hint` line telling you what to grep for. Treat that list as authoritative for what's running on that URL — skip ahead to step 5 (Identify fixes) for those animations.
-   Anti-patterns (mount/frame thrashing, blur, persistent will-change, etc.) come with write→read selector pairs already grouped — use them to locate the offending code directly.
-   This report checks specific things that code analysis has a harder time with, like max concurrent animations and GPU pressure, plus accessibility checks (`prefers-reduced-motion`, long animations, flashing content). They can work together.
-   If the runtime audit fails (no dev server, navigation timeout, browser launch error), fall back to pure static discovery and inform the user.

### 2. Discover

Search the codebase for animation patterns. Cast a wide net:

**CSS/SCSS/Sass:**

-   `transition:` and `transition-property:`
-   `animation:` and `@keyframes`
-   `will-change:`
-   `scroll-timeline`, `view-timeline`, `animation-timeline`

**JavaScript/TypeScript:**

-   `element.style.*` assignments in loops or rAF callbacks
-   `element.animate()` (WAAPI)
-   `.classList.add/remove/toggle` where the class has transitions
-   `requestAnimationFrame` callbacks that modify style
-   `scrollTop`, `scrollLeft`, `getBoundingClientRect()` in animation loops

**Library imports — read `tier-reference.md` and `references/property-tiers.json` for patterns:**

-   Motion: `animate`, `motion.`, `useAnimate`, `useSpring`, `layout`, `layoutId`, `whileInView`, `scroll()`
-   GSAP: `gsap.to`, `gsap.from`, `gsap.timeline`, `ScrollTrigger`
-   react-spring: `useSpring`, `animated.`
-   anime.js: `anime()`
-   Lottie: `lottie.loadAnimation`, `<Lottie`
-   View Transitions: `document.startViewTransition`, `::view-transition`

### 3. Classify

For every animation found:

1. Identify ALL values being animated
2. Look up each value's tier in `references/property-tiers.json`
3. **Worst-tier wins** — if an animation touches `opacity` (S) and `width` (D), the animation is D-tier.
4. Factor in the animation method:
    - CSS transitions/animations or WAAPI with compositor props → S-tier
    - JS-driven (rAF, GSAP) with compositor props → A-tier
    - Motion `layout`/`layoutId` → B-tier
    - CSS variable animations → always C-tier minimum, even for compositor values
5. Read `tier-reference.md` for edge cases and caveats.

### 4. Detect anti-patterns

Scan for these specific problems (see `tier-reference.md` for detection details):

| Anti-Pattern                                                 | Severity | Tier |
| ------------------------------------------------------------ | -------- | ---- |
| Layout thrashing (interleaved reads/writes in loops)         | Critical | F    |
| CSS variable on `:root`/`html`/`body` being animated         | Critical | F    |
| `scrollTop`/`scrollLeft` read in animation loop              | High     | D    |
| CSS variable used in compositor value (`opacity: var(--x)`)  | High     | C    |
| Excessive `will-change` (>3 properties or on >10 elements)   | Medium   | —    |
| `filter: blur()` with value >10px on large elements          | Medium   | S\*  |
| Long-running off-screen animations (no IntersectionObserver) | Medium   | —    |
| Missing `prefers-reduced-motion` handling                    | Medium   | —    |
| Multiple DOM-reading libraries without shared scheduling     | Medium   | —    |
| View Transition without interruption handling                | Low      | —    |

### 5. Identify fixes

For every animation below S-tier, check if an upgrade is possible:

-   **D → S**: Replace `width`/`height` animation with `transform: scale()` if possible
-   **D → S**: Replace `top`/`left` animation with `transform: translate()` if possible
-   **D → B**: Replace `width`/`top` etc with Motion `layout` prop
-   **C → S** (low priority, large elements only): `background-color` transitions on large surfaces (cards, panels, sections, hero areas — not buttons or badges) can be replaced with two absolutely-positioned `::before`/`::after` pseudo-elements crossfaded via `opacity`. Only flag this if profiling shows the paint is a bottleneck.
-   **C → C**: Ensure CSS variables are registered with `@property` and `inherits: false` where applicable. Doesn't prevent paint but will prevent style recalculation wildfire.
-   **A → S**: Replace Motion `x`/`y`/`scale` etc with `transform` to run animation via WAAPI. Only works when values aren't being animated independently, for instance:

```
<motion.div initial={{ x: -100 }} animate={{ x: 0 }} whileHover={{ scale: 2 }} />
```

Or with value-specific transitions.

-   **F → A**: Batch reads/writes Motion's `frame.read()`/`frame.update()`

Only suggest upgrades that are practical. Not every D-tier animation can be upgraded — `font-size` animations genuinely need to trigger layout (although arguably even these could be emulated with `scale`, it depends on the precise application). Say so.

### 6. Check accessibility

-   [ ] `prefers-reduced-motion` media query present and meaningful (not just `animation: none`)
-   [ ] No animations longer than 5 seconds without user control
-   [ ] No rapidly flashing content (>3 flashes per second)
-   [ ] Decorative animations are pausable or respect reduced-motion
-   [ ] Essential motion (e.g. page transitions) has a reduced alternative, not just removal (say crossfade instead of shared element)

## Step 2: Report

Structure every audit report exactly like this.

```
## Motion Performance Audit

**Scope:** [file/component/directory/project]
**Files scanned:** [count]
**Animations found:** [count]

### Scorecard

Render as a code block bar chart. Bar width is proportional to percentage using `█` for filled and `░` for empty, 25 characters total width. Right-align counts and percentages, round percentage to nearest 4% increment. Ensure bars are all the same length.

The overall rank should be a perceptual average rank of the scores. For instance two A's and a C should be a B. 50% S and an evenly spread distribution for other scores should be A. Etc.

```

Rank

:::'███::::
::'██ ██:::
:'██:. ██::
'██:::. ██:
.█████████:
.██.... ██:
.██:::: ██:
..:::::..::

Breakdown
S ██████████████████████░░░░░░░░░░░░░░░░░░░ 14 · 45%
A █████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 6 · 19%
B ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 4 · 13%
C ███████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 5 · 16%
D █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 1 · 3%
F █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 1 · 3%

```

Each ranking has its own ASCII graphic.

S

```

:'██████::
'██... ██:
.██:::..::
. ██████::
:..... ██:
'██::: ██:
. ██████::
:......:::

```

A

```

:::'███::::
::'██ ██:::
:'██:. ██::
'██:::. ██:
.█████████:
.██.... ██:
.██:::: ██:
..:::::..::

```

B

```

'████████::
.██.... ██:
.██:::: ██:
.████████::
.██.... ██:
.██:::: ██:
.████████::
........:::

```

C

```

:'██████::
'██... ██:
.██:::..::
.██:::::::
.██:::::::
.██::: ██:
. ██████::
:......:::

```

D

```

'████████::
.██.... ██:
.██:::: ██:
.██:::: ██:
.██:::: ██:
.██:::: ██:
.████████::
........:::

```

F

```

'████████:
.██.....::
.██:::::::
.██████:::
.██...::::
.██:::::::
.████████:
........::

```

### Findings

Only list animations that have an actionable upgrade or are B-tier or below. S and A-tier animations with no upgrade path should be omitted from individual findings. Instead, summarise them in a single line, e.g.:

> 14 animations are already S-tier — nice work.

#### [file:line] — Tier [X]
**What:** [property being animated, e.g. "`width` transition on `.card`"]
**Why Tier [X]:** [one sentence — which property triggers which pipeline stage]
**Impact:** [quantified if possible — "triggers layout on ~200 elements per frame"]
**Upgrade:** [specific suggestion or "No practical upgrade — layout is required here"]

(...repeat for each animation with an actionable finding...)

### Anti-Patterns

#### [severity] — [pattern name]
**Location:** [file:line]
**Problem:** [what's happening]
**Fix:** [specific code change]

### Accessibility

Only list issues found. Omit checks that pass — no news is good news.

- ✗ [issue description and location]
- ✗ [issue description and location]

### Top 3 Recommendations

1. **[Highest impact fix]** — [one sentence with expected tier improvement]
2. **[Second highest]** — [one sentence]
3. **[Third highest]** — [one sentence]
```

## Step 3: Fix

If prompted, implement the fixes and other suggestions found in the report.

Try to keep fixes authored in the original library used. For instance, if Motion was originally used then tend towards keeping the fix in Motion.

That said, if there's a GSAP animation that uses independent transforms for no reason, and a good fix would be to use Motion's hardware accelerated `transform`, then suggest this to the user. Likewise, if we're triggering paint using CSS variables and we could more performantly do this with Motion then also suggest this.

## Voice rules

-   Be **specific**: name the exact property, file, and line
-   Be **decisive**: assign a tier, don't hedge with "might be"
-   Be **quantified**: "triggers layout on ~50 elements" not "could be slow"
-   Be **actionable**: every finding has a concrete upgrade path or explicit "no upgrade available"
-   **No false positives**: if a `transform` animation is already S-tier via CSS, don't flag it. Only report problems or upgrades.
-   Don't pad the report — if there are only 2 animations, report 2. If everything is S-tier, say so and move on.

## Library-specific notes

**Motion (framer-motion / motion)**

-   Uses WAAPI under the hood → compositor properties are genuine S-tier
-   `animate()` with `transform`/`opacity`/`clipPath`/`filter` → S-tier
-   `layout` / `layoutId` props → B-tier (FLIP technique, one-time layout read)
-   `whileInView` → S-tier (depending on values being animated) + auto-deactivates off-screen (good practice)
-   `scroll()` with compositor targets → S-tier (ScrollTimeline)
-   `frame.read()` / `frame.update()` → prevents layout thrashing across libraries
-   Deferred keyframe resolution batches DOM reads (2.5x faster than unbatched)

**GSAP**

-   Uses `requestAnimationFrame`, not WAAPI → A-tier ceiling for compositor properties
-   `ScrollTrigger` reading `scrollTop` → D-tier unless using native scroll timeline
-   Can animate thousands of elements efficiently due to optimized internals

**CSS Native**

-   `transition` / `@keyframes` with compositor props → S-tier
-   `scroll-timeline` / `view-timeline` → S-tier (hardware accelerated)
-   `@property` registration prevents CSS variable inheritance cost

**View Transitions API**

-   `::view-transition-*` pseudo-elements animate `transform` + `opacity` → S-tier
-   But measuring old/new states triggers layout → overall B-to-D depending on what changes
-   Cannot be interrupted mid-flight without Motion's `animateView` wrapper
