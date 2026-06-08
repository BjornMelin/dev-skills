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
