# Motion Vocabulary for 3D Web and Native Apps

Use these terms in prompts, component names, variants, code comments, and acceptance criteria.

## Global principles

- Purposeful spectacle: large motion serves orientation, feedback, hierarchy, relationship, or product storytelling.
- Signature motion: repeated branded behavior that makes the app memorable.
- Spatial continuity: objects keep identity across states, screens, and depths.
- Material truth: glass, metal, plastic, fabric, liquid, and data motion each feel physically appropriate.
- Interruption-first design: interactive motion can redirect mid-flight without snapping.
- Frame-budget first: visual complexity degrades before the app becomes janky.
- Reduced-motion parity: state remains clear when large motion is removed.

## Entrances and exits

- Fade in and fade out: change opacity or material opacity.
- Slide in: translate from offscreen, edge, depth, or trigger position.
- Scale in: grow from small to full size, often with opacity.
- Pop in: enter with controlled overshoot and settle.
- Reveal: uncover with clip, mask, shader progress, layout, or material opacity.
- Dissolve: appear or disappear through noise, threshold, particles, or alpha.
- Portal reveal: reveal content through a plane, card, aperture, or route gateway.
- Layer cascade: foreground, midground, and background enter in staged sequence.

## Sequencing and timing

- Keyframes: defined points in a timeline.
- Tween: generated in-between values between start and end.
- Stagger: multiple elements animate one after another.
- Orchestration: camera, object, material, light, UI, and postprocessing move as one event.
- Delay: time before animation starts.
- Duration: fixed length of an animation.
- Fill mode: final or initial state persists before or after animation.
- Stepped animation: discrete changes such as tickers, counters, or frame sequences.
- Chapter: a scroll or route segment with one message and one motion event.

## 3D movement

- Translate: move on X, Y, or Z.
- Rotate: spin around an axis.
- Scale: resize uniformly or per axis.
- Pivot: parent origin for hinge, orbit, fold, and origin-aware motion.
- Quaternion blend: smooth rotation without gimbal artifacts.
- Slerp: spherical interpolation between orientations.
- LookAt: orient object or camera toward a target.
- Dolly: camera moves forward or backward.
- Truck: camera moves horizontally.
- Crane: camera moves vertically.
- Orbit: camera or object circles a target.
- Camera rail: constrained path for cinematic movement.
- Focus pull: depth-of-field focus moves between subjects.
- Parallax stack: layers move at different amplitudes by depth.
- Foreground occlusion: object passes in front of UI or text to create depth.

## Transitions between states

- Crossfade: one layer fades out as another fades in.
- Continuity transition: visual identity persists across states.
- Shared element transition: element appears to travel between states or routes.
- Morph: one shape, material, layout, or mesh becomes another.
- Layout animation: changed position or size animates instead of snapping.
- Accordion collapse: height expands or collapses.
- Direction-aware transition: forward and backward navigation move in opposite directions.
- Scene morph: full 3D composition transforms into another composition.
- HUD to world transition: screen-space UI becomes a world-space object.
- World to HUD transition: 3D object becomes a 2D UI element.

## Feedback and interaction

- Hover lift: object raises, scales, lights, or tilts on hover.
- Press depth: object compresses, darkens, or shadow shrinks on press.
- Tap ripple: circle or shader wave expands from input point.
- Hold to confirm: progress fills while press is held.
- Drag: object follows pointer or touch.
- Throw release: release velocity carries motion forward.
- Snap point: nearest valid resting state after release.
- Rubber-banding: resistance beyond bounds, followed by snap-back.
- Drag to reorder: list items shift as active item moves.
- Swipe to dismiss: gesture moves element offscreen to close.
- Error shake: side-to-side rejection signal.
- Magnetic hover: object, light, or particles attract toward pointer.

## Easing and physics

- Ease-out: fast start, slow end for direct responses.
- Ease-in-out: smooth movement between states already on screen.
- Linear: constant speed for loops, spinners, marquees, and shader time.
- Cubic bezier: custom curve for brand feel.
- Spring: physics-driven motion using stiffness, damping, mass, velocity, or duration-based spring config.
- Stiffness: how strongly the spring pulls to target.
- Damping: how quickly oscillation settles.
- Mass: perceived weight.
- Bounce: intentional overshoot and settle.
- Velocity carryover: gesture speed feeds into spring or decay.
- Decay: inertia after release.
- Interruptible animation: target can change mid-flight.

## Looping and ambient motion

- Idle animation: subtle motion while waiting for interaction.
- Float: gentle up-and-down movement.
- Pulse: repeating opacity, scale, glow, or emissive change.
- Orbital loop: continuous circular motion.
- Marquee: content scrolls continuously.
- Alternate or yoyo: loop reverses every iteration.
- Breathing material: roughness, glow, opacity, or shader intensity changes slowly.
- Ambient camera drift: tiny camera movement after hero settles.
- Dust motes: small particles that reveal light and depth.
- Data particles: particles implying information flow.
- Loop hygiene: pause or reduce loops offscreen and under reduced motion.

## Polish and effects

- PBR material: physically based material values for roughness, metalness, clearcoat, transmission, and lighting.
- HDRI environment: high-dynamic-range image used for reflections and lighting.
- Contact shadow: tight grounding shadow below objects.
- Light sweep: animated highlight across a material.
- Reflection sweep: moving reflection cue on glossy surfaces.
- Bloom: glow around bright or emissive areas.
- Selective bloom: glow applied only to chosen objects.
- Depth of field: blur by distance from focus plane.
- SSAO: screen-space ambient occlusion for creases and contact depth.
- Fresnel: edge brightness based on view angle.
- Shader ripple: interaction wave in UVs, vertices, or screen space.
- Dissolve shader: noise-threshold reveal or exit.
- Trail: fading geometry or screen-space afterimage following motion.
- Skeleton shimmer: loading placeholder with moving sheen.
- Number ticker: digits roll or count to new value.
- Tabular numbers: fixed-width digits preventing layout shift.

## Performance vocabulary

- Draw call: GPU submission to render geometry.
- Instancing: render many copies with one draw call.
- LOD: use simpler geometry at distance.
- DPR clamp: limit device pixel ratio for GPU budget.
- On-demand rendering: render only when scene changes.
- Hot frame loop: per-frame code path that must avoid allocations and React state.
- Worklet: Reanimated function running on the UI thread.
- Shared value: mutable animation value synchronized for UI-thread updates.
- Layout thrashing: animating layout properties that force recalculation.
- Compositing: moving or fading layer without repainting layout.
- Overdraw: drawing too many overlapping transparent layers.
- Quality ladder: high, medium, low, reduced-motion variants.
