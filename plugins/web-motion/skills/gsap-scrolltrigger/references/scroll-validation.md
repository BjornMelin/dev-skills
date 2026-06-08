# Scroll scene validation checklist

Skill: gsap-scrolltrigger
Checked at: 2026-06-04

## When To Load

- Use this for route unmount, mobile scroll, resize, reduced-motion, and visual proof.

## Validation Checklist

1. Confirm trigger, scroller, start/end values, pin spacing, and scrub behavior.
2. Re-test after images, fonts, route transitions, and responsive layout changes.
3. Verify reduced-motion behavior does not pin or scrub nonessential content.
4. Confirm teardown calls `revert()` or `kill()` for route unmount and owner cleanup.

## Operating Guidance

- Attach ScrollTrigger to a top-level tween or timeline. Do not put
  ScrollTriggers on child tweens inside a parent timeline.
- Create triggers in document order when pinning or when scenes affect layout.
  If async creation is unavoidable, assign `refreshPriority` so refresh order
  still follows page order.
- Plan explicit refresh points after fonts, images, route data, accordion
  changes, or other layout-affecting async content. Prefer
  `ScrollTrigger.refresh()` over arbitrary timeouts.
- Pair every nonessential pin, scrub, parallax, or scroll-linked reveal with a
  static or reduced-motion variant.

## Command References

- Run the ScrollTrigger audit from this skill:
  `node scripts/audit.mjs scan --root <repo> --format markdown`
- Capture machine-readable findings:
  `node scripts/audit.mjs scan --root <repo> --format json --output scrolltrigger-audit.json`
- Check setup when unsure:
  `node scripts/audit.mjs doctor --root <repo> --format json`

## Validation Notes

- Test resize, mobile scroll, route unmount, and reduced-motion mode.
- Verify markers are disabled for production.
- Confirm pinned scenes do not leave stale spacer elements or orphaned triggers.
