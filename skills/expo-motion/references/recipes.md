# Expo / React Native Motion Recipes (Reanimated 4, TSX)

Copy-paste, production-minded recipes for iOS + Android. All assume Reanimated 4 + the New Architecture, `react-native-gesture-handler` (app wrapped in `GestureHandlerRootView`), and `react-native-worklets` (babel plugin last).

Conventions used throughout:
- Transient motion lives in **shared values**; product state stays in React/store.
- Animate **`transform`/`opacity`**, never layout props.
- Every recipe has a **reduced-motion** path via `useReducedMotion()` (or `.reduceMotion(ReduceMotion.System)` on layout animations) — reduced motion keeps the functional outcome, it just removes the movement.
- Clean up with `cancelAnimation` on unmount; cross to JS with `scheduleOnRN` (not the deprecated `runOnJS`).

## Table of contents
1. [Swipe-to-dismiss card](#1-swipe-to-dismiss-card)
2. [Bottom sheet](#2-bottom-sheet)
3. [Animated tab-bar indicator](#3-animated-tab-bar-indicator)
4. [Screen transition (Expo Router / native-stack)](#4-screen-transition)
5. [Collapsing scroll header](#5-collapsing-scroll-header)
6. [FlatList item enter/exit + reorder](#6-flatlist-item-enterexit--reorder)
7. [Custom pull-to-refresh indicator](#7-pull-to-refresh)
8. [Skia animated loader / chart](#8-skia-animated-loader)

---

## 1. Swipe-to-dismiss card

Pan to dismiss; spring back if the throw is small. `scheduleOnRN` calls the JS `onDismiss` from the worklet.

```tsx
"use client";
import { useReducedMotion, useSharedValue, useAnimatedStyle, withSpring, withTiming } from "react-native-reanimated";
import Animated from "react-native-reanimated";
import { Gesture, GestureDetector } from "react-native-gesture-handler";
import { scheduleOnRN } from "react-native-worklets";

export function SwipeCard({ onDismiss }: { onDismiss: () => void }) {
  const x = useSharedValue(0);
  const reduce = useReducedMotion();

  const pan = Gesture.Pan()
    .onUpdate((e) => { x.value = e.translationX; })
    .onEnd((e) => {
      if (Math.abs(e.translationX) > 120) {
        x.value = reduce ? Math.sign(e.translationX) * 500 : withTiming(Math.sign(e.translationX) * 500, { duration: 200 });
        scheduleOnRN(onDismiss);
      } else {
        x.value = reduce ? 0 : withSpring(0);
      }
    });

  const style = useAnimatedStyle(() => ({ transform: [{ translateX: x.value }], opacity: 1 - Math.min(Math.abs(x.value) / 400, 1) }));
  return <GestureDetector gesture={pan}><Animated.View style={[styles.card, style]} /></GestureDetector>;
}
```

Reduced motion: the card still dismisses (functional outcome preserved) — it just jumps instead of throwing.

---

## 2. Bottom sheet

Drag a sheet between closed/open snap points; clamp and spring to the nearest.

```tsx
const OPEN = 0, CLOSED = 600;
const y = useSharedValue(CLOSED);
const reduce = useReducedMotion();
const start = useSharedValue(CLOSED);

const pan = Gesture.Pan()
  .onStart(() => { start.value = y.value; })
  .onUpdate((e) => { y.value = Math.max(OPEN, start.value + e.translationY); })
  .onEnd((e) => {
    const target = (y.value > 300 || e.velocityY > 800) ? CLOSED : OPEN;
    y.value = reduce ? target : withSpring(target, { velocity: e.velocityY, damping: 18 });
  });

const style = useAnimatedStyle(() => ({ transform: [{ translateY: y.value }] }));
```

Open programmatically: `y.value = withSpring(OPEN)`. Prefer a vetted library (`@gorhom/bottom-sheet`) for full a11y/keyboard handling in production; this shows the core mechanism.

---

## 3. Animated tab-bar indicator

Slide an underline to the active tab; width/x come from measured layout (store in shared values, animate transforms).

```tsx
const indicatorX = useSharedValue(0);
const reduce = useReducedMotion();
function onSelect(index: number, layout: { x: number }) {
  indicatorX.value = reduce ? layout.x : withSpring(layout.x, { damping: 20, stiffness: 200 });
}
const style = useAnimatedStyle(() => ({ transform: [{ translateX: indicatorX.value }] }));
```

Capture each tab's `x` via `onLayout`; never animate `left`/`width` directly — translate a fixed-width indicator.

---

## 4. Screen transition

Navigation owns screen transitions — configure them on the route, don't reimplement with Reanimated. Expo Router (native-stack):

```tsx
// app/_layout.tsx
import { Stack } from "expo-router";
export default function Layout() {
  return (
    <Stack screenOptions={{ animation: "slide_from_right", gestureEnabled: true }}>
      <Stack.Screen name="details" options={{ presentation: "modal", animation: "slide_from_bottom" }} />
    </Stack>
  );
}
```

For in-screen content motion on focus, animate with Reanimated inside the screen and **cancel/cleanup on blur/unmount** (see [expo-router-transitions](./expo-router-transitions.md)). Reduced motion: set `animation: "fade"` or `"none"` when the user prefers reduced motion.

---

## 5. Collapsing scroll header

Drive header height/opacity from scroll offset on the UI thread.

```tsx
import Animated, { useScrollOffset, useAnimatedRef, useAnimatedStyle, interpolate, Extrapolation } from "react-native-reanimated";

const ref = useAnimatedRef<Animated.ScrollView>();
const offset = useScrollOffset(ref);
const header = useAnimatedStyle(() => ({
  transform: [{ translateY: interpolate(offset.value, [0, 120], [0, -60], Extrapolation.CLAMP) }],
  opacity: interpolate(offset.value, [0, 120], [1, 0], Extrapolation.CLAMP),
}));
<>
  <Animated.View style={[styles.header, header]} />
  <Animated.ScrollView ref={ref} scrollEventThrottle={16}>{/* content */}</Animated.ScrollView>
</>;
```

Reduced motion: keep the header static (skip the interpolation) — content still scrolls.

---

## 6. FlatList item enter/exit + reorder

Layout animations handle mount/unmount/reorder declaratively. Honor reduced motion via `.reduceMotion`.

```tsx
import Animated, { FadeInDown, FadeOut, LinearTransition, ReduceMotion } from "react-native-reanimated";

<Animated.FlatList
  data={items}
  itemLayoutAnimation={LinearTransition}            // animates reorder/add/remove
  keyExtractor={(it) => it.id}
  renderItem={({ item }) => (
    <Animated.View
      entering={FadeInDown.duration(220).reduceMotion(ReduceMotion.System)}
      exiting={FadeOut.reduceMotion(ReduceMotion.System)}>
      {/* row */}
    </Animated.View>
  )}
/>;
```

Stable `keyExtractor` is required for correct enter/exit/reorder. Layout animations need the New Architecture.

---

## 7. Pull-to-refresh

Custom indicator driven by overscroll (use the platform `RefreshControl` for the actual refresh trigger; animate a custom visual from scroll offset).

```tsx
const offset = useScrollOffset(ref);
const spinner = useAnimatedStyle(() => {
  const pull = Math.min(Math.max(-offset.value, 0), 80);   // overscroll at top
  return { opacity: pull / 80, transform: [{ scale: 0.6 + (pull / 80) * 0.4 }, { rotate: `${pull * 4}deg` }] };
});
```

Pair with `<RefreshControl onRefresh={...} />` for the data fetch; the Reanimated part is purely visual. Reduced motion: show a static indicator (no rotate/scale).

---

## 8. Skia animated loader

Custom canvas animation — shared values pass directly into Skia props. Use `useClock`/`useDerivedValue`; for color use Skia's own interpolation.

```tsx
import { Canvas, Circle, Group } from "@shopify/react-native-skia";
import { useClock, useDerivedValue } from "@shopify/react-native-skia";

function Loader() {
  const clock = useClock();                          // ms since mount, frame-driven
  const rotation = useDerivedValue(() => (clock.value / 1000) % (Math.PI * 2));
  const transform = useDerivedValue(() => [{ rotate: rotation.value }]);
  return (
    <Canvas style={{ width: 64, height: 64 }} accessibilityLabel="Loading">
      <Group origin={{ x: 32, y: 32 }} transform={transform}>
        <Circle cx={32} cy={8} r={5} color="#6cf" />
      </Group>
    </Canvas>
  );
}
```

Reduced motion: render a static frame (e.g. a non-animated spinner or a text status) when `useReducedMotion()` is true — a perpetually spinning loader is exactly what reduced-motion users want suppressed. Wrap the opaque canvas with an accessible label/role. See [skia](./skia.md) for memory + shader details.

## Related references

- [Reanimated core](./reanimated-core.md) · [Worklets & threading](./worklets-threading.md) · [Gestures](./gestures.md) · [Layout animations](./layout-animations.md) · [Scroll](./scroll.md) · [Expo Router transitions](./expo-router-transitions.md) · [Skia](./skia.md) · [Accessibility & performance](./accessibility-performance.md)
