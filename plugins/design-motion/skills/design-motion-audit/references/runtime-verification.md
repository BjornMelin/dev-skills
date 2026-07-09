# Runtime Verification

Static analysis (this skill's scripts, `motion-token-audit`, the R3F art-direction
audit, type-check, lint) proves *structure*. It cannot prove that a scene actually
renders, that motion holds its frame budget, or that reduced motion works — those
are runtime facts that need a running app. This reference is the runtime-proof path
for design-motion work; it wires **existing** tools, and every one is optional (if a
tool is absent, degrade and state what stayed unverified — never imply a pass).

## When runtime proof is required

Any change with a visible motion surface. A green static audit is necessary but not
sufficient. Match depth to blast radius: a small tween → a screenshot; a hero 3D
scene or a gesture system → interaction + a frame-budget sample + reduced-motion
check. For a full run, use the `motion-runtime-verifier` subagent (design-motion
plugin), which orchestrates the steps below and returns a pass / pass-with-risks /
block verdict into the `motion-qa-reviewer` gate.

## Web (three.js / R3F / GSAP / CSS / Motion)

1. **Launch** with the `run` skill or the project's dev server → a URL.
2. **Drive with `playwright-cli`** (all optional; use what is installed):
   - **Non-blank render** — `eval` a `<canvas>` readback (`readPixels` / `toDataURL`)
     to prove the scene drew, not a black frame.
   - **Interaction & responsive** — `click`/`hover`/`drag`/`scroll`, then `resize`.
   - **Frame budget** — `eval` a short `requestAnimationFrame` + `performance.now()`
     sampler to estimate FPS / long frames during motion.
   - **Capture** — `screenshot` key states; `video-start`/`video-stop` for
     interactive motion; read `console`/`requests` for errors.
3. **Animation cost grade** — if available, `npx motionscore <url> --agent` returns
   an S→F render-pipeline grade plus max-concurrent-animations, GPU pressure, and
   reduced-motion checks. The session's Motion MCP (`generate-css-spring`,
   `visualise-spring`) can generate/preview CSS springs. If neither is present, use
   the playwright FPS sample and say the S→F grade is unavailable.
4. **Reduced motion** — re-run with `prefers-reduced-motion: reduce` emulated;
   confirm decorative travel/loops drop while functional feedback stays.

## Native (Expo / React Native)

Do not use a browser. The mature device-proof ladder already exists at
`skills/expo-motion/references/validation.md`: Expo Doctor, `expo install --check`,
New-Architecture verification, a dev build on a real device, the 5-tier risk ladder,
and a closeout naming device/OS with an attached recording. Unit tests cannot assert
frame pacing, dropped frames, or GPU output.

## Degradation contract

- No browser driver → report what static gates covered and name the exact runtime
  claim left UNVERIFIED (do not imply it passed).
- No MotionScore → playwright FPS sampler; S→F grade unavailable.
- Headless-only / no dev server → capture what you can, flag the gap.

Feed the result into the audit punch list and the `motion-qa-reviewer` launch gate.
