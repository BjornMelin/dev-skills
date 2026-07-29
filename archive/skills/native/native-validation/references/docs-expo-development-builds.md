# Expo Development Builds

Source: https://docs.expo.dev/develop/development-builds/introduction/

Checked at: 2026-06-04.

## Validation Rules

- A development build is a debug build of the app that includes
  `expo-dev-client`.
- Expo Go is a prebuilt app with a fixed native library set. It cannot prove
  native code that is not already bundled into Expo Go.
- The JavaScript bundle can hot reload, but native APIs are available only when
  they are included in the installed native app.
- Adding native libraries, app name/icon/splash changes, remote push, and
  Android App Links/iOS Universal Links require a development build or
  release-like build to validate.

## Native Motion Implications

Use a development build instead of Expo Go when validating:

- Reanimated/worklets native package changes;
- NativeWind/Metro/Babel setup that affects native runtime;
- Lottie, Rive, Skia, GL, R3F, native asset, or config plugin changes;
- app icon, splash, permissions, links, push, widgets, share extensions, or
  other native config;
- older SDK behavior not supported by the current Expo Go app.

## Closeout Evidence

Record:

- build type: local development build, EAS development build, preview, or
  release-like build;
- platform: iOS simulator, iOS device, Android emulator, Android device;
- route/screen and user action tested;
- observable result: first render, interaction, playback, unmount, background
  or foreground behavior, reduced motion, asset fallback.
