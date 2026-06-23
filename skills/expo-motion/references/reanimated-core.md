# Reanimated 4 Core

The core mental model and API for `react-native-reanimated` 4.x in Expo (SDK 56+) / React Native motion. Reanimated 4 **requires the New Architecture (Fabric, RN 0.76+)**; everything below assumes it. Worklets ship in a separate package, `react-native-worklets` — threading mechanics live in the sibling reference, not here.

## Contents

- [Mental model](#mental-model)
- [Setup and assumptions](#setup-and-assumptions)
- [Shared values: `useSharedValue`](#shared-values-usesharedvalue)
- [Animated styles: `useAnimatedStyle`](#animated-styles-useanimatedstyle)
- [Animated props: `useAnimatedProps`](#animated-props-useanimatedprops)
- [Animation builders](#animation-builders)
- [Derived values: `useDerivedValue`](#derived-values-usederivedvalue)
- [Reactions: `useAnimatedReaction`](#reactions-useanimatedreaction)
- [Cleanup: `cancelAnimation`](#cleanup-cancelanimation)
- [Interpolation, color, easing](#interpolation-color-easing)
- [CSS-style API (declarative)](#css-style-api-declarative)
- [Choosing an approach](#choosing-an-approach)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## Mental model

Reanimated runs animations on the UI thread so they never stall when JavaScript is busy. Three roles keep that clean:

- **Shared values** live on the UI thread — the single source of truth for *transient* motion (an offset, a progress value, a drag delta).
- **Styles and props derive from shared values** inside worklet callbacks (`useAnimatedStyle`, `useAnimatedProps`). These re-run on the UI thread when a shared value they read changes, applying the result to the native view with no React re-render.
- **Product state stays in React / your store.** What the app cares about (is the sheet open, which item is selected, the persisted count) belongs in `useState`/reducer/store — not in a shared value.

Shared values are fast per-frame but slow and stale to read from the JS thread. Keep "what the user sees moving" in shared values and "what the app is" in React. When motion completes and a durable outcome results (a dismissal, a committed value), push it back to React from the animation callback via the threading bridge — see [Worklets & threading](./worklets-threading.md).

## Setup and assumptions

Everything imports from `react-native-reanimated`:

```tsx
import Animated, {
  useSharedValue, useAnimatedStyle, useAnimatedProps,
  useDerivedValue, useAnimatedReaction, cancelAnimation,
  withTiming, withSpring, withDecay, withSequence, withRepeat, withDelay,
  interpolate, interpolateColor, Extrapolation, Easing,
} from 'react-native-reanimated';
```

You do **not** hand-write `'worklet';` on callbacks passed to Reanimated hooks (`useAnimatedStyle`, `useAnimatedProps`, `useDerivedValue`, `useAnimatedReaction`, gesture/animation callbacks) — the Babel plugin auto-workletizes them. The directive is only for standalone helper functions you author and want to run on the UI thread.

## Shared values: `useSharedValue`

A shared value is a mutable container readable and writable from both threads. Read and write `.value` (or use `.get()` / `.set()` for React Compiler compatibility) — never during render.

```tsx
function MoveBox() {
  const offset = useSharedValue(0);

  const animatedStyle = useAnimatedStyle(() => ({
    transform: [{ translateX: offset.value }],
  }));

  return (
    <Animated.View style={[styles.box, animatedStyle]}>
      <Button title="Move" onPress={() => { offset.value = withSpring(200); }} />
    </Animated.View>
  );
}
```

Rules that bite if ignored:

- **Never destructure** (`const { value } = sv`) — it severs reactivity.
- For objects, **reassign the whole value** (`sv.value = { ...sv.value, x: 50 }`); mutating a field in place loses reactivity. For large arrays/objects, mutate in place with `sv.modify(arr => { arr.push(item); return arr; })`.
- **Do not read or write `.value` during component render.** Reanimated warns "Reading from `value` during component render" / "Writing to `value` during component render". Touch shared values only inside worklet callbacks, event handlers, or effects.
- If a value is only ever needed on the JS thread, use `useState` instead — reading a shared value across threads is slow and can return a stale value.

## Animated styles: `useAnimatedStyle`

Returns a style object computed from shared values on the UI thread. Apply it in the style array; it overrides matching static styles.

```tsx
const progress = useSharedValue(0);

const animatedStyle = useAnimatedStyle(() => ({
  opacity: progress.value,
  transform: [{ scale: 0.8 + progress.value * 0.2 }],
}));

<Animated.View style={[styles.card, animatedStyle]} />
```

- Keep static styles in `StyleSheet.create()`; put only dynamic parts in the updater.
- Removing an animated property does not reset it — set it explicitly to `undefined` to clear.
- **Never mutate a shared value inside the updater** (e.g. `offset.value = withTiming(1)`). It loops infinitely. Mutate in handlers/effects; only read here.

## Animated props: `useAnimatedProps`

For animating component *props* rather than styles — SVG geometry, `TextInput` text, etc. Do conversions inside the callback.

```tsx
import { Circle } from 'react-native-svg';
const AnimatedCircle = Animated.createAnimatedComponent(Circle);

const r = useSharedValue(10);
const animatedProps = useAnimatedProps(() => ({
  r: r.value,
  cx: 50,
}));

<AnimatedCircle animatedProps={animatedProps} cy={50} fill="tomato" />
```

To animate text without re-rendering React every frame, drive an animated `TextInput`'s `text` prop:

```tsx
const AnimatedTextInput = Animated.createAnimatedComponent(TextInput);

const animatedProps = useAnimatedProps(() => ({
  text: String(Math.round(progress.value)),
  defaultValue: '0',
}));

<AnimatedTextInput animatedProps={animatedProps} editable={false} />
```

Custom color props need manual `processColor()` wrapping inside the callback.

Built-in animated components: `Animated.View`, `Animated.Text`, `Animated.Image`, `Animated.ScrollView`, `Animated.FlatList`. Wrap anything else with `Animated.createAnimatedComponent`.

## Animation builders

Assign a builder to `sv.value` and Reanimated animates from the current value to the target on the UI thread. They compose by nesting.

### `withTiming`

Duration-based tween with an easing curve.

```tsx
opacity.value = withTiming(1, { duration: 300, easing: Easing.out(Easing.cubic) });
```

### `withSpring`

Physics. Two config modes that **cannot be mixed** (if both appear, duration-based wins):

- Physics-based: `stiffness`, `damping`, `mass`.
- Duration-based: `duration`, `dampingRatio`. `dampingRatio < 1` is bouncy, `1` settles fastest with no bounce, `> 1` is slow with no overshoot.

```tsx
scale.value = withSpring(1, { dampingRatio: 0.6, duration: 500 });
```

### `withDecay`

Momentum that decelerates from an initial velocity (typically a fling). When `rubberBandEffect: true`, `clamp` is **required** and the value bounces at the clamp bounds.

```tsx
offset.value = withDecay({ velocity: e.velocityX, clamp: [0, width], rubberBandEffect: true });
```

### `withSequence`

Runs builders one after another.

```tsx
x.value = withSequence(withTiming(50, { duration: 200 }), withSpring(0), withDelay(300, withTiming(100)));
```

### `withRepeat`

Repeats an animation, e.g. `rotation.value = withRepeat(withTiming(360, { duration: 1000 }), -1)`. Count `0` or `-1` repeats forever (until cancelled/unmounted). `reverse: true` ping-pongs — but only wraps *animation functions* (`withTiming`/`withSpring`), not modifiers like `withSequence`.

### `withDelay`

Delays any builder — handy for staggered entrances, e.g. `sv[i].value = withDelay(i * 80, withSpring(1))` in a loop.

### Completion callbacks

`withTiming`/`withSpring`/`withDecay`/`withRepeat` accept a callback `(finished, current)`, auto-workletized on the UI thread. `finished` is `true` on natural completion, `false` if cancelled (e.g. interrupted by a new animation). To run JS work on completion, bridge from here — see [Worklets & threading](./worklets-threading.md).

```tsx
opacity.value = withTiming(0, { duration: 200 }, (finished) => {
  'worklet';
  if (finished) { /* schedule JS-side cleanup via the threading bridge */ }
});
```

## Derived values: `useDerivedValue`

A read-only shared value that recomputes on the UI thread whenever its dependencies change. Use it to compute one or many values from a single source.

```tsx
const scroll = useSharedValue(0);
const headerOpacity = useDerivedValue(() => interpolate(scroll.value, [0, 120], [1, 0], Extrapolation.CLAMP));
```

If you need the previous value, use `useAnimatedReaction` instead.

## Reactions: `useAnimatedReaction`

Watches a prepared value and runs a reaction when it changes, giving you `(current, previous)`. Use it for side effects driven by shared-value changes — including bridging back to JS when a threshold is crossed.

```tsx
useAnimatedReaction(
  () => progress.value,
  (current, previous) => {
    if (previous !== null && current >= 1 && previous < 1) {
      // completed crossing — bridge to JS here (see worklets-threading.md)
    }
  },
);
```

## Cleanup: `cancelAnimation`

Stops an in-flight animation on a shared value. **Always cancel infinite or long-running shared-value animations on unmount** to avoid leaks and post-unmount warnings.

```tsx
useEffect(() => {
  rotation.value = withRepeat(withTiming(360, { duration: 1000 }), -1);
  return () => { cancelAnimation(rotation); };
}, []);
```

Never start an infinite animation in module scope or a global timer — it cannot be cleaned up. CSS animations (below) self-clean on unmount and do not need this.

## Interpolation, color, easing

`interpolate` maps an input range to an output range inside a worklet. Control out-of-range behavior with `Extrapolation` (`CLAMP`, `EXTEND`, `IDENTITY`).

```tsx
const style = useAnimatedStyle(() => ({
  opacity: interpolate(scroll.value, [0, 200], [1, 0], Extrapolation.CLAMP),
  transform: [{ translateY: interpolate(scroll.value, [0, 200], [0, -50]) }],
  // interpolateColor crosses color stops (hex/rgb/hsl):
  backgroundColor: interpolateColor(scroll.value, [0, 200], ['#fff', '#3b82f6']),
}));
```

Common easings: `Easing.linear`, `Easing.ease`, `Easing.in/out/inOut(fn)`, `Easing.cubic`, `Easing.bezier(x1, y1, x2, y2)`. Match to intent — `Easing.out(Easing.cubic)` for entrances, `Easing.inOut` for symmetric transitions.

## CSS-style API (declarative)

Reanimated 4 ships a CSS-style animation/transition API on `Animated.*` components. It is the **simpler default** for state-driven motion: declarative, no worklets, no shared values, and less to get wrong. Reach for shared values only when motion is gesture-driven, needs per-frame math, or reads layout each frame.

### CSS transitions

Animate a style smoothly whenever a state-driven value changes. Set the property normally, then declare what/how-long/how to transition.

```tsx
<Animated.View
  style={{
    width: isExpanded ? 200 : 100,
    transitionProperty: 'width',
    transitionDuration: 300,
    transitionTimingFunction: 'ease-out',
  }}
/>
```

Arrays target multiple properties (order must match across all three):

```tsx
transitionProperty: ['transform', 'opacity', 'backgroundColor'],
transitionDuration: [300, 200, 150],
transitionTimingFunction: ['ease-out', 'linear', 'ease-in-out'],
```

This is also the right tool for simple press/toggle feedback: pair `Pressable` + React state (`onPressIn`/`onPressOut` toggling a `pressed` flag) with a `transform`/`transitionProperty` transition, and skip worklets entirely.

Notes:

- Avoid `transitionProperty: 'all'` — it evaluates every style property each frame.
- Discrete props (`flexDirection`, `justifyContent`, `alignItems`) flip instantly; add `transitionBehavior: 'allow-discrete'` to flip at the midpoint (`display` flips at start). For smoother discrete changes, use [Layout animations](./layout-animations.md).
- Negative delays start the transition partway through.

### CSS animations

Use a keyframe sequence independent of external state — loaders, pulses, entrance choreography.

```tsx
const pulse = { '0%': { opacity: 1 }, '50%': { opacity: 0.4 }, '100%': { opacity: 1 } };

<Animated.View
  style={{
    animationName: pulse,
    animationDuration: '1200ms',
    animationIterationCount: 'infinite',
    animationTimingFunction: 'ease-in-out',
  }}
/>
```

Notes:

- The element's current state is the implicit `0%`; define only the frames that differ (at least one keyframe required).
- The timing function on the final keyframe is ignored (nothing follows it).
- All entries in a `transform` array must appear in the same order across every keyframe.
- `animationIterationCount: 'infinite'` self-cleans on unmount — no `cancelAnimation` needed. Pause/resume with `animationPlayState: 'paused' | 'running'`.
- Avoid `animationFillMode: 'forwards' | 'both'` with fractional iteration counts and relative units — a later parent resize can strand stale dimensions.

## Choosing an approach

- **State-driven A→B** → CSS transition.
- **State-driven keyframe loop / choreography** → CSS animation.
- **Press/toggle feedback** → CSS transition + `Pressable` + state.
- **Gesture tracking, per-frame math/trig, layout reads, or fanning many values from one source** → shared values + `useAnimatedStyle`.

## Pitfalls / Do-not

- **Do not read or write `sharedValue.value` during render.** It is slow, returns stale data, and triggers Reanimated warnings. Read only inside worklet callbacks, event handlers, or effects; keep app-visible state in React.
- **Do not animate layout props** (`top`, `left`, `width`, `height`, `margin`, `padding`) — each frame forces a layout pass and janks. Animate `transform` (`translateX/Y`, `scale`, `rotate`), `opacity`, and `backgroundColor` instead; use `scale` to fake size changes.
- **Do not skip cleanup.** Cancel infinite/long-running shared-value animations in a `useEffect` return with `cancelAnimation`. Never start them in module scope or global timers. (CSS animations self-clean.)
- **Do not mutate a shared value inside `useAnimatedStyle`/`useAnimatedProps`** — it loops infinitely. Do not destructure shared values or mutate object/array fields in place without `.modify()`.
- **Do not assume Reanimated 3 advice applies.** Reanimated 4 requires the New Architecture; the old-architecture path is unmaintained and its threading/setup guidance differs.

## Related references

- [Worklets & threading](./worklets-threading.md)
- [Gestures](./gestures.md)
- [Layout animations](./layout-animations.md)
- [Accessibility & performance](./accessibility-performance.md)
- [Recipes](./recipes.md)
