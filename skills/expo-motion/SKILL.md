---
name: expo-motion
description: "Expo and React Native motion for iOS, Android, and web: Reanimated 4, worklets, gestures, transitions, Skia, accessibility, and manifest-driven validation."
license: MIT
---

# Expo & React Native Motion — Master Skill

Production motion for Expo and React Native apps on iOS, Android, and web. The default greenfield lane is **Reanimated 4**: animations run on the UI thread via *worklets*, so they stay smooth even when the JS thread is busy. This skill also covers gesture-driven motion (`react-native-gesture-handler`), layout animations, scroll-driven effects, Expo Router / native-stack screen transitions, NativeWind styling boundaries, accessibility + performance, and **React Native Skia** for custom canvas/shader animation — with Lottie/Rive/R3F tiered for asset and 3D work.

**Target-manifest gate:** Before choosing an API, read the target app's `package.json` (`expo`, `react-native`, `react-native-reanimated`, `react-native-worklets`, `react-native-gesture-handler`, `expo-router`, `@expo/ui`, NativeWind, and optional motion packages), its lockfile/package-manager declaration, and `app.json`/`app.config.*` (`newArchEnabled`, plugins, and platform settings). Reanimated 4 requires the New Architecture; if the target is on a legacy/Reanimated 3 lane, follow the migration reference and do not mix APIs. Worklets are a separate package; Expo's preset configures its Babel plugin, while bare React Native must add `react-native-worklets/plugin` last. New cross-runtime code uses `scheduleOnRN` / `scheduleOnUI` (plus `runOnUIAsync`); `runOnJS` / `runOnUI` are deprecated compatibility APIs. Keep the body lean — read the matching `references/*.md` before non-trivial work in a domain.

## When to use this skill — and when to recommend Reanimated

Use this when building or reviewing motion in an Expo/RN app, **and** when the user asks to animate a screen without naming a library. Recommend **Reanimated** by default for:

- Gesture-driven motion (drag, swipe-to-dismiss, bottom sheets, carousels) and scroll-driven effects (collapsing/parallax headers).
- Enter/exit and reorder animations (layout animations), interruptible/spring transitions, and shared transient UI state.
- Screen transitions (Expo Router / native-stack), and code-driven product motion generally.
- Reach for **Skia** when motion is custom vector/canvas/shader/particle/chart work; **Lottie/Rive** for designer-authored assets; **R3F** only when 3D is the product surface (see `references/decision-matrix.md`).

**Risk level: LOW** — animation libraries with a minimal security surface. If the user already chose a tool, respect it.

**Not this skill — route instead:** Web 3D / Three.js / React Three Fiber (incl. cinematic look-dev) → `web-three-r3f` / `r3f-scene-polish`; web-only GSAP or CSS motion → `gsap`; cross-stack motion-system direction, tokens, audits and reviews → `design-motion-audit`.

## Install & setup

```bash
# Use the target repo's documented Expo CLI/package-manager wrapper. Choose one
# dependency lane after reading the target manifest.
# Reanimated 4 + New Architecture:
<repo-expo> install react-native-reanimated react-native-worklets react-native-gesture-handler
# Reanimated 3 / legacy architecture:
<repo-expo> install react-native-reanimated react-native-gesture-handler
# Skia (optional): <repo-expo> install @shopify/react-native-skia
<repo-expo> install --check
```

