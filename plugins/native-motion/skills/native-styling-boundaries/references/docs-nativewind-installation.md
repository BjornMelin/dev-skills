# NativeWind, Tailwind, and react-native-css Boundaries

Use this note when changing NativeWind setup, migrating Tailwind versions,
mapping class names into third-party components, or reviewing runtime style and
motion boundaries.

## Version Lanes

### NativeWind v5 Preview

- Treat v5 as pre-release unless the target repo has explicitly adopted it.
  Current registry evidence pins `nativewind@preview` to 5.0.0-preview.4.
- Do not present v5 as a production-default upgrade. The official v5 docs call
  the release pre-release and state that it is not intended for production use
  yet.
- Expect Tailwind CSS v4.1+ and CSS-first configuration: `@import`, `@theme`,
  `@utility`, `@custom-variant`, `@source`, `@plugin`, and `@apply`.
- Expect `react-native-css`, `@tailwindcss/postcss`, Reanimated 4+,
  `react-native-safe-area-context`, Lightning CSS, and React Native New
  Architecture-oriented behavior. The v5 docs list React Native 0.81+ as a
  practical baseline for setup.
- Prefer `withNativewind(config)` from `nativewind/metro`. The older
  `withNativeWind` spelling exists for compatibility, but lowercase `w` is the
  v5 spelling.
- Do not add the old NativeWind Babel preset to a v5 app. If Babel config is
  still present, verify whether it is for another tool before deleting it.
- Add `@import "nativewind/theme";` when the app needs NativeWind's RN theme
  values and variants such as `native:`, `web:`, `ios:`, `android:`, and `tv:`.
- Use the v5 env typing lane (`react-native-css/types` or the generated env
  file for the target setup). `nativewind/types` is v4-era guidance.
- NativeWind v5 replaces older `remapProps` and `cssInterop` customization
  patterns with `styled` for most component styling. Keep older APIs only when
  auditing v4 apps or a proven third-party/native prop mapping still needs them.

### NativeWind v4 / Tailwind v3

- Do not apply v5 CSS-first guidance to a v4 app without a migration task.
- Current registry evidence pins `nativewind@latest` to 4.2.4. Treat that as
  the stable lane unless the repo explicitly opts into `preview`.
- v4 setup commonly includes Tailwind 3.4-era `tailwind.config.js`,
  `@tailwind` directives, and NativeWind Babel/Metro wiring.
- A mixed setup such as Tailwind v4 imports plus `nativewind/babel`, or
  Tailwind v3 directives plus NativeWind v5 packages, is a migration smell.

## Class Extraction

- Tailwind scans source as text. It cannot infer strings assembled by
  interpolation such as `bg-${color}-600`.
- Keep complete utility names in source:

```tsx
const toneClass = {
  info: 'bg-blue-600 text-white',
  danger: 'bg-red-600 text-white',
} as const;

<View className={toneClass[tone]} />;
```

- In Tailwind v4, use `@source` for monorepo/shared-package paths and
  `@source inline()` only for finite, intentional generated utilities.
- Avoid broad `@source "../.."`, wildcard, generated, vendor, or dependency
  scans. They make builds slower and can hide ownership of generated classes.
- Avoid accepting class names from APIs, users, CMS fields, or remote config.
  Convert external values to approved tokens or CSS variables.

## Tokens and Runtime Themes

- For static tokens in Tailwind v4, define values in CSS with `@theme`.
- For runtime themes in NativeWind v5, prefer `VariableContextProvider` with
  CSS variable defaults. `vars()` remains exported but is deprecated in current
  v5 docs.
- If dynamic variables do not update as expected with `react-native-css`, check
  Metro options. Source docs warn that inlining variables can conflict with
  runtime variable contexts; use `inlineVariables: false` only when the target
  setup needs dynamic variable behavior and validation confirms it.
- Keep token names semantic and shared across platforms. Use platform media
  queries for platform-specific values, not divergent token meanings.

## Component Mapping

- For app-owned components, pass `className` through to RN primitives or the
  local styled wrapper. Do not register global component mappings from screens.
- In NativeWind v5, prefer `styled(Component)` where official docs support the
  component. Keep wrappers typed and colocated with the local design-system
  boundary.
- In NativeWind v4 or legacy third-party integrations, use `remapProps` for
  components with multiple style props when a simple class-name-to-style mapping
  is enough.
- Use `cssInterop` only when styles must be extracted into native props such as
  `placeholderTextColor`, `selectionColor`, `contentContainerStyle`, or
  component-specific props.
- Use `useCssElement` when the repo deliberately disables the global className
  polyfill or needs a reusable CSS-enabled wrapper.
- Keep wrappers in one local module. Do not scatter `cssInterop` registration
  across unrelated screens.

## Motion Boundary

- NativeWind v5 animation utilities are backed by Reanimated CSS animation
  support, but native docs mark many animation classes experimental or partial.
- Use class utilities for static styles, pseudo-state styles, and simple
  supported transitions.
- Use Reanimated directly for gestures, scroll-linked values, route
  transitions, interruption/cancellation, timelines, layout reads, and
  accessibility-aware reduced-motion behavior.
- Avoid toggling animation classes in a way that remounts children or resets
  state. `react-native-css` source warns about animation class addition/removal
  after initial render and suggests `will-change-animation` only when that
  behavior is intentional and verified.

## Review Checklist

- Confirm the installed NativeWind, Tailwind, `react-native-css`, Reanimated,
  React Native, and Expo SDK versions.
- Confirm Metro wraps the config once and in the right order for the repo.
- Confirm global CSS uses the correct Tailwind lane: v4 imports or v3
  directives, not both by accident.
- Confirm monorepo source paths are explicit and narrow.
- Confirm the env typing lane matches the installed package generation:
  `nativewind-env.d.ts`/`nativewind/types` for v4, `react-native-css/types` or
  generated v5 env files for v5.
- Confirm iOS, Android, and web behavior for platform variants, dark mode,
  dynamic type, reduced motion, and any runtime theme changes.
