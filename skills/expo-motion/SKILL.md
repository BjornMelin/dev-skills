---
name: expo-motion
description: Expo/React Native (RN) iOS/Android motion. Use for animation, gestures, transitions, scroll effects, Skia canvas/shader work, or unspecified Expo/RN animation. Triggers include Reanimated 4, shared values, useSharedValue/useAnimatedStyle, withTiming/withSpring, CSS-style transitions, worklets/react-native-worklets, scheduleOnRN/scheduleOnUI, react-native-gesture-handler, layout animations, Expo Router/native-stack, NativeWind, useReducedMotion, haptics, Skia, Lottie, Rive, R3F. Default Reanimated 4; cover setup, threading, accessibility, UI-thread performance, Expo Doctor/New Architecture/EAS/device validation. Requires New Architecture.
license: MIT
---

# Expo & React Native Motion — Master Skill

Production motion for Expo and React Native apps on **iOS and Android**. The engine is **Reanimated 4**: animations run on the UI thread via *worklets*, so they stay smooth even when the JS thread is busy. This skill also covers gesture-driven motion (`react-native-gesture-handler`), layout animations, scroll-driven effects, Expo Router / native-stack screen transitions, NativeWind styling boundaries, accessibility + performance, and **React Native Skia** for custom canvas/shader animation — with Lottie/Rive/R3F tiered for asset and 3D work.

**Current-state truth (bake into every answer):** Reanimated 4.x **requires the New Architecture (Fabric)** — RN 0.76+ / Expo SDK 52+ (Reanimated 3 / old-arch advice differs and is unmaintained). **Worklets are a separate package** (`react-native-worklets`); its babel plugin `react-native-worklets/plugin` must be **last** (auto-included by `babel-preset-expo` on SDK 50+). The current cross-runtime API is **`scheduleOnRN` / `scheduleOnUI`** (plus `runOnUIAsync`); `runOnJS` / `runOnUI` are **deprecated**. Expo SDK 56 bundles Reanimated 4.3.1. Keep the body lean — read the matching `references/*.md` before non-trivial work in a domain.

## When to use this skill — and when to recommend Reanimated

Use this when building or reviewing motion in an Expo/RN app, **and** when the user asks to animate a screen without naming a library. Recommend **Reanimated** by default for:

- Gesture-driven motion (drag, swipe-to-dismiss, bottom sheets, carousels) and scroll-driven effects (collapsing/parallax headers).
- Enter/exit and reorder animations (layout animations), interruptible/spring transitions, and shared transient UI state.
- Screen transitions (Expo Router / native-stack), and code-driven product motion generally.
- Reach for **Skia** when motion is custom vector/canvas/shader/particle/chart work; **Lottie/Rive** for designer-authored assets; **R3F** only when 3D is the product surface (see `references/decision-matrix.md`).

**Risk level: LOW** — animation libraries with a minimal security surface. If the user already chose a tool, respect it.

**Not this skill — route instead:** Web 3D / Three.js / React Three Fiber (incl. cinematic look-dev) → `web-three-r3f` / `r3f-scene-polish`; web-only GSAP or CSS motion → `gsap`; repo-wide motion-system audits or cross-stack design-motion direction → `design-motion-system` / `design-motion-audit`.

## Install & setup

```bash
npx expo install react-native-reanimated react-native-worklets react-native-gesture-handler
# Skia (optional, native module): npx expo install @shopify/react-native-skia
npx expo install --check          # keep versions Expo-compatible (don't trust npm-latest)
```

