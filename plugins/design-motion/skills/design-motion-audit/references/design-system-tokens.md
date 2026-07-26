# Design Motion Tokens Reference

A motion system must be tokenized before it becomes reliable across agents, routes, and platforms.

## Token groups

- Durations: instant, micro, short, medium, long, cinematic.
- Easing: out, inOut, linear, brand, emphasized.
- Springs: snappy, soft, bouncy, heavy, gesture.
- Depth: none, card, sheet, modal, hero, world.
- Shadows: rest, hover, pressed, modal, hero.
- Parallax: none, subtle, medium, hero.
- Blur: none, backdrop, glass, depth.
- Glow: none, subtle, active, hero.
- Quality: high, medium, low, reduced.
- Reduced motion: instant, fadeOnly, noParallax, noLoop, noCameraTravel.

## Suggested TypeScript shape

```ts
// One const, all ten token groups. Values are a starting point — tune per brand.
export const motion = {
  duration: { instant: 0, micro: 120, short: 200, medium: 360, long: 700, cinematic: 1200 },
  easing: {
    out: [0.16, 1, 0.3, 1],
    inOut: [0.65, 0, 0.35, 1],
    emphasized: [0.2, 0, 0, 1],
    brand: [0.34, 1.2, 0.64, 1],
    linear: [0, 0, 1, 1],
  },
  spring: {
    snappy: { stiffness: 520, damping: 42, mass: 0.85 },
    soft: { stiffness: 220, damping: 28, mass: 1 },
    bouncy: { stiffness: 340, damping: 20, mass: 0.9 },
    heavy: { stiffness: 180, damping: 34, mass: 1.4 },
    gesture: { stiffness: 420, damping: 36, mass: 1 },
  },
  depth: {
    none: { z: 0, scale: 1 },
    card: { z: 12, scale: 1.02 },
    sheet: { z: 32, scale: 1.0 },
    modal: { z: 48, scale: 1.0 },
    hero: { z: 80, scale: 1.06 },
    world: { z: 160, scale: 1.1 },
  },
  shadow: {
    rest: "0 1px 2px rgba(0,0,0,0.12)",
    hover: "0 6px 16px rgba(0,0,0,0.16)",
    pressed: "0 1px 1px rgba(0,0,0,0.20)",
    modal: "0 24px 60px rgba(0,0,0,0.28)",
    hero: "0 40px 120px rgba(0,0,0,0.35)",
  },
  parallax: { none: 0, subtle: 0.035, medium: 0.08, hero: 0.12 },
  blur: { none: 0, backdrop: 8, glass: 16, depth: 24 },
  glow: { none: 0, subtle: 0.15, active: 0.4, hero: 0.8 },
  quality: {
    high: { dpr: 2, samples: 8, shadows: true },
    medium: { dpr: 1.5, samples: 4, shadows: true },
    low: { dpr: 1, samples: 0, shadows: false },
    reduced: { dpr: 1, samples: 0, shadows: false },
  },
  reducedMotion: { instant: true, fadeOnly: true, noParallax: true, noLoop: true, noCameraTravel: true },
} as const;
```

## CSS variable naming

```css
:root {
  --motion-duration-micro: 120ms;
  --motion-duration-short: 200ms;
  --motion-duration-medium: 360ms;
  --motion-duration-long: 700ms;
  --motion-ease-out: cubic-bezier(0.16, 1, 0.3, 1);
  --motion-ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);
  --motion-ease-brand: cubic-bezier(0.34, 1.2, 0.64, 1);
  --motion-parallax-subtle: 0.035;
  --motion-parallax-hero: 0.12;
  --motion-shadow-rest: 0 1px 2px rgba(0, 0, 0, 0.12);
  --motion-shadow-hover: 0 6px 16px rgba(0, 0, 0, 0.16);
  --motion-shadow-hero: 0 40px 120px rgba(0, 0, 0, 0.35);
  --motion-blur-glass: 16px;
  --motion-glow-active: 0.4;
}

/* Reduced motion: zero decorative parallax here; loop/travel/duration collapse
   lives in the `reducedMotion` token group in the TypeScript const above. */
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-parallax-subtle: 0;
    --motion-parallax-hero: 0;
  }
}
```

## Cross-stack mapping

- Web CSS uses durations, easing, opacity, transform, CSS variables.
- R3F uses durations, damping, camera offsets, layer depths, DPR quality, material values.
- Reanimated uses durations, easing arrays, spring configs, snap points, gesture thresholds.
- Reduced motion maps across all stacks to no camera travel, no parallax, no loops, no bounce, and fade-only state changes.

## Rules

1. Add a token before adding the third similar animation.
2. Name tokens by intent, not numeric value.
3. Use stack adapters instead of duplicating values.
4. Keep gesture thresholds with motion tokens, not hidden in components.
5. Include reduced-motion tokens beside normal tokens.
