# NativeWind Metro, Babel, and CSS pipeline

Skill: native-styling-boundaries
Checked at: 2026-06-04

## When To Load

- Read when NativeWind classes do not apply, hot reload fails, or setup crosses Expo/Metro/Tailwind config.

## Source Anchors

- https://www.nativewind.dev/docs/getting-started/installation
- https://docs.expo.dev/guides/tailwind/

## Reference Notes

- NativeWind/react-native-css setup is a toolchain pipeline: Babel, Metro, CSS entrypoint, Tailwind content scanning, and package versions must agree.
- Expo and native projects can differ in CSS asset support. Verify the actual app surface before copying setup from web Tailwind.
- Class extraction depends on static discoverability unless the local policy provides an explicit mapping.

## Focused Checks

- Inspect package versions, Babel config, Metro config, CSS import, and content paths.
- Run a clean-start or cache-reset check when setup changes.

## Failure Modes

- Debugging missing styles only at component code while setup is broken.
- Runtime class strings built from arbitrary user or CMS data.


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
