# Platform transition option map

Skill: native-controls-transitions
Checked at: 2026-06-04

## When To Load

- Read before changing Expo Router Stack, native-stack, presentation, gesture, header, or modal transition options.

## Source Anchors

- https://docs.expo.dev/router/advanced/stack/
- https://reactnavigation.org/docs/native-stack-navigator/

## Reference Notes

- Navigation transitions are platform contracts. Prefer native-stack options before custom screen overlays when the transition is navigation-owned.
- iOS and Android can expose different transition names, gestures, and modal behavior. Validate both when product behavior is cross-platform.
- Screen options should be centralized where route ownership lives, not scattered across leaf content components.

## Focused Checks

- Verify back gesture, deep link entry, modal dismiss, and reduced-motion settings.
- Check route params and unmount timing for transition cleanup.

## Failure Modes

- Screen-level Reanimated transitions fighting native-stack transitions.
- Assuming iOS modal presentation behavior exists on Android.


## Operating Guidance

Expo Router Stack/native-stack transitions, react-native-screens boundaries, Expo UI SwiftUI/Jetpack Compose controls, native control animation ownership, and validation.

### Decision Boundaries

- Use native-motion-core for Reanimated-owned product motion.
- Use native-styling-boundaries for NativeWind style ownership.
- Use native-validation for EAS/device gates.

### Workflow Details

1. Identify whether navigation, native control, or app state owns the transition.
2. Prefer platform-native transition knobs before custom overlays.
3. Keep Expo UI controls as leaf native controls.
4. Validate iOS and Android behavior when native controls or navigation config change.

### Gotchas

- Navigation transitions can fight screen-level Reanimated transitions.
- Expo UI control props are not arbitrary React Native View animation surfaces.
- Route params and unmount timing affect transition cleanup.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
