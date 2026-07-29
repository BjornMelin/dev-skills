# React Native Animated Source Notes

Use this only when the task is deciding whether to keep React Native
`Animated`/`LayoutAnimation` or migrate a native app to Reanimated.

## When RN Animated Is Enough

- Isolated opacity/transform changes can stay on RN Animated when the existing
  code already uses it cleanly and does not need shared values, Worklets,
  layout reads, gesture composition, or UI-runtime callbacks.
- Prefer `useNativeDriver: true` for supported props. React Native serializes
  the animation to native before it starts so it can continue without crossing
  the JS bridge every frame.
- Keep all animations using the same `Animated.Value` on the same driver.
  Mixing native and JS drivers on one value is not valid.
- Use RN Animated sparingly for list/scroll work. Direct scroll events can use
  the native driver, but bubbling events such as `PanResponder` cannot.

## When To Migrate To Reanimated

- Gesture-driven motion, scroll-linked transforms, interruption math, layout
  measurement, frame callbacks, and cross-runtime callbacks are better handled
  with Reanimated shared values and Worklets.
- Reanimated is the default for new Expo SDK 56 product motion when the repo is
  already using it or the feature needs UI-runtime execution.
- Do not migrate unrelated RN Animated code just because a file is touched.
  Upgrade only the motion path needed for the requested behavior.

## LayoutAnimation Boundary

`LayoutAnimation` can be useful for broad layout transactions such as a simple
expand/collapse that affects parent layout. It gives less control than
Reanimated layout animations and needs Android setup in older RN contexts, so
use it only when that global transaction behavior is the intended effect.

## Source Anchors

- React Native Animations guide:
  https://reactnative.dev/docs/animations
- React Native Animated API:
  https://reactnative.dev/docs/animated
- React Native Animated source:
  https://github.com/facebook/react-native/blob/main/packages/react-native/Libraries/Animated/AnimatedImplementation.js
