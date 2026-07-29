# Expo And React Native New Architecture

Source: https://docs.expo.dev/guides/new-architecture/

Checked at: 2026-06-04.

## Validation Rules

- Expo SDK 55 and later run entirely on the React Native New Architecture. It
  is always enabled and cannot be disabled.
- React Native 0.82 was the first version where disabling the New Architecture
  was removed; Expo SDK 55 uses React Native 0.83 and inherits that behavior.
- Expo SDK 53 and SDK 54 enable the New Architecture by default.
- SDK 54 is the last Expo SDK where the New Architecture can be disabled.
- Expo Doctor integrates React Native Directory data to help identify packages
  that are unmaintained, incompatible, or untested with the New Architecture.
- React Native Directory checks can be configured through the Expo Doctor
  package.json config, but exclusions need package-specific rationale.

## Native Motion Implications

- Treat `newArchEnabled: false` as stale config on Expo SDK 55 and later.
- Reanimated 4 validation must include New Architecture compatibility because
  Reanimated 4 targets the New Architecture.
- Native modules, Fabric/JSI packages, config plugins, GL/canvas libraries,
  Skia/Rive/Lottie packages, and older native packages need platform proof.
- A passing JS test suite cannot prove New Architecture compatibility.

## Closeout Evidence

Record:

- Expo SDK and React Native version;
- whether `newArchEnabled` is present and what it means for that SDK;
- Expo Doctor/React Native Directory findings;
- native build or development-build proof when native packages are involved.