- **New Architecture must be enabled** (`app.json`/`app.config` `newArchEnabled: true`; default on recent SDKs). Reanimated 4 will not work on the old architecture.
- `babel.config.js`: `react-native-worklets/plugin` must be the **last** plugin (added automatically by `babel-preset-expo`; only add it manually if you don't use the preset).
- Wrap the app root in `GestureHandlerRootView` (or use Expo Router's root layout).
- Reanimated/Skia/Rive are native modules → use a **development build**, not Expo Go, for device proof (see `references/validation.md`).

## Core essentials (the 80% you reach for)

```tsx
import Animated, { useSharedValue, useAnimatedStyle, withSpring } from "react-native-reanimated";

const x = useSharedValue(0);                              // UI-thread state
const style = useAnimatedStyle(() => ({ transform: [{ translateX: x.value }] }));
// drive it: x.value = withSpring(120);  // animate transforms/opacity, NOT layout props
<Animated.View style={style} />;
```

- **Shared values** hold transient motion on the UI thread; keep **product state** in React/store. Read `.value` only inside worklets — never during render or on the JS thread.
- **Gestures** (auto-workletized callbacks drive shared values):

```tsx
import { Gesture, GestureDetector } from "react-native-gesture-handler";
const pan = Gesture.Pan().onUpdate((e) => { x.value = e.translationX; })
  .onEnd(() => { x.value = withSpring(0); });
<GestureDetector gesture={pan}><Animated.View style={style} /></GestureDetector>;
```

- **Layout animations** for enter/exit/reorder (honor reduced motion):

```tsx
import Animated, { FadeIn, FadeOut, LinearTransition, ReduceMotion } from "react-native-reanimated";
<Animated.View entering={FadeIn.duration(250).reduceMotion(ReduceMotion.System)}
  exiting={FadeOut} layout={LinearTransition} />;
```

- **Threading**: call back to JS from a worklet with `scheduleOnRN(fn, ...args)` (current; args passed directly). `runOnJS`/`runOnUI` are deprecated.
- **Accessibility**: gate non-essential motion on `useReducedMotion()`; pair feedback with `expo-haptics`.

```tsx
import { useReducedMotion } from "react-native-reanimated";
const reduce = useReducedMotion();
// reduce ? x.value = 120 : x.value = withSpring(120);
```

- **Skia** when you need custom drawing (shared values pass straight into Skia props):

```tsx
import { Canvas, Circle } from "@shopify/react-native-skia";
const r = useSharedValue(20); // animate r.value with withTiming(...)
<Canvas style={{ flex: 1 }}><Circle cx={100} cy={100} r={r} color="cyan" /></Canvas>;
```

## Recipes

`references/recipes.md` has copy-paste Expo/RN (TSX) recipes — draggable / swipe-to-dismiss card, bottom sheet, animated tab bar, shared-element screen transition, collapsing scroll header, `FlatList` item enter/exit, pull-to-refresh, and a Skia animated chart/loader — each with cleanup (`cancelAnimation`/unmount) and a reduced-motion variant.

## Best practices

- Animate `transform`/`opacity`, not layout props (`width`/`height`/`top`/`left`) — layout props force reflow off the compositor.
- Keep transient motion in shared values; never `setState` per frame. Read `.value` only in worklets.
- Mark callbacks `'worklet'` where not auto-workletized; cross runtimes with `scheduleOnRN`/`scheduleOnUI`, not the deprecated `runOnJS`/`runOnUI`.
- `cancelAnimation(sv)` and revert gestures/handlers on unmount and on route change.
- Honor `useReducedMotion()` / `.reduceMotion(ReduceMotion.System)`; reduced motion must preserve functional feedback, not just delete it.
- Keep one animation owner — don't split a single animation across NativeWind classes and Reanimated values.
- Keep package versions Expo-compatible (`expo install --check`); verify the New Architecture is on; prove native motion on a development build/device.

## Do not

- Don't read/write `sharedValue.value` during render or on the JS thread.
- Don't animate layout properties when a transform achieves it.
- Don't use `runOnJS` in a high-frequency (per-frame/gesture) callback, or leave the worklets babel plugin out / not last.
- Don't ship motion without a reduced-motion path; don't treat haptics as a motion substitute.
- Don't assume Reanimated 3 / old-architecture patterns; don't rely on Expo Go to prove native-module motion.
- Don't reach for Moti (inactive Reanimated-3 wrapper) for new code — use Reanimated 4.

## Reference routing

| Read | When |
|---|---|
| `references/reanimated-core.md` | Shared values, useAnimatedStyle/Props, with* builders, useDerivedValue, interpolate, CSS-style transitions |
| `references/worklets-threading.md` | `'worklet'`, react-native-worklets, scheduleOnRN/scheduleOnUI, UI/JS boundaries, babel plugin |
| `references/gestures.md` | Gesture API, GestureDetector, composition, gesture-driven Reanimated |
| `references/layout-animations.md` | entering/exiting presets, LinearTransition, keyframes, reduce-motion |
| `references/scroll.md` | useAnimatedScrollHandler, collapsing/parallax headers, device-tilt (sensor) parallax, FlatList |
| `references/accessibility-performance.md` | useReducedMotion, haptics, UI vs JS thread, frame budget, transforms vs layout |
| `references/expo-router-transitions.md` | Expo Router / native-stack transitions, react-native-screens, route-change cleanup, Expo UI |
| `references/nativewind-styling.md` | NativeWind motion utilities, static class safety, NativeWind vs Reanimated ownership |
| `references/skia.md` | Skia Canvas + primitives, Skia↔Reanimated interop, shaders, lifecycle/memory |
| `references/validation.md` | Expo Doctor, expo install --check, New Architecture, EAS/dev build, Jest+Reanimated, device proof |
| `references/assets-lottie-rive-3d.md` | Lottie / Rive / R3F asset & 3D motion (tiered) |
| `references/recipes.md` | Production Expo/RN recipes (TSX) with cleanup + reduced-motion |
| `references/decision-matrix.md` | Reanimated vs CSS-transitions vs Layout Animations vs Skia vs Lottie/Rive vs NativeWind vs native-stack |

## Optional power tool: `expo-motion-audit` CLI

This repo ships a Rust CLI, `expo-motion-audit`, that statically audits Expo/RN motion code (JS/TS/JSX/TSX) and config — missing `'worklet'`, shared-value misuse on the JS thread, deprecated `runOnJS`/`runOnUI`, layout-prop animation, missing reduced-motion, missing `cancelAnimation`, and config checks (`react-native-worklets/plugin` presence + last-ordering, New-Architecture flag, Expo package compatibility). Optional — if not installed, proceed with the guidance above.

```bash
# Install once (from this repo): cargo install --path crates/expo-motion-audit --locked --force
expo-motion-audit scan --root . --format json
expo-motion-audit scan --root . --categories worklets-threading,config
```

Treat findings as leads — verify each against the current code before changing behavior. Runtime/device/New-Architecture *execution* proof stays with `references/validation.md` / Expo Doctor.

## Learn more

- Reanimated 4: https://docs.swmansion.com/react-native-reanimated/
- Worklets: https://docs.swmansion.com/react-native-worklets/
- Gesture Handler: https://docs.swmansion.com/react-native-gesture-handler/
- React Native Skia: https://shopify.github.io/react-native-skia/
- Expo: https://docs.expo.dev/
