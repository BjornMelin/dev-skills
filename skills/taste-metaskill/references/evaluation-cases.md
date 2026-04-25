# Taste Metaskill Evaluation Cases

Use these cases to test whether the router selects the right reference and avoids predictable failure modes. Each case should be run against the active `SKILL.md` without loading all references up front.

## Case 1: Baseline product UI

Prompt:

```text
Make this dashboard page feel less like generic AI SaaS. Keep the current React and Tailwind structure, but improve typography, spacing, empty states, and motion without adding unnecessary dependencies.
```

Expected primary reference:

- `references/design-taste-frontend.md`

Pass criteria:

- Chooses the baseline/product UI route.
- Checks existing dependencies before imports.
- Preserves the existing design system unless explicitly redesigning.
- Mentions loading, empty, and error states where relevant.
- Avoids purple gradients, generic three-card strips, and missing dependency assumptions.

Failure modes:

- Loads every taste reference.
- Applies high-end marketing art direction to a utilitarian dashboard.
- Adds Framer, GSAP, or icon packages without checking the repo.

## Case 2: High-end marketing/showcase

Prompt:

```text
Redesign the landing page so it feels like a premium agency-built product launch, with cinematic spacing, stronger micro-interactions, and a distinctive hero.
```

Expected primary reference:

- `references/high-end-visual-design.md`

Pass criteria:

- Chooses the high-end marketing/showcase route.
- Locks one vibe archetype and one layout archetype.
- Uses premium typography, large macro spacing, and deliberate motion.
- Defines mobile behavior for asymmetric or overlapping layouts.

Failure modes:

- Produces neutral SaaS cards with a new font only.
- Uses generic sticky top nav with no air.
- Keeps perfectly even card grids without rhythm.

## Case 3: Minimalist/editorial/workspace

Prompt:

```text
Make this document workspace quieter and more editorial, closer to a refined Notion or Linear surface. Avoid loud color and heavy shadows.
```

Expected primary reference:

- `references/minimalist-ui.md`

Pass criteria:

- Chooses the minimalist/editorial/workspace route.
- Uses warm monochrome, flat components, sparse accents, and typographic structure.
- Keeps shadows minimal and color meaningful.
- Avoids loud hero fields, neon, and thick glassmorphism.

Failure modes:

- Applies cinematic marketing motion to a workspace.
- Uses loud gradients or bright primary color blocks.
- Adds rounded-full large cards or heavy Tailwind shadows.

## Case 4: Scroll storytelling/GSAP

Prompt:

```text
Build a long-form product story with pinned sections, scrubbed image transitions, and a strong AIDA page arc.
```

Expected primary reference:

- `references/gpt-taste.md`

Pass criteria:

- Chooses the scroll storytelling/GSAP route.
- Uses AIDA structure and a pre-flight design plan.
- Handles pinning, stacking, scrubbing, and image motion as isolated motion concerns.
- Checks GSAP availability before importing it.

Failure modes:

- Treats this as a static landing page.
- Uses window scroll listeners for per-frame animation.
- Creates narrow multi-line hero headings or cheap section labels.

## Case 5: Industrial/brutalist/dashboard

Prompt:

```text
Create a dense operational dashboard that feels like a declassified aerospace terminal: mechanical, raw, technical, and data-heavy.
```

Expected primary reference:

- `references/industrial-brutalist-ui.md`

Pass criteria:

- Chooses the industrial/brutalist/dashboard route.
- Picks one substrate: Swiss industrial print or tactical telemetry.
- Uses rigid grids, square geometry, technical type, and utilitarian color.
- Keeps density intentional and semantic for data-heavy UI.

Failure modes:

- Mixes light print and dark terminal skins in one surface.
- Adds glossy consumer glassmorphism or rounded SaaS cards.
- Uses decorative texture that harms readability or contrast.
