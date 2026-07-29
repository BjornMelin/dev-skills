# Expo Tailwind Setup Boundaries

Use this note when an Expo app uses Tailwind, NativeWind, Uniwind,
`react-native-css`, or DOM components. It is a boundary guide, not a copy-paste
installer; follow the target repo's package manager and Expo install policy.

## Expo Official Tailwind Lane

- Expo's Tailwind guide configures standard Tailwind CSS for Metro web.
- Standard Tailwind CSS does not support Android or iOS by itself. Do not
  treat a working Expo web Tailwind setup as native support.
- For universal native support, use a compatibility layer such as NativeWind or
  Uniwind, or stay with RN styles/local tokens.
- For web-only Expo routes, check that `web.bundler` is `metro`, global CSS is
  imported at the entry point, and Tailwind v4 PostCSS/config files match the
  repo's current Tailwind lane.
- For Expo web with Tailwind v4, expect CSS-first `@import "tailwindcss"` and
  `@tailwindcss/postcss` in PostCSS. Old `@tailwind base/components/utilities`
  directives indicate a v3 lane or stale migration.

## Native iOS/Android Choices

Choose one lane deliberately:

- **NativeWind v4**: Tailwind v3-era config and NativeWind v4 setup.
- **NativeWind v5**: Tailwind v4 CSS-first setup, `react-native-css`,
  Reanimated 4+, `react-native-safe-area-context`, and New
  Architecture-oriented validation. Treat this as preview/native-risk.
- **Uniwind**: follow Uniwind docs and do not mix NativeWind-specific Metro or
  component mapping advice into that app.
- **RN styles/local design system**: safest when Tailwind would add more
  package and build complexity than the feature needs.
- **Expo DOM components**: a DOM/WebView island for web UI inside native. Use
  only when product requirements justify DOM semantics, bundle cost, and
  native/web boundary testing.

## Setup Audit

Before changing files, inspect:

- `package.json` for Expo SDK, React Native, NativeWind/Uniwind,
  `react-native-css`, Tailwind, PostCSS, Reanimated, safe-area packages, and
  package-manager overrides.
- `app.json`/`app.config.*` for `web.bundler`.
- `metro.config.*` for `withNativewind` or `withReactNativeCSS` composition.
- `babel.config.*` for stale NativeWind v4 Babel setup in v5 apps.
- `postcss.config.*`, `global.css`, and Tailwind v4 `@source`/`@theme` usage.
- `nativewind-env.d.ts`, `react-native-css-env.d.ts`, or generated type files.
- local wrapper modules that map `className`, `contentContainerClassName`,
  image props, text-input colors, or third-party component style props.
- Expo Router platform-specific files (`.web.tsx`, `.ios.tsx`,
  `.android.tsx`, `.native.tsx`) before introducing runtime `Platform.select`
  class construction.

## Tailwind v4 CSS Shape

For NativeWind v5/react-native-css, expect CSS-first configuration:

```css
@import "tailwindcss/theme.css" layer(theme);
@import "tailwindcss/preflight.css" layer(base);
@import "tailwindcss/utilities.css";
@import "nativewind/theme";

@theme {
  --color-brand: #2563eb;
}

@source "../src";
```

Use `@source inline()` for finite safelists only. In monorepos, prefer explicit
source paths over broad root scans.

For Expo web-only Tailwind, the CSS can be simpler, but the boundary stays the
same: web Tailwind utilities do not make RN native primitives support arbitrary
CSS.

## Component and DOM Boundaries

- Do not create a parallel wrapper system unless the repo lacks a canonical
  styling entrypoint.
- If `globalClassNamePolyfill` is enabled, app-owned components should usually
  pass `className` down instead of registering `cssInterop`.
- If the repo disables the global polyfill, wrappers using `useCssElement` can
  be appropriate, but keep them centralized and typed.
- DOM components need `'use dom'` and must import CSS inside the DOM component
  boundary. Native modules, gestures, accessibility, and layout proof do not
  behave like ordinary RN views inside that island.
- DOM component props cross a native/WebView bridge. Keep props serializable
  and avoid using DOM components to bypass unsupported RN style or motion
  primitives unless product requirements justify the boundary.

## Validation

- Clear Metro cache after setup or migration changes.
- Run the repo's Expo doctor/typecheck/lint gates.
- For native styling packages, use simulator/device proof for iOS and Android,
  especially after Reanimated, Metro, Babel, or New Architecture changes.
- For universal screens, compare web and native screenshots for unsupported CSS
  properties, platform variants, safe areas, dynamic type, dark mode, and
  reduced motion.
- For DOM components, verify native navigation, focus, accessibility labels,
  gestures, bridge payload size, and bundle behavior on a representative
  device.
