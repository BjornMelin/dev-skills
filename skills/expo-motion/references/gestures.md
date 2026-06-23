# Gestures

Touch and pointer interaction in Expo / React Native is built on
`react-native-gesture-handler`'s modern declarative **Gesture API**. You build
gesture objects, attach them to a `GestureDetector`, and let their worklet
callbacks drive Reanimated shared values directly on the UI thread. This page
covers the gesture types, composition, the worklet callback contract, app setup,
and cleanup. It assumes **Reanimated 4 on the New Architecture (Fabric)**.

Keep transient interaction state (translation, scale, velocity) in shared values
on the UI thread, and cross to React/JS only when a durable product state change
needs to happen. See [Worklets & threading](./worklets-threading.md) for how the
two runtimes talk to each other.

## App setup

`GestureDetector` must have a `GestureHandlerRootView` somewhere above it in the
tree. Mount it once at the app root so it wraps every screen.

```tsx
import { GestureHandlerRootView } from 'react-native-gesture-handler';

export default function App() {
  return (
    <GestureHandlerRootView style={{ flex: 1 }}>
      {/* navigation / screens */}
    </GestureHandlerRootView>
  );
}
```

In Expo apps, import `react-native-gesture-handler` at the very top of the entry
file before anything else when not using Expo Router. With Expo Router the root
view is wired through the navigator, but you should still confirm a
`GestureHandlerRootView` is present (Expo Router does this for you in recent
templates).

## The Gesture API

Construct gestures with the `Gesture` factory and attach them with
`GestureDetector`. The detector takes a single `gesture` prop; use composition
(below) to combine more than one.

| Builder               | Use for                                  |
| --------------------- | ---------------------------------------- |
| `Gesture.Pan()`       | drag, swipe, sheets, sliders             |
| `Gesture.Tap()`       | taps, double-tap (`numberOfTaps`)        |
| `Gesture.LongPress()` | press-and-hold, context menus            |
| `Gesture.Pinch()`     | zoom (`scale`, `velocity`, `focalX/Y`)   |
| `Gesture.Rotation()`  | two-finger rotation (`rotation`)         |
| `Gesture.Fling()`     | directional flicks (`direction`)         |

### Worklet callbacks

Callbacks run on the UI thread and are **auto-workletized** by the Gesture
Handler / Reanimated Babel plugin, so you can read and write `.value` inside them
without an explicit `'worklet'` directive. The common lifecycle is:

- `onBegin` — pointer touched down and the gesture activated its begin phase.
- `onStart` — recognition criteria met (the gesture "starts").
- `onUpdate` — fires on every frame while active; carries the live event
  (`translationX`, `scale`, `velocityX`, etc.).
- `onEnd` — pointer lifted; `success` flag tells you whether it ended naturally.
- `onFinalize` — always runs last, whether the gesture succeeded or was cancelled.

To call back into React state, wrap the JS function with `scheduleOnRN` (see
[Worklets & threading](./worklets-threading.md)); never call a plain React
setter directly from a worklet callback.

## Draggable card (Pan)

```tsx
import Animated, {
  useAnimatedStyle,
  useSharedValue,
  withSpring,
} from 'react-native-reanimated';
import { Gesture, GestureDetector } from 'react-native-gesture-handler';

export function DraggableCard() {
  const x = useSharedValue(0);
  const y = useSharedValue(0);
  // Capture the offset at gesture start so drags are cumulative.
  const startX = useSharedValue(0);
  const startY = useSharedValue(0);

  const pan = Gesture.Pan()
    .onBegin(() => {
      startX.value = x.value;
      startY.value = y.value;
    })
    .onUpdate((e) => {
      x.value = startX.value + e.translationX;
      y.value = startY.value + e.translationY;
    })
    .onEnd((e) => {
      // Spring back to origin, carrying the release velocity.
      x.value = withSpring(0, { velocity: e.velocityX });
      y.value = withSpring(0, { velocity: e.velocityY });
    });

  const style = useAnimatedStyle(() => ({
    transform: [{ translateX: x.value }, { translateY: y.value }],
  }));

  return (
    <GestureDetector gesture={pan}>
      <Animated.View style={[styles.card, style]} />
    </GestureDetector>
  );
}
```

## Swipe-to-dismiss (Pan + threshold)

