# NativeWind & styling boundaries

NativeWind brings Tailwind-style `className` styling to React Native, but in an
Expo (SDK 56+) New Architecture app it sits next to Reanimated 4, not on top of
it. This reference covers the v4 stable and v5 preview lanes, how NativeWind's
motion/transition utilities relate to Reanimated, the **static class-safety
policy** (no runtime string concatenation), design tokens, the single-owner rule
for animation, and which web-Tailwind props don't map to RN. For the animation
engine itself see [Reanimated core](./reanimated-core.md); for content
enter/exit see [Layout animations](./layout-animations.md).

## Version lanes: v4 stable vs v5 preview

Pick one lane deliberately and confirm installed versions before editing.

### NativeWind v4 (stable / Tailwind v3)

- Stable lane: `nativewind@latest` resolves to the 4.2.x line. Use this unless
  the repo explicitly opts into `preview`.
- Tailwind 3.4-era `tailwind.config.js`, `@tailwind base/components/utilities`
  directives, and NativeWind Babel + Metro wiring.
- Type lane: `nativewind-env.d.ts` / `nativewind/types`.

### NativeWind v5 (preview)

- Pre-release (`nativewind@preview`, the 5.0.0-preview line). The official v5
  docs say it is **not intended for production yet** — do not present it as the
  default upgrade.
- Tailwind v4.1+ CSS-first config: `@import`, `@theme`, `@utility`,
  `@custom-variant`, `@source`, `@plugin`, `@apply`. Expects `react-native-css`,
  `@tailwindcss/postcss`, Reanimated 4+, `react-native-safe-area-context`,
  Lightning CSS, and React Native 0.81+ / New Architecture behavior.
- Metro: prefer `withNativewind(config)` from `nativewind/metro` (lowercase `w`
  is the v5 spelling; `withNativeWind` survives for compat). Do **not** add the
  old NativeWind Babel preset to a v5 app.
- Add `@import "nativewind/theme";` for RN theme values and variants
  (`native:`, `web:`, `ios:`, `android:`, `tv:`).
- Type lane: `react-native-css/types` or the generated v5 env file.
- v5 replaces `remapProps`/`cssInterop` with `styled` for most component
  styling; keep the older APIs only when auditing v4 or mapping native props.

A mixed setup — Tailwind v4 imports plus `nativewind/babel`, or Tailwind v3
directives plus NativeWind v5 packages — is a migration smell.

## Motion utilities and how they relate to Reanimated

NativeWind v5's animation utilities are backed by Reanimated's CSS-animation
support, but many animation classes are marked **experimental or partial** in
the native docs. Use the right tool per job.

```tsx
// Good: NativeWind owns static + pressed state; a simple supported transition.
<Pressable className="rounded-lg bg-blue-600 px-4 py-2 active:bg-blue-700 transition-colors">
  <Text className="text-white">Save</Text>
</Pressable>
```

- Use class utilities for static styles, pseudo-state styles (`active:`,
  `disabled:`), and simple supported transitions.
- Use **Reanimated directly** for gestures, scroll-linked values, route
  transitions, interruption/cancellation, timelines, layout reads, and
  reduced-motion-aware behavior (see [Reanimated core](./reanimated-core.md)).
- Avoid toggling animation classes in ways that remount children or reset state;
  `react-native-css` warns about adding/removing animation classes after the
  initial render and suggests `will-change-animation` only when that is
  intentional and verified.

## Static class safety (no runtime string concatenation)

Tailwind scans source **as text**. It cannot infer strings assembled by
interpolation, and the safety policy forbids untrusted runtime class strings.

```tsx
// BAD — breaks Metro/Babel extraction; class never generated.
<View className={`bg-${color}-600`} />

// GOOD — full utility names present in source, statically discoverable.
const toneClass = {
  info: 'bg-blue-600 text-white',
  danger: 'bg-red-600 text-white',
} as const;

<View className={toneClass[tone]} />;
```

- Keep complete utility names in source. Never build class names from APIs,
  users, CMS fields, or remote config — convert external values to approved
  tokens or CSS variables first.
- In Tailwind v4, use `@source` for monorepo/shared-package paths and
  `@source inline()` only for a finite, intentional safelist. Avoid broad
  `@source "../.."`, wildcard, vendor, or dependency scans — they slow builds
  and hide ownership of generated classes.
