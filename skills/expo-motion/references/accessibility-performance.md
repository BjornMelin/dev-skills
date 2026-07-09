# Accessibility & Performance

Motion is a feature, not decoration. On Expo and React Native (Reanimated 4 on the
New Architecture), the same animation that delights most users can disorient,
nauseate, or simply drop frames for others. This reference covers two tightly
coupled concerns: honoring the system reduced-motion preference (a WCAG
accessibility requirement, not a nicety) and keeping animations cheap enough to
hold a 60/120fps frame budget on real devices.

The throughline: reduced motion must *reduce vestibular movement while preserving
functional feedback*, and smooth motion must *stay on the UI thread and off React
state*. For the primitives see [Reanimated core](./reanimated-core.md); for the
threading model that makes UI-thread work fast see [Worklets & threading](./worklets-threading.md).
This file does not re-explain shared values or worklets — it shows how to apply
them accessibly and performantly.

## Reduced motion

Users who enable "Reduce Motion" (iOS) or "Remove animations" (Android) are
telling you that large or unexpected movement causes them discomfort. Respect it.

### Reading the preference

Two complementary APIs:

- `useReducedMotion()` from `react-native-reanimated` — a reactive hook usable
  directly inside components and worklet-adjacent logic. It returns a boolean and
  re-renders when the setting changes.
- `AccessibilityInfo.isReduceMotionEnabled()` from `react-native` — a one-shot
  promise, plus a `reduceMotionChanged` subscription for live changes. Use this
  for non-render contexts (effects, services, imperative code).

```tsx
import { useReducedMotion } from "react-native-reanimated";
import { AccessibilityInfo } from "react-native";
import { useEffect, useState } from "react";

// Component-level: prefer the Reanimated hook.
function useMotionPreference() {
  return useReducedMotion();
}

// Imperative / effect context: use AccessibilityInfo + listener.
function useReduceMotionListener() {
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    AccessibilityInfo.isReduceMotionEnabled().then(setReduced);
    const sub = AccessibilityInfo.addEventListener(
      "reduceMotionChanged",
      setReduced,
    );
    return () => sub.remove();
  }, []);

  return reduced;
}
```

### Honoring it

There is no single correct response — choose per animation by how much vestibular
movement it produces:

- **Skip the transient and jump to the end state.** A fade-and-slide reveal
  becomes an instant appearance. The element still appears; it just does not
  travel.
- **Set `duration: 0`** (or near-instant timing) so a `withTiming` value lands on
  its target without visible motion.
- **Render a static end-state** for purely decorative loops (skip the rotating
  spinner; show a static indeterminate state or a progress label instead).
- **Layout animations:** apply `.reduceMotion(ReduceMotion.System)` to entering,
  exiting, and layout transitions so Reanimated follows the device setting
  automatically.

```tsx
import Animated, {
  FadeIn,
  withTiming,
  ReduceMotion,
  useSharedValue,
  useAnimatedStyle,
  useReducedMotion,
} from "react-native-reanimated";
import { useEffect } from "react";

// Layout animation that respects the system setting on its own.
function Card() {
  return (
    <Animated.View
      entering={FadeIn.duration(220).reduceMotion(ReduceMotion.System)}
    />
  );
}

// Manual control: collapse the duration when motion is reduced.
function Toast({ visible }: { visible: boolean }) {
  const reduced = useReducedMotion();
  const opacity = useSharedValue(0);

  useEffect(() => {
    opacity.value = withTiming(visible ? 1 : 0, {
      duration: reduced ? 0 : 200,
    });
  }, [visible, reduced, opacity]);

  const style = useAnimatedStyle(() => ({ opacity: opacity.value }));
  return <Animated.View style={style} />;
}
```

### Preserve functional feedback

This is the part that gets dropped. Reduced motion is not "no feedback" — it is
"reach the same communicative end-state without the journey." A user who reduces
motion still needs to know that a button was pressed, a form field is invalid, a
save succeeded, or focus moved. Strip the parallax and the bounce; keep the
pressed color change, the error outline, the success checkmark, and any
screen-reader announcement. Removing all feedback under reduced motion is an
accessibility regression, not a fix.

## Haptics

`expo-haptics` provides tactile confirmation: `impactAsync` (light/medium/heavy
collisions), `notificationAsync` (success/warning/error), and
`selectionAsync` (discrete picker ticks). Treat haptics as *complementary* — they
reinforce a state change that is already visible or announced, never the sole
signal. A haptic with no visible or accessible counterpart is invisible to users
who cannot feel it (and to screen readers).

Fire haptics at the moment the state actually settles: a gesture-end callback or
an animation-complete callback, not mid-animation. Cross thread boundaries
correctly — `expo-haptics` runs on JS, so schedule it from the UI thread (see
[Worklets & threading](./worklets-threading.md)).

