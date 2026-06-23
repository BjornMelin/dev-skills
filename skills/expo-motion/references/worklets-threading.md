# Worklets & Threading

Reanimated 4 splits your app across two JavaScript runtimes. Understanding which
runtime a function runs on — and how to move work and data between them — is the
single most important mental model for writing correct, jank-free motion in
Expo/React Native on the New Architecture. This reference covers the runtime
split, the `'worklet'` directive and auto-workletization, the
`react-native-worklets` package and its Babel plugin, the current cross-runtime
scheduling API, and the rules for using shared values safely.

## Two runtimes

A Reanimated app runs two JavaScript runtimes side by side:

- **The RN (JS) runtime** — your normal React world. Component render, hooks,
  React state, navigation, network calls, analytics, timers, and ordinary JS
  objects all live here. This is where the app boots and where business logic
  belongs.
- **The UI runtime** — a separate JS runtime pinned to the native UI thread.
  Worklets run here. Animations, gesture math, scroll handlers, and animated
  style computation all execute on this runtime so they can update the screen in
  sync with the render loop, independent of how busy the JS thread is.

Keeping per-frame work on the UI runtime is why Reanimated animations stay smooth
even when the RN runtime is blocked. The cost is discipline: code on one runtime
cannot freely touch the other. Crossing the boundary is explicit and has rules.

## Worklets and the `'worklet'` directive

A worklet is a JavaScript function that can be serialized and run on the UI
runtime. You mark one explicitly with the `'worklet'` directive as the first
statement in the function body:

```tsx
function clampToScreen(x: number, width: number): number {
  'worklet';
  return Math.max(0, Math.min(x, width));
}
```

A worklet can only close over serializable values and call other worklets. It can
read and write shared values, call animation builders (`withTiming`,
`withSpring`), and call other worklet helpers. It cannot synchronously call an
ordinary RN-runtime function, touch React state, or use APIs that only exist on
the JS runtime.

### Auto-workletization

You rarely write `'worklet'` by hand for the common cases. The Babel plugin
**automatically workletizes** the function bodies it knows run on the UI runtime:

- `useAnimatedStyle`, `useDerivedValue`, `useAnimatedScrollHandler`, and
  `useAnimatedReaction` callbacks.
- Gesture callbacks on the Gesture Handler 2 `Gesture` API (`.onUpdate`,
  `.onEnd`, `.onBegin`, etc.).
- Layout/entering/exiting animation builders.

```tsx
const animatedStyle = useAnimatedStyle(() => {
  // Auto-workletized: no 'worklet' directive needed here.
  return { transform: [{ translateX: offset.value }] };
});

const pan = Gesture.Pan().onUpdate((e) => {
  // Also auto-workletized.
  offset.value = e.translationX;
});
```

Add the directive yourself only to standalone helper functions you intend to call
from inside a worklet, since the plugin cannot infer those from context.

## `react-native-worklets` and the Babel plugin

In Reanimated 4 the worklet machinery lives in a separate package,
`react-native-worklets`, and is a required peer of `react-native-reanimated`.
The threading helpers (`scheduleOnRN`, `scheduleOnUI`, `runOnUIAsync`, and
friends) are imported from `react-native-worklets`, not from
`react-native-reanimated`.

Workletization is performed by the Babel plugin `react-native-worklets/plugin`.

- **On Expo SDK 50+**, `babel-preset-expo` includes the Worklets Babel plugin
  automatically when Reanimated is installed. You do not add it manually.
- **On bare React Native CLI apps**, you must add `react-native-worklets/plugin`
  to `babel.config.js` yourself, and it **MUST be listed last**:

```js
// babel.config.js (RN CLI only — Expo SDK 50+ handles this for you)
module.exports = {
  presets: ['module:@react-native/babel-preset'],
  plugins: [
    // ...all other plugins first...
    'react-native-worklets/plugin', // MUST be the last plugin
  ],
};
```

If the plugin is missing or not last, workletization silently breaks: gesture and
`useAnimatedStyle` bodies fail to run on the UI runtime, and you get cryptic
errors or animations that simply never update.

> Migration note: the old `react-native-reanimated/plugin` path is replaced by
> `react-native-worklets/plugin` in Reanimated 4. Update any leftover reference.

## Crossing runtimes

Use the current `react-native-worklets` threading API. Arguments are passed
**directly** as trailing arguments — there is no currying.

### UI runtime to RN runtime: `scheduleOnRN`

When a worklet needs to run JS-runtime code (set React state, navigate, fire
analytics), schedule it with `scheduleOnRN(fn, ...args)`. The function must be
defined on the RN runtime (component or module scope), never inside a worklet.

```tsx
import { scheduleOnRN } from 'react-native-worklets';
import { Gesture } from 'react-native-gesture-handler';

function MyComponent() {
  const [committed, setCommitted] = useState(false);

  const onCommit = useCallback((didSnap: boolean) => {
    // Runs on the RN runtime — safe to touch React state.
    setCommitted(didSnap);
  }, []);

  const pan = Gesture.Pan()
    .onUpdate((e) => {
      offset.value = e.translationX; // stays on UI runtime
    })
    .onEnd((e) => {
      const snapped = Math.abs(e.translationX) > 120;
      offset.value = withSpring(snapped ? 240 : 0);
      // Cross to RN runtime exactly once, at the boundary of the interaction.
      scheduleOnRN(onCommit, snapped);
    });

  // ...
}
```

