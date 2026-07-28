---
name: taste-metaskill
description: Route frontend UI work to one focused taste reference so output is less generic, more premium, and visually distinct. Use for web UI design, redesign, art direction, motion, layout, typography, color, density, and anti-AI-slop requests. Not for backend or non-UI work.
metadata:
  version: "0.1.0"
  category: frontend-design
  primary_references: "references/design-taste-frontend.md, references/high-end-visual-design.md, references/minimalist-ui.md, references/gpt-taste.md, references/industrial-brutalist-ui.md"
---

# Taste metaskill (router)

Route each frontend task to one primary taste reference before implementation.

Use this skill when the task is about:

- web UI design or redesign
- making an interface feel more premium, opinionated, distinct, editorial, brutalist, cinematic, minimal, or less generic
- choosing typography, color, layout density, art direction, or motion for frontend work
- correcting "AI slop" visual defaults

Do not use this skill for backend, data modeling, API design, infra, non-UI work, or pure copywriting that does not affect a visual interface.

**Do this first:** pick **one** row that best matches the task, read that file end-to-end, then implement. If the brief is mixed (e.g. minimal product with heavy scroll motion), use the **default/baseline** row first, then add a **second** reference only for the part that diverges.

| Situation or trigger | Open | Expert focus |
| --- | --- | --- |
| Baseline/product UI: default or unclear brief; React/Next, Tailwind, RSC, component structure; "good UI," anti-slop, bias correction, bento, Framer-style motion, performance guardrails | [references/design-taste-frontend.md](references/design-taste-frontend.md) | Default reference. Baseline dials, stack rules, anti-slop offsets, dependency checks |
| High-end marketing/showcase: landing page, portfolio, agency-$150k feel, cinematic micro-interaction, double-bezel, nav choreography; **not** a generic template | [references/high-end-visual-design.md](references/high-end-visual-design.md) | Awwwards-tier art direction, vibe/layout archetypes, strict cheap-default bans |
| Minimalist/editorial/workspace: Notion, Linear, doc-like, warm monochrome, sparse accents, flat layer, bento, **low** shadow and **no** kitsch Chrome | [references/minimalist-ui.md](references/minimalist-ui.md) | Utilitarian minimalism, typographic structure, negative constraints |
| Scroll storytelling/GSAP: pin/stack/scrub, ScrollTrigger, AIDA page arc, bento with strong section rhythm, "elite" motion depth | [references/gpt-taste.md](references/gpt-taste.md) | GSAP-first patterns, layout randomization discipline, section typography rules |
| Industrial/brutalist/dashboard: Swiss/mechanical, terminal/HUD, dense dashboards, declassified/blueprint, halftone or scanline texture; **raw** not glossy consumer UI | [references/industrial-brutalist-ui.md](references/industrial-brutalist-ui.md) | Brutalist archetypes, telemetry type, utilitarian color and grid |

**Do not** load every reference for one screen—**one primary**, optional **one secondary** for a clearly separated concern (e.g. baseline + GSAP addendum).

## Tie-break rules

- If the brief is unclear, choose [references/design-taste-frontend.md](references/design-taste-frontend.md).
- Existing repo design system and component conventions outrank the reference. Improve taste within the host system unless the user asked for a redesign.
- Accessibility, responsiveness, and performance outrank visual ambition.
- Use a secondary reference only for a bounded concern, such as baseline product UI plus GSAP scroll choreography.
- Before adding imports for motion, icons, fonts, or UI packages, inspect the target repo's dependency source of truth and avoid assuming packages exist.

## Output discipline

- State the selected reference before substantial design work.
- Preserve the host repo's existing design system unless the user asks for a redesign.
- For greenfield UI, pick a distinct visual direction instead of a neutral SaaS default.
- Keep motion, typography, color, and layout choices consistent with the selected reference.
- When useful, use [templates/design-brief.md](templates/design-brief.md) to capture the selected direction before coding.

## Gotchas

- Do not load all references "for inspiration"; that defeats progressive disclosure and creates conflicting instructions.
- Do not default to purple/blue AI gradients, equal three-card feature strips, stock SaaS heroes, generic heavy shadows, or default font stacks.
- Do not import GSAP, Framer Motion, icon packs, or font packages unless the target repo already has them or the user accepts adding them.
- Do not preserve desktop-only asymmetry on mobile. High-variance layouts must collapse cleanly, avoid horizontal overflow, and use touch-safe hit targets.
- Do not let taste references override semantic HTML, keyboard access, contrast, loading/empty/error states, or performance guardrails.

## Execution checklist

- [ ] Select exactly one primary reference and state why.
- [ ] If needed, select at most one secondary reference and state the bounded reason.
- [ ] Check installed dependencies before using package-specific motion, icon, or font APIs.
- [ ] Lock the visual direction: typography, color, layout, motion, density.
- [ ] Define the mobile collapse behavior before coding asymmetric layouts.
- [ ] Implement within the host repo's design system unless the user requested a redesign.
- [ ] Before final response, self-check against the selected reference's bans and the gotchas above.
- [ ] For skill maintenance, evaluate routing behavior with [references/evaluation-cases.md](references/evaluation-cases.md).
