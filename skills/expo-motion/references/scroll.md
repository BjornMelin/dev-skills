# Scroll-driven animation

Scroll position is one of the most useful animation drivers on mobile:
collapsing headers, parallax hero images, sticky toolbars, and progress
indicators all read from a single scroll offset. In Reanimated 4 the offset
lives in a shared value on the UI thread, so the animation tracks scrolling
frame-for-frame without crossing to JS. This page covers the two scroll hooks,
the animated scroll components, interpolation patterns, and reduced-motion
handling. Examples assume Reanimated 4 on the New Architecture.

You must scroll with `Animated.ScrollView` / `Animated.FlatList`, not the plain
RN components — the animated wrappers forward the scroll event to the UI thread.

## Reading the scroll offset

### `useScrollOffset` (simple position)

When you only need the current scroll position as a shared value, this is the
least-boilerplate option. It auto-detects horizontal vs. vertical scroll and
works with `ScrollView`, `FlatList`, and `FlashList`.

```tsx
import Animated, {
  interpolate,
  useAnimatedRef,
  useAnimatedStyle,
  useScrollOffset,
} from 'react-native-reanimated';

const ref = useAnimatedRef<Animated.ScrollView>();
const offset = useScrollOffset(ref);

const headerStyle = useAnimatedStyle(() => ({
  opacity: interpolate(offset.value, [0, 100], [1, 0]),
}));

<Animated.ScrollView ref={ref}>{children}</Animated.ScrollView>;
```

> In Reanimated 4 the older name `useScrollViewOffset` is deprecated in favor of
> `useScrollOffset`. Replace existing usages unless you are pinned to an older
> version.

### `useAnimatedScrollHandler` (multiple events + context)

When you need the drag/momentum lifecycle — not just the position — use the
handler. It exposes a shared `context` object across the handlers for the same
scroll view.

```tsx
import Animated, {
  useAnimatedScrollHandler,
  useSharedValue,
  withSpring,
} from 'react-native-reanimated';

const offset = useSharedValue(0);

const scrollHandler = useAnimatedScrollHandler({
  onScroll: (e) => {
    offset.value = e.contentOffset.y;
  },
  onBeginDrag: (e, ctx) => {
    ctx.startY = e.contentOffset.y;
  },
  onEndDrag: (e, ctx) => {
    if (e.contentOffset.y - ctx.startY > 100) offset.value = withSpring(200);
  },
});

<Animated.ScrollView onScroll={scrollHandler}>{children}</Animated.ScrollView>;
```

Available handlers: `onScroll`, `onBeginDrag`, `onEndDrag`, `onMomentumBegin`,
`onMomentumEnd`. Passing a single function is treated as `onScroll`. The handler
is auto-workletized — no `'worklet'` directive needed. On Web only `onScroll`
fires; the drag/momentum events are iOS/Android only.

## Collapsing header

Map the scroll offset to header height and content fade with `interpolate`.
Clamp so the header doesn't overshoot when bouncing past the top.

```tsx
import Animated, {
  Extrapolation,
  interpolate,
  useAnimatedRef,
  useAnimatedStyle,
  useScrollOffset,
} from 'react-native-reanimated';

const MAX_HEADER = 160;
const MIN_HEADER = 64;
const RANGE = MAX_HEADER - MIN_HEADER;

export function CollapsingHeaderScreen() {
  const ref = useAnimatedRef<Animated.ScrollView>();
  const offset = useScrollOffset(ref);

  const headerStyle = useAnimatedStyle(() => ({
    height: interpolate(
      offset.value,
      [0, RANGE],
      [MAX_HEADER, MIN_HEADER],
      Extrapolation.CLAMP,
    ),
  }));

  const titleStyle = useAnimatedStyle(() => ({
    // Fade and shrink the large title as the header collapses.
    opacity: interpolate(offset.value, [0, RANGE], [1, 0], Extrapolation.CLAMP),
    transform: [
      { scale: interpolate(offset.value, [0, RANGE], [1, 0.85], Extrapolation.CLAMP) },
    ],
  }));

  return (
    <>
      <Animated.View style={[styles.header, headerStyle]}>
        <Animated.Text style={[styles.title, titleStyle]}>Title</Animated.Text>
      </Animated.View>
      <Animated.ScrollView
        ref={ref}
        scrollEventThrottle={16}
        contentContainerStyle={{ paddingTop: MAX_HEADER }}
      >
        {/* list content */}
      </Animated.ScrollView>
    </>
  );
}
```

## Parallax and sticky patterns

The same offset value drives several effects — read it once, interpolate per
element:

- **Parallax hero**: translate a background image at a fraction of scroll speed,
  e.g. `translateY: interpolate(offset.value, [0, H], [0, H * 0.5])`. Allow the
  image to over-scale (`scale` > 1) when scrolling *up* past the top for a
  stretch effect.
- **Sticky header / toolbar**: keep the bar pinned at the top of the scroll
  content, then cross-fade a compact title in once the large header passes a
  threshold (drive both with one `interpolate` over the same input range).
- **Scroll progress bar**: `interpolate(offset.value, [0, contentHeight - viewportHeight], [0, 1])`
  mapped to `scaleX`.

Use `Extrapolation.CLAMP` on the ends so bounce/overscroll doesn't push values
out of range. Keep input ranges in strictly increasing order. See
[Reanimated core](./reanimated-core.md) for `interpolate`/`interpolateColor`
details.

## Reduced motion

Parallax and large collapsing translations are exactly the kind of motion that
can trigger discomfort. When the OS reduced-motion setting is on:

- Drop or heavily damp parallax offsets (track scroll 1:1 instead of at a
  fraction), and avoid over-scale stretch effects.
- Keep functional behavior (the header can still collapse) but remove decorative
  travel and bounce.
- Read the preference with `useReducedMotion()` and branch your interpolation
  ranges. Details and the haptics/AccessibilityInfo story live in
  [Accessibility & performance](./accessibility-performance.md).

## Pitfalls / Do-not

- Do **not** attach a scroll handler to a plain `ScrollView`/`FlatList` — use
  `Animated.ScrollView` / `Animated.FlatList` or the offset is never forwarded.
- Do not read `offset.value` on the JS thread on the scroll hot path; keep
  interpolation inside `useAnimatedStyle` on the UI thread.
- Do not forget `Extrapolation.CLAMP`; without it overscroll/bounce drives
  heights and opacities past their intended bounds.
- Do not give `interpolate` a non-increasing input range — it must be strictly
  increasing.
- Do not animate `height`/layout props every frame on large lists when a
  `transform` (`translateY`/`scaleY`) achieves the same look more cheaply; see
  [Accessibility & performance](./accessibility-performance.md).
- Do not assume drag/momentum events on Web — only `onScroll` fires there.

## Related references

- [Reanimated core](./reanimated-core.md)
- [Worklets & threading](./worklets-threading.md)
- [Gestures](./gestures.md)
- [Layout animations](./layout-animations.md)
- [Accessibility & performance](./accessibility-performance.md)
- [Recipes](./recipes.md)