```tsx
import * as Haptics from "expo-haptics";
import { Gesture } from "react-native-gesture-handler";
import { scheduleOnRN } from "react-native-worklets";

const fling = Gesture.Fling().onEnd(() => {
  "worklet";
  // Pair the haptic with the gesture resolving — and with a visible state
  // change elsewhere in the UI; the haptic is reinforcement, not the message.
  scheduleOnRN(Haptics.impactAsync, Haptics.ImpactFeedbackStyle.Light);
});
```

For reduced-motion users, haptics can usefully *stand in for* the movement you
removed — but only alongside a visible end-state, never instead of one.

## Performance

Smoothness comes from keeping animation work off the JS thread and off React's
render path.

### Thread classification

Before fixing jank, classify where the work runs:

- **UI thread** — Reanimated worklets, animated styles/props, gesture handling.
  Driven independently of JS; this is where animation *should* live so it keeps
  running even when JS is busy.
- **JS thread** — React renders, effects, business logic, network. Blocking the
  JS thread does not stall a UI-thread animation, but per-frame JS work does.

Also account for layout, image decode, list virtualization, and GPU/canvas load
when diagnosing — a "slow animation" is often really list or layout work nearby.

### Keep transient motion in shared values, not React state

Calling `setState` on every frame is the single most common cause of native
animation jank. Each update triggers a React render, reconciliation, and a JS→UI
round trip — 60+ times per second. Animated values that change every frame belong
in shared values driven on the UI thread; React state is for discrete,
user-visible mode changes (open/closed, selected tab), not for tweening.

```tsx
// Anti-pattern: re-renders every frame, fights the UI thread.
const [x, setX] = useState(0);
useEffect(() => {
  const id = setInterval(() => setX((v) => v + 1), 16);
  return () => clearInterval(id);
}, []);

// Idiomatic: the value lives and animates on the UI thread.
const x = useSharedValue(0);
const style = useAnimatedStyle(() => ({
  transform: [{ translateX: x.value }],
}));
// x.value = withTiming(target) — no React render per frame.
```

### Animate transforms and opacity, not layout props

`transform` (translate/scale/rotate) and `opacity` are composited cheaply and do
not trigger layout. Animating `width`, `height`, `top`, `left`, or `margin`
forces the layout engine to reflow every frame — expensive and janky. To grow a
card, animate `scale`; to move it, animate `translateX/translateY`. Reserve
layout-prop animation for genuine layout transitions (handled by Reanimated's
layout animations), not per-frame tweens.

### Minimize UI-thread work

The UI thread has its own frame budget. Keep worklets lean: precompute constants
in JS, avoid allocating objects or doing heavy math inside `useAnimatedStyle` /
`useDerivedValue` on every frame, and don't read or write more shared values than
necessary. Offloading to the UI thread only helps if the UI-thread work itself
fits the budget.

### Frame budget and instrumentation

A 60fps target gives ~16.6ms per frame; a 120fps (ProMotion) target gives ~8.3ms.
Cross the budget and frames drop. Measure rather than guess:

- **Perf Monitor** — the in-app dev overlay shows JS and UI thread FPS side by
  side, which immediately tells you which thread is the bottleneck.
- **Reanimated devtools / profiling** — surfaces per-frame UI-thread cost and
  long worklets.
- Test in a **release or development build**, not Metro dev mode — dev-mode JS is
  far slower and overstates jank. Validate on representative devices for
  GPU-heavy or large-motion surfaces; simulator-only proof is not enough there.

See [Validation](./validation.md) for the full device-proof and command checklist.

## Motion sickness

Large, sustained, or automatic movement can trigger vestibular discomfort even in
users who have not enabled reduced motion. Big parallax, device-tilt (sensor)
parallax, full-screen zoom or rotation, auto-playing carousels, and shared-element
transitions across large distances deserve stricter scrutiny and stronger device
proof than a small opacity or scale affordance. Prefer shorter travel distances, give users control
over auto-advancing motion, and always provide a reduced-motion path for these
heavy effects. When in doubt, the small affordance is the safer default. See
[Recipes](./recipes.md) for patterns that stay within comfortable limits.

## Pitfalls / Do-not

- **Do not drive per-frame animation with React `setState`.** It re-renders every
  frame and destroys native performance. Use shared values on the UI thread.
- **Do not ignore the reduced-motion setting.** It is a WCAG accessibility
  requirement. Honor `useReducedMotion()` / `AccessibilityInfo` on every
  non-trivial animation.
- **Do not strip functional feedback under reduced motion.** Reduce vestibular
  movement, but keep pressed, focus, progress, success, and error signals.
- **Do not animate layout props (`width`/`height`/`top`/`left`/`margin`)
  per frame.** They force reflow. Animate `transform` and `opacity` instead.
- **Do not treat haptics as a substitute for motion or visible state.** Haptics
  reinforce; they are invisible to anyone who cannot feel them and to screen
  readers.
- **Do not validate jank in Metro dev mode or on simulator alone** for GPU-heavy
  or large-motion surfaces. Use a release/development build on real devices.

## Related references

- [Reanimated core](./reanimated-core.md)
- [Worklets & threading](./worklets-threading.md)
- [Validation](./validation.md)
- [Recipes](./recipes.md)
