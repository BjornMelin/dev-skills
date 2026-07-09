---
name: r3f-scene-polish
description: Art-direct an EXISTING three.js / React Three Fiber scene to premium, cinematic quality — postprocessing (bloom, depth-of-field, N8AO, tone mapping), HDRI/PBR lighting, contact & accumulative shadows, transmission/glass materials, and camera choreography. Use for look-development and "make this 3D scene cinematic / premium / hyperrealistic" once it already renders. Do NOT use for scene setup or correctness — route Canvas/createRoot lifecycle, loaders, GLTF, disposal, SSR/client boundaries, and DPR to web-three-r3f; native (Expo/RN) motion to expo-motion; repo-wide motion-system direction to design-motion-system.
license: MIT
---

# R3F Scene Polish — Cinematic Look-Dev

Take an **already-working** three.js / React Three Fiber scene and art-direct it
to a premium, cinematic result. This skill owns the *aesthetic* layer:
postprocessing, physically-based lighting, shadows, materials, tone mapping, and
camera choreography.

## Boundary (read first)

This skill is the **second half** of a 3D scene — the look-dev pass after it
renders correctly. It does **not** own setup or correctness:

- Canvas/createRoot lifecycle, loaders, GLTF, disposal, SSR/client boundaries,
  DPR, "blank canvas" and resize bugs → **`web-three-r3f`**.
- Native (Expo / React Native) motion → **`expo-motion`**.
- Cross-stack motion-system direction / tokens / repo-wide upgrades →
  **`design-motion-system`**.

If the scene doesn't render yet, start in `web-three-r3f`, then return here.

## Before implementing

- Read `references/art-direction.md` — the current-API (2026) look-dev recipes:
  the `@react-three/postprocessing` quality ladder, `drei` `Environment` /
  `ContactShadows` / `AccumulativeShadows` / `MeshTransmissionMaterial`, tone
  mapping (AgX vs ACES), selective bloom, and the WebGPU/TSL caveat. **Verify
  every API against the repo's installed versions before editing** — the
  reference pins the versions it was written against.
- Read `references/motion-vocabulary.md` for named camera/lighting decisions.
- Read `references/performance-accessibility.md` before final QA.

## Look-dev rules

1. **Light for form.** HDRI (`Environment`) for image-based lighting; add
   key/fill/rim intentionally; ground the subject with `ContactShadows` or
   `AccumulativeShadows` (or float it deliberately).
2. **Tone-map deliberately.** three-core defaults to `NoToneMapping`; the R3F
   `<Canvas>` defaults to `ACESFilmicToneMapping`. Prefer `AgX` / `Neutral` for
   accurate highlight rolloff; don't double-tonemap (renderer *or* the
   postprocessing `ToneMapping` effect, not both).
3. **Build a postprocessing quality ladder**, not a pile-up: order effects
   intentionally, gate expensive ones (N8AO, DoF) behind a device/DPR quality
   tier, and give reduced-motion / low-power a lighter branch.
4. **Prefer PBR material values, HDRI, and physically-plausible lighting** over
   fake tricks; use `MeshTransmissionMaterial` for glass, sharing one FBO for
   many transmissive objects.
5. **Choreograph the camera** with intent (dolly/truck/orbit + easing), not large
   spins for basic feedback; keep text legible through the motion.
6. **Animate hot paths cheaply** (refs + delta-time in `useFrame`); this skill
   assumes `web-three-r3f`'s correctness rules (no `setState` in the frame loop,
   reuse math objects, dispose) — see that skill, don't re-derive them here.
7. **Always add a reduced-motion / low-power alternative** for camera travel,
   parallax, heavy postprocessing, and idle loops.

Return complete code changes or a complete look-dev plan, depending on the
request.

## Optional power tool: art-direction audit

This skill ships `scripts/audit.mjs`, a static auditor for R3F/three.js
**art-direction** quality — tone mapping (double-tonemap, legacy API), color
management (deprecated `outputEncoding`/`sRGBEncoding`), lighting (unlit scenes),
postprocessing quality (missing quality ladder, legacy `SSAO`, WebGPU mismatch), and
material color-space. This is the visual layer `web-three-r3f`'s lifecycle audit does
**not** cover; run both. Optional — findings are leads.

```bash
node scripts/audit.mjs doctor                    # list every rule
node scripts/audit.mjs scan --root . --format json
```

Verify each finding against the repo's installed package versions before changing behavior.
