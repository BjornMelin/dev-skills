# Cross-platform style and animation differences

Skill: native-styling-boundaries
Checked at: 2026-06-04

## When To Load

- Read when a class, transition, transform, color, or layout utility behaves differently on iOS, Android, and web.

## Source Anchors

- https://www.nativewind.dev/v5/guides/migrate-from-v4
- https://docs.expo.dev/guides/tailwind/

## Reference Notes

- React Native style semantics are not identical to browser CSS. Verify support for transforms, layout, pseudo states, media variants, and transitions per platform.
- Animation ownership should be singular: NativeWind classes can express static/pressed states, while Reanimated should own continuous interactive motion.
- Design tokens should map to platform-supported values rather than leaking browser-only CSS assumptions.

## Focused Checks

- Check iOS, Android, and web where the app supports all three.
- Verify native accessibility states still map to visible states.

## Failure Modes

- Assuming every Tailwind web utility has native parity.
- Combining NativeWind class transitions and Reanimated shared values for the same property without an owner.


## Operating Guidance

NativeWind, react-native-css, Tailwind-style classes in React Native, static class safety, design tokens, Reanimated/CSS transition boundaries, and Expo setup.

### Decision Boundaries

- Use native-motion-core for Reanimated implementation logic.
- Use web-tailwind-motion for browser Tailwind.
- Do not generate untrusted runtime class strings.

### Workflow Details

1. Inspect NativeWind/react-native-css versions and Babel/Metro setup.
2. Keep classes statically discoverable and token-driven.
3. Choose style/class ownership before adding Reanimated or CSS transitions.
4. Validate iOS/Android/web behavior where NativeWind support differs.

### Gotchas

- Web Tailwind assumptions do not always map to React Native style props.
- Runtime string concatenation can break class extraction and policy.
- Animation ownership should not be split between NativeWind classes and Reanimated shared values.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
