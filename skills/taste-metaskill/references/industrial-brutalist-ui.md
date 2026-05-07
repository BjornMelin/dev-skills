---
name: industrial-brutalist-ui
description: Raw mechanical UI blending Swiss print discipline with military / terminal telemetry. Rigid modular grids, extreme type scale contrast, utilitarian color, analog-style degradation (halftone, scanlines, dithering). For data-heavy dashboards, portfolios, or editorial sites that should feel like declassified blueprints.
---

# SKILL: Industrial Brutalism & Tactical Telemetry UI

## 1. Skill Meta

**Name:** Industrial Brutalism & Tactical Telemetry Interface Engineering
**Description:** Advanced work at the intersection of mid-century Swiss typography, industrial manuals, and retro aerospace / military terminal UI. The stack is rigid modular grids, extreme type scale contrast, utilitarian palettes, and programmatic analog effects (halftone, scanlines, bitmap dithering). The aim is high data density, mechanical precision, and a refusal of default consumer UI gloss.

## 2. Visual Archetypes

The system pulls from two **compatible** lineages. **Pick one for the whole product** and do not mix light “print” and dark “terminal” in the same surface.

### 2.1 Swiss Industrial Print

From 1960s corporate systems and machine documentation.

* **Characteristics:** High-contrast light skin (newsprint, off-white). Heavy neo-grotesk sans. Grids with visible structure and rules. Asymmetric white space, sometimes dominated by one huge numeral or letter. Primary red for alerts and emphasis.

### 2.2 Tactical Telemetry & CRT Terminal

From secure databases, mainframes, and aircraft-style HUDs.

* **Characteristics:** Dark mode only, dense tabular data, monospace as the main reading face, technical framing (ASCII brackets, crosshairs), and cues of hardware limits (phosphor glow, scan lines, low bit depth).

## 3. Typographic Architecture

Type carries structure and most of the “decoration.” Photography and illustration are supporting. The system leans on large jumps in size, weight, and spacing, not on soft UI chrome.

### 3.1 Macro-Typography (Structural Headers)

* **Classification:** Neo-Grotesque / Heavy Sans-Serif.
* **Optimal Web Fonts:** Neue Haas Grotesk (Black), Inter (Extra Bold/Black), Archivo Black, Roboto Flex (Heavy), Monument Extended.
* **Implementation Parameters:**
  * **Scale:** Very large, fluid type (e.g. `clamp(4rem, 10vw, 15rem)`).
  * **Tracking (Letter-spacing):** Tight, often negative (`-0.03em` to `-0.06em`) so lines read as solid blocks.
  * **Leading (Line-height):** Tight band (`0.85` to `0.95`).
  * **Casing:** Uppercase for primary structural lines.

### 3.2 Micro-Typography (Data & Telemetry)

* **Classification:** Monospace / Technical Sans.
* **Optimal Web Fonts:** JetBrains Mono, IBM Plex Mono, Space Mono, VT323, Courier Prime.
* **Implementation Parameters:**
  * **Scale:** Small, fixed range (`10px` to `14px` / `0.7rem` to `0.875rem`).
  * **Tracking:** Relaxed (`0.05em` to `0.1em`) to echo typewriter or early terminals.
  * **Leading:** Standard to slightly tight (`1.2` to `1.4`).
  * **Casing:** Uppercase for metadata, nav, unit IDs, and coordinates.

### 3.3 Textural Contrast (Artistic Disruption)

* **Classification:** High-Contrast Serif.
* **Optimal Web Fonts:** Playfair Display, EB Garamond, Times New Roman.
* **Implementation Parameters:** Use rarely. When used, run through strong degradation (halftone, 1-bit dither) so clean vector curves feel broken next to the sans system.

## 4. Color System

No gradients, soft long shadows, or glass blur as a default. Palette reads as **substrate and ink** or **glowing phosphor on glass**.

**CRITICAL: One substrate per project. Do not switch between light and dark in one interface.**

### If Swiss Industrial Print (Light)

* **Background:** `#F4F4F0` or `#EAE8E3` (matte, paper-like).
* **Foreground:** `#050505` to `#111111` (carbon ink).
* **Accent:** `#E61919` or `#FF2A2A` (aviation / hazard). Treat as the **only** accent: strike lines, key dividers, or critical data callouts.

### If Tactical Telemetry (Dark)

* **Background:** `#0A0A0A` or `#121212` (off-black CRT, not pure void; avoid flat `#000000` for large fields).
* **Foreground:** `#EAEAEA` (white phosphor, default body).
* **Accent:** Same hazard reds as the light system, same role.
* **Terminal Green (`#4AF626`):** Optional. If used, one clear role only (a single status LED, one channel of data, etc.); never as general body text unless you are intentionally aping a green-screen only layout.

## 5. Layout and Spatial Engineering

Layout should read as measured and gridded, not as generic “web padding in a box.”

* **The Blueprint Grid:** CSS Grid: elements sit on defined tracks, not casual float layouts.
* **Visible Compartmentalization:** `1px` or `2px` solid rules between regions; full-width `<hr>` to separate operating blocks where it helps.
* **Bimodal Density:** Tight teletype-style clusters of metadata next to long calm areas around the macro headlines.
* **Geometry:** Reject `border-radius`. Corners stay square for a machined feel.

## 6. UI Components and Symbology

Default components give way to industrial graphic language.

* **Syntax Decoration:** ASCII and similar marks frame content.
  * *Framing:* `[ DELIVERY SYSTEMS ]`, `< RE-IND >`
  * *Directional:* `>>>`, `///`, `\\\\`
* **Industrial Markers:** `®`, `©`, and `™` at large scale, read as form and rhythm, not footnote law text.
* **Technical Assets:** Crosshairs (`+`), vertical rhythm lines, thick hazard stripes, and placeholder strings like `REV 2.6`, `UNIT / D-01` to suggest live machines and revision control.

## 7. Textural and Post-Processing Effects

To avoid a sterile “vector app” read, add analog **fake** in CSS and SVG.

* **Halftone and 1-Bit Dithering:** Turn photos or big serif type into dot or threshold patterns, via assets or `mix-blend-mode: multiply` with SVG noise / dot layers.
* **CRT Scanlines:** For terminal looks, a `repeating-linear-gradient` on the view (e.g. `repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.1) 2px, rgba(0,0,0,0.1) 4px)`) to suggest horizontal sweeps.
* **Mechanical Noise:** One low-opacity film grain (often SVG) at the root so light and dark modes share a physical surface.

## 8. Web Engineering Directives

1. **Grid Determinism:** `display: grid; gap: 1px;` with different parent/child backgrounds can draw hairline structure without a forest of `border` declarations.
2. **Semantic Rigidity:** Prefer real semantics where they match the content: `<data>`, `<samp>`, `<kbd>`, `<output>`, `<dl>` for definitions and telemetry.
3. **Typography Clamping:** Use `clamp()` for macro type so hero scale stays bold from phone to wide desktop without breaking the grid.
