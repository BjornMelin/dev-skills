# Performance and Accessibility Gates

## R3F and three.js gates

- No React state in `useFrame` or high-frequency pointer handlers.
- Use delta time.
- Reuse temporary objects.
- Clamp DPR.
- Use demand rendering for mostly static scenes.
- Use instancing for repeated geometry.
- Reuse materials and geometries.
- Compress GLB and textures.
- Budget postprocessing passes.
- Budget dynamic shadows.
- Add low, medium, high, and reduced quality paths.
- Pause loops, particles, and shader time when hidden.

## Reanimated and Expo gates

- Gesture and hot animation logic runs on shared values and worklets.
- Avoid per-frame JS-thread state updates.
- Preserve velocity on release.
- Cancel and retarget animations on interruption.
- Use transform and opacity where possible.
- Budget shadows, blur, list cell animations, and layout transitions.
- Stop repeating animations offscreen.
- Validate on real iOS device and release build.

## Reduced-motion gates

- Disable camera travel, orbit, aggressive zoom, parallax, particles, loops, and sensor motion.
- Replace large movement with opacity, small transform, or instant state.
- Remove bounce and overshoot where it can cause discomfort.
- Keep pressed, selected, success, error, and focus states clear.
- Provide skip behavior for long hero intros.

## Accessibility gates

- Preserve readable text during motion.
- Avoid rapid flashes.
- Keep touch targets stable and large enough.
- Provide keyboard focus states for interactive elements.
- Do not hide critical state only in motion.
- Do not trap users in scroll-driven or gesture-only interactions.
