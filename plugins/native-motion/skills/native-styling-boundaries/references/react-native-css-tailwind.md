# React Native CSS and Tailwind compatibility

Skill: native-styling-boundaries
Checked at: 2026-06-04

## When To Load

- Read when CSS-like transitions or Tailwind utilities cross native/web boundaries.


## Compatibility Notes

- Treat browser Tailwind utilities as intent, not a direct native contract. React Native style props, units, pseudo states, and layout defaults differ from DOM CSS.
- Prefer static class strings so NativeWind and Metro can discover every utility at build time.
- Keep platform-specific differences explicit with local style objects, variants, or platform files instead of runtime class concatenation.
- Verify CSS transition support against the installed `react-native-css`, NativeWind, and Reanimated versions before mixing utility classes with animated styles.
- Use browser Tailwind guidance only for web surfaces; use this native skill when the target is iOS, Android, or Expo web/native parity.