- Pipeline reality: NativeWind/`react-native-css` setup is a toolchain — Babel,
  Metro, the CSS entrypoint, Tailwind content scanning, and package versions
  must agree. When classes don't apply, debug the setup (config, Metro cache,
  content paths), not just component code. Clear the Metro cache after setup or
  migration changes.

## Design tokens

- Tailwind v4 static tokens live in CSS via `@theme`
  (e.g. `--color-brand: #2563eb;`). Keep token names semantic and shared across
  platforms; use platform media queries for platform-specific values, not
  divergent token meanings.
- For runtime themes in v5, prefer `VariableContextProvider` with CSS-variable
  defaults. `vars()` is still exported but deprecated in current v5 docs.
- If dynamic variables don't update with `react-native-css`, check Metro
  options — inlining can conflict with runtime variable contexts; use
  `inlineVariables: false` only when validation confirms it is needed.

## Ownership boundary: pick one animation owner

Do **not** split one animation across NativeWind classes and Reanimated shared
values for the same property. Choose a single owner per visual property.

```tsx
// Reanimated owns continuous/interactive motion for this element.
import Animated, { useSharedValue, useAnimatedStyle, withSpring } from 'react-native-reanimated';

function Card() {
  const scale = useSharedValue(1);
  const style = useAnimatedStyle(() => ({ transform: [{ scale: scale.value }] }));
  // className handles static look; Animated style owns the moving property.
  return (
    <Animated.View className="rounded-xl bg-white p-4 shadow" style={style}>
      {/* ... */}
    </Animated.View>
  );
}
```

- NativeWind classes can express static and pressed states; Reanimated should
  own continuous interactive motion. Animation ownership must be **singular**.
- For app-owned components, pass `className` through to RN primitives or a local
  styled wrapper; do not register global component mappings from screens.
- Use `styled(Component)` (v5) where docs support it; `remapProps` for
  multi-style-prop components (v4); `cssInterop` only when styles must extract
  into native props (`placeholderTextColor`, `selectionColor`,
  `contentContainerStyle`). Keep wrappers in one local, typed module.

## Web-Tailwind props that don't map to RN

Treat browser Tailwind utilities as **intent, not a native contract**. RN style
props, units, pseudo-states, and layout defaults differ from DOM CSS.

- A working Expo **web** Tailwind setup is not native support: standard Tailwind
  CSS does not support iOS/Android by itself. For universal native, use a
  compatibility layer (NativeWind / Uniwind) or RN styles/local tokens.
- Verify per platform: transforms, layout, pseudo-states, media variants, and
  transitions are not guaranteed to have native parity. Do not assume every web
  utility maps to a native primitive.
- Keep platform differences explicit with local style objects, variants, or
  platform files (`.ios.tsx`, `.android.tsx`, `.web.tsx`) rather than runtime
  `Platform.select` class construction.
- DOM components need `'use dom'`, import CSS inside the DOM boundary, and cross
  a serializable native/WebView bridge — they don't behave like ordinary RN
  views and aren't a shortcut around unsupported RN style/motion primitives.

## Pitfalls / Do-not

- Do not build class names by interpolation or from untrusted/runtime data —
  extraction breaks and the safety policy fails.
- Do not animate the same property with both a NativeWind transition class and a
  Reanimated shared value — pick one owner.
- Do not present v5 preview as a production default upgrade.
- Do not mix Tailwind v3 directives with v5 packages (or v4 CSS with the v4
  Babel preset) — that is a migration smell.
- Do not assume web Tailwind utilities have native parity; validate iOS,
  Android, and web (dark mode, dynamic type, reduced motion).
- Do not scatter `cssInterop`/`remapProps` registration across screens; keep one
  local, typed wrapper module.
- Do not toggle animation classes in ways that remount children or reset state.

## Related references

- [Reanimated core](./reanimated-core.md) — shared values, worklets, threading,
  reduced motion; the engine NativeWind v5 motion utilities sit on.
- [Layout animations](./layout-animations.md) — entering/exiting and `Layout`
  content motion when classes are not enough.
- [Expo Router & screen transitions](./expo-router-transitions.md) — navigation
  vs content animation ownership and `@expo/ui` leaf controls.
- [Recipes](./recipes.md) — end-to-end styled + animated component patterns.
