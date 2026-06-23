# React Native Testing Scope

Source: https://reactnative.dev/docs/testing-overview

Checked at: 2026-06-04.

## Validation Rules

- Static analysis, unit tests, component tests, and E2E tests prove different
  layers. Do not substitute one for another without naming the risk.
- User interaction tests should assert what users can see or hear instead of
  implementation details such as component props or state.
- React Native component tests run in Node and do not account for iOS or
  Android platform code behind native components.
- E2E tests run against a built app on a device, simulator, or emulator and can
  interact with real screens from the user's perspective.
- E2E tests are slower and costlier to maintain. Use them for vital flows and
  native risks, not every animation detail.

## Native Motion Implications

Use JS tests for:

- pure state transitions;
- reducer/helper logic;
- accessibility labels and visible interaction outcomes;
- Reanimated Jest timing where native runtime is not the risk.

Use native smoke or E2E for:

- gestures, platform views, layout, sensors, haptics, media, or native assets;
- New Architecture compatibility;
- Skia/Rive/Lottie/GL first render and playback;
- route transitions, auth/onboarding, payments, push, app links, files, camera,
  or release-critical flows.

## Closeout Evidence

Record:

- which test layer was used and why it covers the risk;
- user-visible assertion or manual observation;
- simulator/device/platform details for native proof;
- E2E tool and build profile when used.
