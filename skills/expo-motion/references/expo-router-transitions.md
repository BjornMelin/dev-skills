# Expo Router & screen transitions

Navigation transitions in Expo (SDK 56+) are a platform contract, not a place
for ad-hoc product animation. Expo Router renders a **native stack** powered by
`react-native-screens`, so route push/pop, modal presentation, back gestures,
and header transitions are driven by UIKit/Android navigators on the New
Architecture. This reference covers what navigation owns, how to configure it
through screen options, when to stay out of its way with Reanimated 4, how to
clean up route-scoped animations, and how `@expo/ui` native controls fit as
leaf nodes. For shared-value mechanics and threading, see
[Reanimated core](./reanimated-core.md); for content enter/exit, see
[Layout animations](./layout-animations.md).

## react-native-screens is the engine

Expo Router `Stack` is a thin layout over React Navigation's native stack, which
delegates route containers and gestures to `react-native-screens`. Treat route
movement as native-screens behavior unless the app explicitly opts into a custom
navigator.

- Confirm `react-native-screens` is installed at the Expo-compatible version
  before changing stack assumptions (use `expo install --check`).
- The native stack owns the **full screen shell**: push/pop, modal dismissal,
  edge-swipe back, hardware back. Do not replace those gestures with a
  full-screen Reanimated gesture just to customize back navigation.
- Reanimated may own content **after** the route is mounted and the transition
  has settled (row enter/exit, inline expansion). It should never animate the
  route shell itself.

## Configuring transitions with screen options

Set transition behavior through `Stack.Screen options`, centralized where route
ownership lives — in the `_layout.tsx`, not scattered across leaf content.

```tsx
// app/_layout.tsx
import { Stack } from 'expo-router';

export default function RootLayout() {
  return (
    <Stack screenOptions={{ headerShown: true }}>
      <Stack.Screen name="index" options={{ title: 'Home' }} />
      <Stack.Screen
        name="details"
        options={{ animation: 'slide_from_right', gestureEnabled: true }}
      />
      <Stack.Screen
        name="compose"
        options={{ presentation: 'modal', animation: 'slide_from_bottom' }}
      />
    </Stack>
  );
}
```

### `animation`

Controls native-stack screen movement. Common values: `default`, `fade`,
`fade_from_bottom`, `fade_from_right`, `reveal_from_bottom`,
`scale_from_center`, `slide_from_right`, `slide_from_left`,
`slide_from_bottom`, `flip`, `simple_push`, and `none`.

- Use `animation: 'none'` to disable transitions. Treat the older
  `animationEnabled: false` as stale unless installed types prove otherwise.
- Some native transitions and presentations **do not allow custom durations**.
  Do not promise exact timing parity across platforms.

### `presentation`

Route semantics, not just a look: `card` (default push), `modal`,
`transparentModal`, `containedModal`, `fullScreenModal`, `formSheet`, plus
related platform presentations. iOS modal/sheet behavior does **not** all exist
on Android — validate both. Sheet detents are configured with native sheet
options and surfaced through events like `sheetDetentChange`.

### `gestureEnabled` and headers

- `gestureEnabled` toggles the interactive back gesture (iOS edge swipe). If a
  screen blocks back navigation, validate hardware back, edge swipe,
  accessibility escape/back, and deep-link restoration.
- Header options (`headerShown`, `title`, `headerTransparent`, large titles)
  drive native header transitions. `Stack.SearchBar` integrates with the native
  header and forces it visible; `Stack.Toolbar` is alpha (iOS SDK 55+, Android
  SDK 56+), renders only on iOS/Android, and needs an app-owned web fallback.
- `Stack.Header`, `Stack.Title`, and related primitives are composition helpers
  that write native-stack options; if multiple instances configure the same
  screen, the last rendered one wins.

## Shared-element / native transitions

Native shared-element and zoom transitions are exposed through
`react-native-screens` / native-stack options rather than a screen-level
Reanimated layout. Prefer the native knob when it exists; reach for the
[Layout animations](./layout-animations.md) `sharedTransitionTag` path only for
content that is not a route-shell transition, and never layer it on top of the
native stack transition for the same screen.

## When navigation owns the lifecycle (don't fight it)