```tsx
import Animated, {
  runOnJS, // alias of scheduleOnRN in older code; prefer scheduleOnRN
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from 'react-native-reanimated';
import { scheduleOnRN } from 'react-native-worklets';
import { Gesture, GestureDetector } from 'react-native-gesture-handler';

const SWIPE_THRESHOLD = 120;

export function SwipeRow({ width, onDismiss }: { width: number; onDismiss: () => void }) {
  const x = useSharedValue(0);

  const pan = Gesture.Pan()
    .activeOffsetX([-12, 12]) // only activate after meaningful horizontal travel
    .onUpdate((e) => {
      x.value = e.translationX;
    })
    .onEnd((e) => {
      const dismissed = Math.abs(e.translationX) > SWIPE_THRESHOLD;
      if (dismissed) {
        x.value = withTiming(Math.sign(e.translationX) * width, { duration: 180 });
        scheduleOnRN(onDismiss); // hop to RN to update product state
      } else {
        x.value = withTiming(0);
      }
    });

  const style = useAnimatedStyle(() => ({
    transform: [{ translateX: x.value }],
    opacity: 1 - Math.min(Math.abs(x.value) / width, 1),
  }));

  return (
    <GestureDetector gesture={pan}>
      <Animated.View style={style}>{/* row content */}</Animated.View>
    </GestureDetector>
  );
}
```

## Composition

Combine gestures instead of nesting detectors:

- `Gesture.Simultaneous(a, b)` — both can be active at once (e.g. pinch + pan + rotate on the same view).
- `Gesture.Race(a, b)` — first to activate wins; the others are cancelled.
- `Gesture.Exclusive(a, b)` — try in priority order (e.g. double-tap before single-tap).

```tsx
const zoom = Gesture.Simultaneous(Gesture.Pinch(), Gesture.Pan(), Gesture.Rotation());
const press = Gesture.Exclusive(Gesture.Tap().numberOfTaps(2), Gesture.Tap());

<GestureDetector gesture={Gesture.Race(zoom, press)}>{/* ... */}</GestureDetector>;
```

Relationships across separate detectors use `.simultaneousWithExternalGesture(ref)`
and `.requireExternalGestureToFail(ref)` — essential for nested scroll plus
draggable rows.

## Cleanup and cancellation

- `GestureDetector` tears down its native handlers on unmount automatically; you
  do not manually remove listeners.
- Shared values are garbage-collected with the component, so an in-flight spring
  on a `withSpring` value simply stops — no leak.
- For programmatic cancel, build with a `.enabled(enabledShared)` shared boolean
  and flip it, or use `.withRef()` plus an imperative method. Avoid leaving an
  animation mid-flight that writes to a value the unmounted view still reads.
- Always handle `onEnd`/`onFinalize` so a cancelled gesture (e.g. parent scroll
  steals it) returns the value to a resting state.

## Reduced motion

Reduced motion does **not** mean disabling gestures — the interaction must still
work. Instead, simplify the *result*: shorten travel, drop parallax/overshoot,
and prefer `withTiming` over bouncy springs. Read the user preference and branch
your `onEnd` config. See
[Accessibility & performance](./accessibility-performance.md).

## Pitfalls / Do-not

- Do **not** call React state setters directly inside a gesture callback — route
  through `scheduleOnRN`. Updating React state on every `onUpdate` frame thrashes
  the JS thread.
- Do not nest a `GestureDetector` inside another to compose — use
  `Gesture.Simultaneous/Race/Exclusive` instead.
- Do not forget `GestureHandlerRootView` at the app root; gestures silently do
  nothing without it.
- Do not hard-code snap distances or swipe thresholds — measure layout (see
  `measure`/`onLayout`) so they adapt to dynamic sizes.
- Do not add an explicit `'worklet'` directive expecting it to fix a missing
  Babel plugin; callbacks are auto-workletized, but the plugin must be installed.
- Do not read `sharedValue.value` from the JS thread on the gesture hot path —
  derive and consume on the UI thread.

## Related references

- [Reanimated core](./reanimated-core.md)
- [Worklets & threading](./worklets-threading.md)
- [Layout animations](./layout-animations.md)
- [Scroll-driven animation](./scroll.md)
- [Accessibility & performance](./accessibility-performance.md)
- [Recipes](./recipes.md)
