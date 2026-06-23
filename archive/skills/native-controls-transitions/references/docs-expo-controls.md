# Native Control Selection Notes

Use this when choosing between React Native controls, Expo UI universal/drop-in
controls, platform-specific Expo UI controls, and app-owned custom controls.

## Defaults

- Use React Native built-ins first when they satisfy the product need:
  `Switch`, `Pressable`, `TextInput`, platform accessibility props, and standard
  navigation primitives.
- Use Expo UI universal controls when the app already has `@expo/ui` or when
  native mobile fidelity matters enough to add it.
- Use Expo UI drop-in replacements when replacing a community control and the
  used API surface is supported.
- Use platform-specific SwiftUI/Compose controls when the universal API is
  missing the required behavior.
- Use custom Reanimated controls only for product-specific interactions that
  native controls cannot express.
- If the value of the feature is native affordance, choose native first. If the
  value is brand-specific product behavior, choose app-owned content motion.

## Control Checklist

- Confirm target platforms and installed package versions.
- Confirm whether Expo Go is enough or whether a development build/rebuild is
  required after installing native packages.
- Keep labels, hints, values, disabled state, focus order, and dynamic type
  behavior explicit.
- Avoid custom colors that break dark mode, high contrast, or Material/SwiftUI
  theming unless the design system owns those tokens.
- Use app-owned segmented controls for product modes that are not a native
  platform segmented-control fit.
- Use native sheets, menus, date/time pickers, sliders, switches, and search
  bars when native behavior is the value.

## Migration Notes

- Community packages can expose more props than Expo UI replacements. Diff the
  installed types before replacing broad call sites.
- Keep platform fallbacks visible in code. Avoid a single cross-platform wrapper
  that silently drops important props on one platform.
- Treat icon assets as platform-specific unless the source package explicitly
  supports both targets.
- Treat wrapper components that erase platform differences as risk. They are
  acceptable only when unsupported props are either not needed or replaced with
  explicit platform behavior.