### RN runtime to UI runtime: `scheduleOnUI` and `runOnUIAsync`

To run a worklet from the RN runtime (e.g. kick off an animation imperatively in
an effect), use `scheduleOnUI(workletFn, ...args)`. When you need the result back
on the RN runtime, `runOnUIAsync(workletFn, ...args)` returns a Promise that
resolves with the worklet's return value.

```tsx
import { scheduleOnUI, runOnUIAsync } from 'react-native-worklets';

useEffect(() => {
  // Fire-and-forget: run a worklet on the UI runtime.
  scheduleOnUI(() => {
    'worklet';
    progress.value = withTiming(1, { duration: 400 });
  });
}, []);

async function measure() {
  const snapshot = await runOnUIAsync(() => {
    'worklet';
    return offset.value; // computed on the UI runtime, awaited on JS
  });
  console.log('current offset', snapshot);
}
```

### Deprecated: `runOnJS` / `runOnUI`

`runOnJS` and `runOnUI` are **deprecated in Reanimated 4**. They still work
(re-exported for compatibility) but are slated for removal. Migrate them:

- `runOnJS(fn)(...args)` becomes `scheduleOnRN(fn, ...args)` — note the loss of
  currying; arguments move into the single call.
- `runOnUI(fn)(...args)` becomes `scheduleOnUI(fn, ...args)`.
- `runOnRuntime` / `executeOnUIRuntimeSync` become `scheduleOnRuntime` /
  `runOnUISync`.

Treat existing `runOnJS` in Reanimated 3 code as a migration flag, not a runtime
bug — but write all new Reanimated 4 code against `scheduleOnRN`.

## Shared value rules

A `useSharedValue` is the shared mutable cell that both runtimes can see, but the
access rules differ by side:

- **On the UI runtime**, read and write `.value` freely — inside worklets,
  `useAnimatedStyle`, `useDerivedValue`, and gesture/scroll callbacks. This is
  the intended consumption path.
- **On the RN runtime**, avoid reading `.value`. Reading it during render is
  wrong (the value is a moving target and the read can be stale or block), and
  reading it on JS hot paths can stall. Writing `.value` from JS to start an
  animation is fine.

Derive and consume on the UI runtime instead of pulling values across:

```tsx
const offset = useSharedValue(0);

// Derive on the UI runtime — never read offset.value during render.
const opacity = useDerivedValue(() => {
  return interpolate(offset.value, [0, 240], [1, 0.4]);
});

const style = useAnimatedStyle(() => ({
  opacity: opacity.value,
  transform: [{ translateX: offset.value }],
}));
```

If the RN runtime genuinely needs the latest value (e.g. to persist on commit),
push it across the boundary with `scheduleOnRN` from a worklet, or read it once
with `runOnUIAsync` — do not poll `.value` from JS.

### Avoid bridge-crossing in high-frequency callbacks

Per-frame and per-gesture callbacks (`onUpdate`, scroll handlers, derived values)
fire up to 60–120 times per second. Crossing to the RN runtime on every tick
serializes work, floods the JS thread, and reintroduces exactly the jank
Reanimated exists to prevent. Keep these callbacks pure UI-runtime worklet code.
Cross runtimes only at interaction boundaries — gesture `onEnd`, a debounced
commit, or a one-shot animation completion callback — not inside the hot path.

```tsx
// Good: state crosses once, at the end of the gesture.
const pan = Gesture.Pan()
  .onUpdate((e) => {
    offset.value = e.translationX; // UI runtime only
  })
  .onEnd(() => {
    scheduleOnRN(onSettled, offset.value); // single crossing
  });
```

## Pitfalls / Do-not

- **Do not call `runOnJS`/`scheduleOnRN` inside a 60fps gesture handler.** Calling
  it from `onUpdate` or a scroll handler on every frame serializes work to the JS
  thread and causes dropped frames. Cross at `onEnd` or other boundaries only.
- **Do not forget the `'worklet'` directive** on standalone helpers you call from
  inside worklets. Only context-known callbacks (animated styles, gesture
  callbacks) are auto-workletized; everything else needs the directive or it runs
  on the wrong runtime.
- **Do not read `sharedValue.value` on the JS thread or during render.** It can be
  stale, can block, and breaks the model. Derive with `useDerivedValue` and
  consume on the UI runtime; pull across explicitly with `runOnUIAsync` if needed.
- **Do not let the Babel plugin fall out of last place.** `react-native-worklets/plugin`
  must be the final entry in `babel.config.js` on RN CLI apps. Out of order or
  missing, workletization silently fails. (Expo SDK 50+ wires this for you — do
  not add it twice.)

## Related references

- [Reanimated core](./reanimated-core.md)
- [Gestures](./gestures.md)
- [Validation](./validation.md)
