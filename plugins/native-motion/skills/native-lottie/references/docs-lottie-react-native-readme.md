# Native Lottie Reference

Use this file when a task needs concrete Expo or React Native Lottie details.
Check installed package versions before recommending exact code or versions in a
target repo.

## Current Package Facts

- Expo SDK 56 targets React Native 0.85 and React 19.2.3. Its bundled native
  module list pins `lottie-react-native` to `~7.3.4`.
- `lottie-react-native` latest npm release observed during refresh:
  `7.3.8`. Expo SDK 56 apps should normally keep the Expo-compatible install
  unless a tested override is intentional.
- `@lottiefiles/dotlottie-react-native` latest npm release observed during
  refresh: `0.9.3`. The published package depends on
  `@lottiefiles/dotlottie-react` `^0.19.4` for its web path.
- Expo recommends `expo install` for third-party React Native libraries when it
  can select compatible versions, and development builds for native libraries
  that Expo Go does not include.

## `lottie-react-native`

Use this player for standard Bodymovin JSON and Expo-friendly playback.

Useful API surface from the current source/README:

- default import: `LottieView`;
- imperative commands: `play(startFrame?, endFrame?)`, `reset()`, `pause()`,
  `resume()`;
- common props: `source`, `progress`, `speed`, `duration`, `loop`, `autoPlay`,
  `resizeMode`, `renderMode`, `cacheComposition`, `colorFilters`,
  `imageAssetsFolder`, platform text filters, and animation lifecycle events;
- `progress` is normalized `0..1` and should have a single owner;
- `colorFilters` target layer keypaths and need iOS/Android export proof.

### Local JSON Playback

Prefer static local assets for production reliability:

```tsx
import LottieView from 'lottie-react-native';

export function SuccessMark() {
  return (
    <LottieView
      source={require('../assets/success.json')}
      style={{ width: 96, height: 96 }}
      resizeMode="contain"
      loop={false}
      autoPlay={false}
    />
  );
}
```

### Imperative Playback

Own the ref in the wrapper that owns the lifecycle:

```tsx
import { useEffect, useRef } from 'react';
import LottieView from 'lottie-react-native';

export function OnceOnMount() {
  const animationRef = useRef<LottieView>(null);

  useEffect(() => {
    animationRef.current?.play();
    return () => animationRef.current?.reset();
  }, []);

  return (
    <LottieView
      ref={animationRef}
      source={require('../assets/complete.json')}
      style={{ width: 120, height: 120 }}
      loop={false}
    />
  );
}
```

### Progress Control

Use `progress` when app state owns the visual state. React Native `Animated`
progress examples use `useNativeDriver: false`.

```tsx
import { useEffect, useRef } from 'react';
import { Animated, Easing } from 'react-native';
import LottieView from 'lottie-react-native';

const AnimatedLottieView = Animated.createAnimatedComponent(LottieView);

export function ControlledProgress() {
  const progress = useRef(new Animated.Value(0));

  useEffect(() => {
    const animation = Animated.timing(progress.current, {
      toValue: 1,
      duration: 500,
      easing: Easing.linear,
      useNativeDriver: false,
    });
    animation.start();
    return () => animation.stop();
  }, []);

  return (
    <AnimatedLottieView
      source={require('../assets/progress.json')}
      progress={progress.current}
      style={{ width: 120, height: 120 }}
    />
  );
}
```

## `.lottie` Files

Expo Asset lists `.lottie` as an embeddable media type, but custom imports still
need Metro asset extension support.

For `lottie-react-native`, add `lottie` to Metro asset extensions before
importing packaged files:

```js
const { getDefaultConfig, mergeConfig } = require('@react-native/metro-config');

const defaultConfig = getDefaultConfig(__dirname);

module.exports = mergeConfig(defaultConfig, {
  resolver: {
    assetExts: [...defaultConfig.resolver.assetExts, 'lottie'],
  },
});
```

For Jest or Vitest, add a stable file stub when tests import `.lottie`:

```js
// __mocks__/lottieMock.js
module.exports = 'lottie-test-file-stub';
```

```js
module.exports = {
  moduleNameMapper: {
    '\\.(lottie)$': '<rootDir>/__mocks__/lottieMock.js',
  },
};
```

## dotLottie React Native

Use `@lottiefiles/dotlottie-react-native` only when the dotLottie runtime is
part of the product contract. It is not just a drop-in replacement for ordinary
JSON playback.

Current `0.9.3` API facts from the published package:

