# Expo WebGPU And Three.js Notes

Use this reference only when the task explicitly chooses native WebGPU,
`react-native-wgpu`, GPU compute, or Three's WebGPU renderer. It is not the
default native R3F path. For ordinary native Three.js scenes, use
`@react-three/fiber/native` with `expo-gl`; see
`r3f-native-api-notes.md`.

## When WebGPU Is Worth The Risk

- The feature needs compute shaders, very high particle counts, procedural
  simulation, or Three WebGPU renderer features that WebGL cannot provide.
- The app can use a development/custom native build and can validate iOS and
  Android device behavior.
- The repo accepts experimental dependency and renderer risk.

Use Expo GL/R3F native for scene-graph-first 3D, Skia for 2D canvas workloads,
and Reanimated for normal UI motion.

## Setup Checks

- Inspect the installed `react-native-wgpu`, `three`, `@react-three/fiber`,
  React Native, React, Reanimated, and Worklets versions before coding.
- Verify New Architecture and native build requirements from the installed
  package and official docs.
- Follow the target repo's package manager. Do not copy package-manager flags
  from examples without translating them to repo policy.
- Treat WebGPU support as native-risk: Expo Go is not enough for proof.

## Renderer Differences To Prove

- Three WebGPU usually imports from `three/webgpu`, not plain `three`.
- React Native WebGPU canvases need device-pixel-ratio sizing.
- Native WebGPU contexts require explicit frame presentation after submitted
  commands.
- R3F integration may need a custom root/canvas adapter because the standard
  native `Canvas` is built around Expo GL/WebGL.
- Any custom adapter must unmount the R3F root and destroy renderer/device
  resources on route exit.

## Validation

- Native build or EAS proof for each target platform.
- Device rendering proof, not just simulator proof.
- Resize/orientation, background/foreground, route unmount/remount, memory, and
  frame pacing.
- Fallback behavior for unsupported devices or failed GPU initialization.
