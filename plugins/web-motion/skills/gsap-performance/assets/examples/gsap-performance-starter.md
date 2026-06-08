# GSAP performance note

- Prefer x/y/scale/rotation/autoAlpha for hot-path UI motion.
- Treat width, height, top, left, filter, box-shadow, and text layout as measured exceptions.
- Verify reduced-motion and cleanup before closeout.