- source import: `DotLottie`;
- ref type: `Dotlottie`;
- `source`: local module number, string, or `{ uri }`;
- props include `loop`, `autoplay`, `speed`, `themeId`, `marker`, `segment`,
  `playMode`, `useFrameInterpolation`, `stateMachineId`, `renderer`, and
  lifecycle/state-machine callbacks;
- methods include `play`, `pause`, `stop`, `setLoop`, `setSpeed`,
  `setPlayMode`, `setFrame`, `freeze`, `unfreeze`, `resize`, `setSegment`,
  `setMarker`, `setTheme`, `loadAnimation`, state-machine input methods, and
  metrics such as `totalFrames`, `duration`, `currentFrame`, `isPlaying`,
  `activeThemeId`, and `activeAnimationId`.

Expo apps need native binaries for this package. The package ships a config
plugin and docs describe prebuild/development-build/EAS paths; Expo Go is not a
valid native runtime proof.

```tsx
import { useRef } from 'react';
import { Button, View } from 'react-native';
import { DotLottie, Mode, type Dotlottie } from '@lottiefiles/dotlottie-react-native';

export function DotLottieBadge() {
  const ref = useRef<Dotlottie>(null);

  return (
    <View>
      <DotLottie
        ref={ref}
        source={require('../assets/badge.lottie')}
        style={{ width: 160, height: 160 }}
        loop={false}
        autoplay={false}
        playMode={Mode.FORWARD}
      />
      <Button title="Play" onPress={() => ref.current?.play()} />
    </View>
  );
}
```

## Asset Lifecycle Checklist

- Confirm file format, size, dimensions, frame rate, frame range, markers,
  animation IDs, theme IDs, state-machine IDs, text layers, keypaths, and
  external images before coding.
- Check the Lottie supported-features matrix for masks, mattes, merge paths,
  expressions, text, effects, gradients, shadows, and platform differences.
- For image-backed animations, verify Android `imageAssetsFolder` or native
  asset placement and rebuild after asset changes.
- Centralize contract strings: keypaths, text filter targets, markers,
  segments, animation IDs, theme IDs, and state-machine inputs.
- Define missing/corrupt asset behavior and static poster/final-frame fallback.
- Prefer local versioned assets. Remote Lottie requires loading, cache,
  integrity, offline, privacy, and failure behavior.
- Validate OTA/static asset inclusion when using Expo Updates or static export.

## Reduced Motion

Use an existing app hook if one exists. Otherwise, React Native
`AccessibilityInfo.isReduceMotionEnabled()` gives the initial state and
`reduceMotionChanged` lets long-lived components react to setting changes.

```tsx
import { AccessibilityInfo } from 'react-native';
import { useEffect, useState } from 'react';

export function useReduceMotion() {
  const [reduceMotion, setReduceMotion] = useState(false);

  useEffect(() => {
    let mounted = true;

    AccessibilityInfo.isReduceMotionEnabled().then((enabled) => {
      if (mounted) setReduceMotion(enabled);
    });

    const subscription = AccessibilityInfo.addEventListener(
      'reduceMotionChanged',
      setReduceMotion,
    );

    return () => {
      mounted = false;
      subscription.remove();
    };
  }, []);

  return reduceMotion;
}
```

The `useReduceMotion` hook centralizes system preference reads and subscription
cleanup so playback components can disable decorative autoplay without duplicating
accessibility wiring.

When reduced motion is enabled:

- do not autoplay decorative animations;
- replace indefinite loops with a static poster, final frame, or concise state
  UI;
- keep essential state changes visible without relying on completion callbacks;
- preserve accessible labels/state text separately from the animation.

## Performance And Reliability

- Stable dimensions prevent layout shifts and blank first frames.
- Avoid large remote JSON on first paint and avoid many active Lottie instances
  in virtualized rows.
- Android `renderMode`, safe mode, hardware acceleration, and composition cache
  settings should be changed only after reproducing a platform issue.
- Do not use Lottie as a splash-screen bridge for slow startup without a native
  splash fallback.
- Add `onAnimationFailure` / `onLoadError` handling when asset availability is
  not guaranteed.

## Validation Closeout

Run the target repo's focused gates plus manual runtime checks for:

- install/version alignment (`expo install --check` or equivalent);
- typecheck and focused tests;
- Metro bundling and Jest/Vitest `.lottie` mock when used;
- iOS playback, final frame, and missing/corrupt asset path;
- Android playback, final frame, and missing/corrupt asset path;
- reduced-motion fallback;
- route unmount/interruption and replay behavior;
- progress, color filter, marker, segment, theme, animation ID, or
  state-machine contract behavior;
- native rebuild proof after package, native image, config plugin, prebuild, or
  dotLottie runtime changes.
