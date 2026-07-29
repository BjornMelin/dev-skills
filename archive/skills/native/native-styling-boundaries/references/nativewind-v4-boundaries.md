# NativeWind setup and version boundaries

Skill: native-styling-boundaries
Checked at: 2026-06-04

## When To Load

- Read for NativeWind/react-native-css setup and class ownership.


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
