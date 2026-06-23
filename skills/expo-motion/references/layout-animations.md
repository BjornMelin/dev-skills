# Layout Animations

Reanimated 4 animates components as they **enter**, **exit**, or **change
position/size** in the view hierarchy — without you wiring up shared values.
Attach a builder to an `Animated.View` via `entering`, `exiting`, or `layout`,
and Reanimated handles the rest on the UI thread. This page covers entering/
exiting presets, layout transitions for reorder/add/remove, `Keyframe`, list
animations, and the caveats that bite in production.

Layout animations require the **New Architecture (Fabric)**; they are a no-op (or
warn) on the legacy Paper renderer. All examples assume Reanimated 4.

## Entering and exiting

Mount/unmount the element conditionally and let the builder animate the
transition. Builders are chainable.

```tsx
import Animated, { FadeIn, FadeOut, SlideInRight } from 'react-native-reanimated';

{visible && (
  <Animated.View entering={FadeIn} exiting={FadeOut}>
    <Text>Hello</Text>
  </Animated.View>
)}
```

### Preset families

`Fade`, `Slide`, `Zoom`, `Bounce`, `Flip`, `Stretch`, `Roll`, `Rotate`,
`LightSpeed`, `Pinwheel`. Most have directional variants — `FadeInUp`,
`FadeInDown`, `SlideInLeft`, `SlideInRight`, `ZoomInRotate`, `BounceInDown`, and
so on. Pair an `*In*` entering preset with the matching `*Out*` exiting preset.

### Modifiers

Chain to tune timing, delay, spring physics, and accessibility:

```tsx
import { FadeIn, ReduceMotion } from 'react-native-reanimated';

entering={FadeIn
  .duration(400)
  .delay(120)
  .springify()        // switch to spring-based timing
  .damping(15)
  .reduceMotion(ReduceMotion.System)} // honor the OS reduced-motion setting
```

`ReduceMotion.System` (the recommended default) follows the device setting;
`ReduceMotion.Always` / `ReduceMotion.Never` force the behavior. When reduced
motion is active, Reanimated substitutes a minimal/instant transition.

> Time-based modifiers (`.duration()`, `.easing()`) are **incompatible** with
> spring-based modifiers (`.springify()`, `.damping()`, `.mass()`,
> `.stiffness()`). Pick one family.

## Layout transitions (reorder / add / remove)

When an item's position or size changes because of a state update, a `layout`
transition tweens it to its new slot.

```tsx
import Animated, { LinearTransition } from 'react-native-reanimated';

<Animated.View>
  {items.map((item) => (
    <Animated.View key={item.id} layout={LinearTransition.duration(250)}>
      <Item {...item} />
    </Animated.View>
  ))}
</Animated.View>;
```

Predefined transitions: `LinearTransition`, `SequencedTransition`,
`FadingTransition`, `JumpingTransition`, `CurvedTransition`,
`EntryExitTransition`. The old generic `Layout` transition is deprecated — use
`LinearTransition`.

Spring config for a transition uses **either** physics (`damping`/`stiffness`)
**or** duration (`duration`/`dampingRatio`), never both; if both are given the
duration-based config wins.

## List enter / exit + reorder

`Animated.FlatList` accepts `itemLayoutAnimation` so rows tween when the data
reorders, and per-row `entering`/`exiting` handle add/remove.

```tsx
import Animated, {
  FadeIn,
  FadeOut,
  LinearTransition,
} from 'react-native-reanimated';

function renderItem({ item }: { item: Todo }) {
  return (
    <Animated.View entering={FadeIn} exiting={FadeOut}>
      <TodoRow todo={item} />
    </Animated.View>
  );
}

<Animated.FlatList
  data={todos}
  keyExtractor={(t) => t.id}      // stable keys are mandatory
  renderItem={renderItem}
  itemLayoutAnimation={LinearTransition}
/>;
```

List rules:

- Single-column only — `numColumns` must be `1` (or unset) for `itemLayoutAnimation`.
- Items need a stable `key`/`id` (or a `keyExtractor`); index keys break the animation.
- Set `itemLayoutAnimation={undefined}` to disable at runtime.
- Use `LinearTransition.skipEnteringExitingAnimations` (the modifier on the
  layout builder) to suppress enter/exit on the FlatList's own mount/unmount.

## Keyframe

For multi-step entering/exiting choreography beyond presets:

```tsx
import { Keyframe, Easing } from 'react-native-reanimated';

const popIn = new Keyframe({
  0: { opacity: 0, transform: [{ scale: 0.5 }, { rotate: '-45deg' }] },
  50: {
    opacity: 1,
    transform: [{ scale: 1.2 }, { rotate: '0deg' }],
    easing: Easing.out(Easing.quad),
  },
  100: { transform: [{ scale: 1 }, { rotate: '0deg' }] },
});

<Animated.View entering={popIn.duration(600)} />;
```

Keyframe rules:

- The `0` (or `from`) frame is **required** and must declare every property you animate.
- `100` (or `to`) is optional. Do not mix `0` with `from`, or `100` with `to`.
- Easing belongs on the **second** frame of a pair — never on `0`. Default is `Easing.linear`.
- Every entry in the `transform` array must appear in the **same order** across all frames.

## Skipping animations for a subtree

```tsx
import { LayoutAnimationConfig } from 'react-native-reanimated';

<LayoutAnimationConfig skipEntering skipExiting>
  {children}
</LayoutAnimationConfig>;
```

Useful to suppress the initial-mount entering animation of a freshly mounted
screen. Nestable. For FlatLists prefer the `.skipEnteringExitingAnimations`
modifier instead.

## Pitfalls / Do-not

- **Fabric required.** Layout animations silently do nothing on the legacy
  architecture — confirm New Architecture is enabled.
- **Stable keys.** Index-based keys make reorder/exit animations target the wrong
  rows; always use a stable id.
- **Not every prop animates.** Layout animations interpolate position/size and a
  preset's own properties (opacity, transform). Arbitrary style props are not
  guaranteed; use shared values + `useAnimatedStyle` for those.
- **`nativeID` conflict.** Reanimated uses `nativeID` internally for entering
  animations; overwriting it (common with `TouchableWithoutFeedback`) breaks the
  animation — wrap the animated child in a plain `View`.
- **View flattening.** Removing a non-animated parent can fire children's exiting
  animations but the parent won't wait; add `collapsable={false}` to the parent.
- **Define builders once.** Declare builders at module scope or memoize with
  `useMemo` so they aren't recreated each render.
- **Springy layout animations are not on web.** Spring-based layout animations are
  unavailable on the web platform.
- **Shared Element Transitions are experimental** in Reanimated 4 — not
  recommended for production; prefer navigator-level transitions (see
  [Recipes](./recipes.md)).

## Related references

- [Reanimated core](./reanimated-core.md)
- [Worklets & threading](./worklets-threading.md)
- [Gestures](./gestures.md)
- [Scroll-driven animation](./scroll.md)
- [Accessibility & performance](./accessibility-performance.md)
- [Recipes](./recipes.md)