If movement is route-level, let the navigator drive it. Coordinate with — do not
replace — the transition using navigation listeners.

```tsx
import { useNavigation } from 'expo-router';
import { useEffect } from 'react';

function Screen() {
  const navigation = useNavigation();
  useEffect(() => {
    const start = navigation.addListener('transitionStart', () => {});
    const end = navigation.addListener('transitionEnd', () => {
      // run content animation only after the route shell settles
    });
    return () => {
      start();
      end();
    };
  }, [navigation]);
  // ...
}
```

- Listen for `transitionStart`, `transitionEnd`, `gestureCancel`, and
  `sheetDetentChange` for route-lifecycle coordination.
- Keep expensive work out of transition frames; listeners coordinate state, they
  do not run heavy work synchronously.
- Do not use component-local Reanimated to simulate push/pop, modal, form-sheet,
  or native back-gesture behavior.

## Route-change cleanup

Route-scoped Reanimated work must be torn down on unmount, or animations and
gesture/scroll handlers leak across navigations.

```tsx
import { useFocusEffect } from 'expo-router';
import { cancelAnimation, useSharedValue } from 'react-native-reanimated';
import { useCallback } from 'react';

function Row() {
  const progress = useSharedValue(0);
  useFocusEffect(
    useCallback(() => {
      return () => {
        cancelAnimation(progress); // stop in-flight withTiming/withSpring
      };
    }, [progress]),
  );
  // ...
}
```

- Call `cancelAnimation()` on shared values driving in-flight animations.
- Kill scroll handlers and gesture detectors on unmount; do not leave a
  `useAnimatedScrollHandler` or `Gesture` attached to a torn-down screen.
- Watch params/unmount timing: a route can unmount mid-transition. Scope
  `beforeRemove`, transition listeners, and guarded navigation to the affected
  screen, and validate interrupted gestures (partial back-swipe cancel, rapid
  double back, route replacement).

## Expo UI controls as leaf nodes

`@expo/ui` renders real SwiftUI (iOS) and Jetpack Compose (Android) controls.
They are **native leaves**, not arbitrary RN animation surfaces.

```tsx
import { Host, Switch, Slider } from '@expo/ui';

function Settings() {
  return (
    <Host matchContents>
      <Switch value={enabled} onValueChange={setEnabled} />
      <Slider value={level} onValueChange={setLevel} />
    </Host>
  );
}
```

- Every native subtree needs a `Host`. Use one obvious `Host` owner per native
  UI subtree; do not wrap a whole screen when only a leaf control needs it.
- Use universal `@expo/ui` for one tree across iOS/Android/web; drop to
  `@expo/ui/swift-ui` or `@expo/ui/jetpack-compose` (with their `/modifiers`)
  only when the universal API lacks a control.
- `matchContents` suits intrinsic controls — do **not** use it on the same axis
  as scrollable content; give scrollable hosts finite size.
- Use Reanimated for product content motion, not to clone a Switch/Slider/Picker
  the platform already provides. Use native `BottomSheet` (controlled by
  `isPresented`/`onDismiss`, semantic `snapPoints`) before a custom sheet.

## Pitfalls / Do-not

- Do not layer a screen-level Reanimated spatial transition on top of a native
  stack transition for the same screen — they fight.
- Do not reimplement push/pop, modal, or back gesture with Reanimated when a
  native-stack option exists.
- Do not assume iOS modal/sheet presentation behavior exists on Android.
- Do not promise custom durations on native transitions that disallow them.
- Do not scatter screen options across leaf content; centralize in the layout.
- Do not leave shared values, scroll handlers, or gestures running after a route
  unmounts — `cancelAnimation` and detach them.
- Do not treat `@expo/ui` controls as `Animated.View`; wrapping them in
  gesture/animation surfaces breaks native accessibility.

## Related references

- [Reanimated core](./reanimated-core.md) — shared values, worklets, threading,
  `cancelAnimation`.
- [Layout animations](./layout-animations.md) — entering/exiting, `Layout`, and
  shared-transition content motion.
- [NativeWind & styling boundaries](./nativewind-styling.md) — class vs
  Reanimated ownership for styling.
- [Recipes](./recipes.md) — end-to-end navigation + content motion patterns.
