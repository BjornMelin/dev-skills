# Support, @supports, and reduced-motion notes

Skill: web-css-animations
Checked at: 2026-06-04

## When To Load

- Read for newer CSS features or accessibility review.

## Support Checklist

1. Check `@supports` for newer features such as scroll timelines, registered properties, and discrete transitions.
2. Pair every nonessential transition or keyframe with `prefers-reduced-motion`.
3. Preserve DOM, focus, and ARIA semantics when visual motion changes visibility.
4. Verify fallback behavior in the repo browser policy before recommending a new CSS motion feature.