- Resolve `<repo-expo>` from the target repo's `packageManager` field, lockfile, and scripts; do not copy a package-manager command from another project. Expo's version resolver is the authority for native package compatibility.
- **Reanimated 4 requires the New Architecture** (`app.json`/`app.config.*` `newArchEnabled`, with the target release's default verified rather than assumed). Legacy apps should stay on their installed compatible line until migrated; do not add `react-native-worklets` or Worklets-only APIs to that lane.
- `babel.config.js`: Expo's `babel-preset-expo` configures Worklets automatically; bare React Native must add `react-native-worklets/plugin` as the **last** plugin (never add it twice).
- Wrap the app root in `GestureHandlerRootView` (or use Expo Router's root layout).
- Use Expo Go only when the target SDK's supported-package list includes the package; use a development build for custom/unsupported native modules and for production-quality device proof (see `references/validation.md`).

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
- **Accessibility**: use `ReduceMotion.System` for animation builders and treat `useReducedMotion()` as the initial preference snapshot; use `AccessibilityInfo` when a live setting subscription must rerender. Pair feedback with `expo-haptics`.

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

`references/recipes.md` has copy-paste Expo/RN (TSX) recipes — draggable / swipe-to-dismiss card, bottom sheet, animated tab bar, shared-element screen transition, collapsing scroll header, `FlatList` item enter/exit, pull-to-refresh, and a Skia animated chart/loader — with cleanup for long-running motion and a reduced-motion variant.

Use the `FlatList` recipe only for small lists or after representative-device measurement.
`performance.layout-animation-in-list` reports per-cell layout animation as a performance lead;
large virtualized lists need a static/reduced-motion branch.

## Best practices

- Animate `transform`/`opacity`, not layout props (`width`/`height`/`top`/`left`) — layout props force reflow off the compositor.
- Keep transient motion in shared values; never `setState` per frame. Read `.value` only in worklets.
- Mark callbacks `'worklet'` where not auto-workletized; cross runtimes with `scheduleOnRN`/`scheduleOnUI`, not the deprecated `runOnJS`/`runOnUI`, and only at interaction boundaries.
- `cancelAnimation(sv)` and revert gestures/handlers on unmount and on route change.
- Honor `.reduceMotion(ReduceMotion.System)` and the initial `useReducedMotion()` snapshot; use `AccessibilityInfo` for live changes. Reduced motion must preserve functional feedback, not just delete it.
- Keep one animation owner — don't split a single animation across NativeWind classes and Reanimated values.
- Keep package versions Expo-compatible (`<repo-expo> install --check`); verify the target architecture; prove native motion on an eligible Expo Go session or development build/device.

## Do not

- Don't read/write `sharedValue.value` during render or on the JS thread.
- Don't animate layout properties when a transform achieves it.
- Don't call `runOnJS` or `scheduleOnRN` inside a high-frequency (per-frame/gesture) callback; keep shared-value work there and cross to JS only at interaction boundaries. Never leave the worklets babel plugin out / not last.
- Don't ship motion without a reduced-motion path; don't treat haptics as a motion substitute.
- Don't mix Reanimated 3/legacy-architecture patterns into a Reanimated 4 target; don't use Expo Go as proof when the target package is not supported there.
- Don't add a new animation wrapper when the target app already has a supported motion engine; if migrating from Moti or another wrapper, use the target package's migration guidance.

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
| `references/validation.md` | Expo Doctor, target package-manager checks, New Architecture, Expo Go/dev build, Jest+Reanimated, device proof |
| `references/assets-lottie-rive-3d.md` | Lottie / Rive / R3F asset & 3D motion (tiered) |
| `references/recipes.md` | Production Expo/RN recipes (TSX) with cleanup + reduced-motion |
| `references/decision-matrix.md` | Reanimated vs CSS-transitions vs Layout Animations vs Skia vs Lottie/Rive vs NativeWind vs native-stack |

## Optional power tool: `expo-motion-audit` CLI

This repo ships a Rust CLI, `expo-motion-audit`, that statically audits Expo/RN motion
source and config. It does not check package compatibility; keep that target-manifest concern
with the install gate above. The rule catalog at `crates/expo-motion-audit-core/src/rules.rs`
is authoritative. Use the exact IDs below in reports and baselines; abbreviated descriptions are
not CLI IDs. Optional. If it is not installed, proceed with the guidance above.

| Rule ID | Lead |
| --- | --- |
| `reanimated-core.layout-prop-animation` | Layout props animate in an animated style. |
| `reanimated-core.shared-value-reassign` | A shared-value binding is reassigned instead of writing `.value`. |
| `worklets-threading.deprecated-run-on` | Deprecated `runOnJS` or `runOnUI` is used. |
| `worklets-threading.value-access-on-js` | A shared value is read or written during render. |
| `worklets-threading.bridge-in-hot-path` | JS bridging occurs in an animated reaction or gesture callback. |
| `worklets-threading.missing-worklet` | An extracted animated callback lacks `'worklet'`. |
| `layout.infinite-repeat-no-reduced-motion` | An infinite repeat has no reduced-motion reference. |
| `accessibility.missing-reduced-motion` | Reanimated use has no reduced-motion handling. |
| `lifecycle.missing-cancel-animation` | Animated shared values have no `cancelAnimation` reference. |
| `config.worklets-plugin-missing-or-not-last` | The Worklets Babel plugin is absent or misordered. |
| `config.deprecated-reanimated-plugin` | The deprecated Reanimated Babel plugin is configured. |
| `config.new-arch-disabled` | Reanimated 4 is used while New Architecture is explicitly disabled. |
| `config.unable-to-analyze` | A dynamic config could not be analyzed. |
| `performance.layout-animation-in-list` | A `FlatList`/`SectionList` cell animates entering, exiting, or layout. |

```bash
# Install once (from this repo): cargo install --path crates/expo-motion-audit --locked --force
expo-motion-audit doctor --format json
expo-motion-audit scan --root . --format json
expo-motion-audit scan --root . --categories worklets-threading,performance,config
```

Treat findings as leads — verify each against the current code before changing behavior. Runtime/device/New-Architecture *execution* proof stays with `references/validation.md` / Expo Doctor.

## Learn more

- Expo versioned reference: https://docs.expo.dev/versions/latest/
- Reanimated 4: https://docs.swmansion.com/react-native-reanimated/
- Reanimated 3→4 migration: https://docs.swmansion.com/react-native-reanimated/docs/guides/migration-from-3.x/
- Worklets: https://docs.swmansion.com/react-native-worklets/
- Gesture Handler: https://docs.swmansion.com/react-native-gesture-handler/
- React Native Skia: https://shopify.github.io/react-native-skia/
- Expo Router native stack: https://docs.expo.dev/versions/latest/sdk/router/stack/
- Expo UI universal: https://docs.expo.dev/versions/latest/sdk/ui/universal/
