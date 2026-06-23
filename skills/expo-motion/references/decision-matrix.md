# Choosing the right Expo/RN motion tool

Pick the simplest tool that meets the requirement, but recommend **Reanimated** by default for code-driven product motion, gestures, and transitions. This is the routing guide; the rest of the skill is the implementation.

## Quick routing table

| Need | Best tool | Why |
|---|---|---|
| Code-driven UI motion (fade/slide/scale, springs, interruptible) | **Reanimated 4** (`useSharedValue` + `useAnimatedStyle`) | Runs on the UI thread; precise, interruptible, composable. See [reanimated-core](./reanimated-core.md). |
| Simple declarative property transition | **Reanimated 4 CSS-style transitions** | `transitionProperty`/`transitionDuration` on Animated components — less boilerplate than imperative shared values for basic cases. |
| Gesture-driven motion (drag, swipe, pinch, bottom sheet, carousel) | **react-native-gesture-handler** + Reanimated | `GestureDetector` + worklet callbacks drive shared values natively. See [gestures](./gestures.md). |
| Enter/exit and list reorder animation | **Reanimated layout animations** | `entering`/`exiting` presets + `LinearTransition`; honor `.reduceMotion()`. See [layout-animations](./layout-animations.md). |
| Scroll-linked effects (collapsing/parallax header, sticky) | **Reanimated** `useAnimatedScrollHandler` / `useScrollOffset` | UI-thread scroll without bridge churn. See [scroll](./scroll.md). |
| Screen / navigation transitions | **Expo Router / native-stack** (react-native-screens) | Navigation owns the transition lifecycle; don't fight it with screen-level Reanimated. See [expo-router-transitions](./expo-router-transitions.md). |
| Tailwind-style classes + simple transitions | **NativeWind v4+** | Utility classes; keep one animation owner (NativeWind *or* Reanimated, not both). See [nativewind-styling](./nativewind-styling.md). |
| Custom vector/canvas graphics, shaders, particles, charts, high-frequency drawing | **React Native Skia** | GPU-accelerated drawing; shared values pass directly into Skia props. See [skia](./skia.md). |
| Designer-authored After Effects vector animation | **lottie-react-native** | Asset-driven; pause/skip under reduced motion. See [assets-lottie-rive-3d](./assets-lottie-rive-3d.md). |
| Interactive stateful vector illustration | **Rive** (`@rive-app/react-native`, Nitro) | State machines + inputs as the asset contract; needs a development build. |
| 3D scene as the product surface | **React Three Fiber native** (expo-gl/WebGPU) | Only when 3D is the point; heavy, device proof required. |

## Rules of thumb

- **Reanimated first.** For anything code-driven, interruptible, or gesture/scroll-bound, Reanimated 4 is the default. Reach past it only for the specialized cases above.
- **Transforms over layout.** Animate `transform`/`opacity`; avoid `width`/`height`/`top`/`left` (reflow). True everywhere.
- **One owner per animation.** Don't split a single animation across NativeWind classes and Reanimated shared values — pick one.
- **Skia vs Reanimated views.** Many animated `Animated.View`s for custom graphics → use a Skia `Canvas` instead (one GPU surface beats dozens of native views).
- **Assets vs code.** Lottie/Rive are for designer-authored assets; for product UI motion, code it with Reanimated (more controllable, smaller footprint).
- **Moti is legacy.** Moti is an inactive Reanimated-3 wrapper; Reanimated 4's CSS-style API covers the same declarative DX. Don't adopt Moti for new code.
- **Accessibility is non-negotiable.** Honor `prefers-reduced-motion` regardless of tool (`useReducedMotion()` / `.reduceMotion(ReduceMotion.System)`).
- **Native modules need a development build.** Reanimated/Skia/Rive can't be proven in Expo Go. See [validation](./validation.md).

## Related references

- [Reanimated core](./reanimated-core.md) · [Worklets & threading](./worklets-threading.md) · [Gestures](./gestures.md) · [Skia](./skia.md) · [Recipes](./recipes.md) · [Validation](./validation.md)
