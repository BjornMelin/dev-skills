# Validation

Motion work in Expo/React Native is mostly *native* work: Reanimated worklets,
Gesture Handler, Skia, Rive, and native navigation transitions all run in native
code that a typecheck, a lint pass, or an Expo Go session cannot exercise. This
reference is the validation contract for motion changes: how to check project
health, pin Expo-compatible package versions, confirm the New Architecture is on,
choose between Expo Go and a development build, scale proof to the blast radius
of the change with a risk-tier ladder, write Reanimated-aware Jest tests, and
write a closeout report. The governing rule: **device proof is required for
native motion — an Expo Go session is not sufficient proof for native modules.**

## Project health: `npx expo-doctor`

Run Expo Doctor first. It surfaces dependency-version mismatches, New
Architecture issues, and config problems before you spend time chasing a runtime
symptom that is really a setup problem.

```bash
npx expo-doctor
```

Treat every warning as a lead, not a verdict. Fix the ones that touch your motion
surface (a mismatched `react-native-reanimated`, `react-native-gesture-handler`,
or `react-native-worklets` version is the usual culprit) and document the rest
with a reason in your closeout.

## Expo-compatible package versions

Each Expo SDK pins a known-good version of every native package it manages.
**These Expo-pinned versions lag npm `latest`** — a copy-pasted `npm install
react-native-reanimated@latest` will frequently install a version that does not
match your SDK and breaks the build or the runtime. Always resolve motion package
versions through Expo, not through npm-latest.

Check what is drifting:

```bash
npx expo install --check
```

Then let Expo fix or add packages at the SDK-correct version:

```bash
# Reconcile existing packages to SDK-compatible versions
npx expo install --fix

# Add a motion package at the version Expo blesses for this SDK
npx expo install react-native-reanimated react-native-worklets
npx expo install react-native-gesture-handler
npx expo install @shopify/react-native-skia
```

When a doc, changelog, or this skill's examples show a version number, treat it
as illustrative. The authority for *your* repo is what `expo install` resolves
against your installed SDK.

## New Architecture verification

**Reanimated 4 requires the New Architecture (Fabric + TurboModules).** A repo
running the legacy architecture cannot run Reanimated 4; it must either enable
the New Architecture or stay on the Reanimated 3 line. Confirm before you write
or review Reanimated 4 code.

The flag lives in `app.json` / `app.config.js`:

```jsonc
{
  "expo": {
    "newArchEnabled": true
  }
}
```

Expo SDK 52+ defaults the New Architecture on, but never assume — read the config
and confirm at runtime. `npx expo-doctor` flags New Architecture compatibility
problems, and a development build will fail fast if a native module is
incompatible. If the project is intentionally on the old architecture, that is a
hard constraint on which Reanimated major you can use; record it rather than
silently upgrading.

## Development build vs Expo Go

Expo Go ships a fixed set of native modules. It can run plain React Native and
JS-only animation, but it **cannot load custom native code**. The motion
libraries in this skill are native:

- Reanimated worklets and the Worklets runtime
- `react-native-gesture-handler`
- `@shopify/react-native-skia`
- Rive (`rive-react-native`)
- `lottie-react-native`

Any change touching these needs a **development build** — a custom dev client
compiled with your native dependencies — not Expo Go.

```bash
# Build and run a local development build on a connected device/simulator
npx expo run:ios
npx expo run:android

# Or produce a development-client build via EAS
eas build --profile development --platform ios
eas build --profile development --platform android
```

If a teammate says "it works in Expo Go," that proves nothing about native motion
behavior — Expo Go may be silently falling back or running a different code path.
**Expo Go is never acceptable proof for a native-module motion change.**

### EAS Build risk

EAS Build compiles your native project in the cloud. It is the highest-cost,
slowest-feedback rung: a misconfigured native dependency or version mismatch can
fail a build minutes in, and a green EAS build still does not prove the animation
*looks* right — only that it compiled and installed. Reserve full EAS builds for
release-risk changes (new native dependency, config plugin, SDK bump) and prove
visual/feel correctness on a real device separately.

## Risk-tier validation ladder

Match the proof to how *native* the change is. Climb only as high as the change
requires, but never skip the rung the change actually lives on.

| Tier | Change surface | Minimum proof |
| --- | --- | --- |
| 1. Static / local test | Pure JS view motion, timing/interpolation math, reduced-motion branching | `tsc` + lint + Jest unit tests |
| 2. Simulator | JS-driven layout/opacity/transform motion, no new native module | Tier 1 + simulator smoke run |
| 3. Physical device | Gesture feel, frame pacing, haptics, scroll-linked motion | Tier 2 + iOS **and** Android device run |
| 4. Development build | New/changed native module (Reanimated, Skia, Rive, Lottie, Gesture Handler) | Tier 3 on a **development build**, not Expo Go |
| 5. EAS | New native dependency, config plugin, SDK/arch change, release gating | Tier 4 + `eas build` for the affected platforms |

Classify each touched file (JS-only, package/config, native module, GPU/canvas,
navigation, release-risk), then run the smallest set that actually proves the
changed runtime surface. A `tsc` pass is not validation for a native runtime or
GPU change. Capture iOS and Android separately — they have different animation
backends and diverge in real ways.

## Jest + Reanimated test setup

Unit tests are Tier 1: they run on the JS runtime with the worklet machinery
mocked. They are fast and deterministic, and they are the right tool for the
logic *around* motion — but they do not render on a device.

Add Reanimated's mock in your Jest setup file:

```js
// jest-setup.js
require('react-native-reanimated').setUpTests(); // default config { fps: 60 }
```

Wire that file in `jest.config.js` (use `setupFiles` instead on Jest < 28):

```js
// jest.config.js
module.exports = {
  preset: 'react-native',
  setupFilesAfterEnv: ['./jest-setup.js'],
};
```

### Fake-timer discipline

Animations advance on timers. Drive them with Jest's fake timers and assert at
explicit time offsets — **never sleep on the wall clock.** Establish fake timers
before triggering the animation and advance them deterministically.

```tsx
import { render, fireEvent } from '@testing-library/react-native';

beforeEach(() => {
  jest.useFakeTimers();
});

test('expands halfway through the 500ms animation', () => {
  const { getByTestId } = render(<ExpandingCard />);
  const view = getByTestId('card');
  const button = getByTestId('toggle');

  expect(view).toHaveAnimatedStyle({ width: 100 });

  fireEvent.press(button);
  jest.advanceTimersByTime(250); // half of a 500ms animation

  expect(view).toHaveAnimatedStyle({ width: 175 });
});
```

`toHaveAnimatedStyle` (and `toHaveAnimatedProps`) come from the Reanimated Jest
setup. Add `{ shouldMatchAllProps: true }` to assert the full style object rather
than a subset.

### What unit tests can and cannot assert

- **Can assert:** deterministic value mapping (interpolation in/out), reduced-motion
  branching, mount/unmount lifecycle guards, that an animation *starts* and reaches
  an expected intermediate/final style at a given time, and that a callback crosses
  back to the RN runtime on completion.
- **Cannot assert:** real frame pacing or dropped frames, gesture feel, native
  rendering correctness, Skia/GPU output, Rive state-machine visuals, or native
  navigation transition smoothness. Those need a device (Tier 3+). Snapshot tests
  in particular do not prove motion behavior — do not lean on them for animation.

Pair every native-module test with device proof; the unit test guards the logic,
the device proves the motion.

## Closeout report

End every motion change with an explicit report so the reviewer can see what was
proven and what risk remains:

```text
## Validation closeout

Commands run:
- npx expo-doctor
- npx expo install --check
- tsc --noEmit && <lint>
- <jest command>
- npx expo run:ios  (development build)

Findings fixed:
- Bumped react-native-reanimated to the SDK-pinned version via expo install --fix.

Findings skipped (with reason):
- expo-doctor warning on <unrelated package>: outside motion surface, no behavior change.

Residual risk:
- Android low-end frame pacing untested on physical hardware; verified on Pixel emulator only.

Device proof:
- iPhone 15 (iOS 18): swipe-to-dismiss gesture + spring settle confirmed smooth.
- Pixel 7 (Android 15): same flow confirmed; screenshot/recording attached.
```

A closeout with no device proof for a native-module change is incomplete. State
the devices and OS versions, and attach a recording or screenshot for the
user-visible motion.

## Pitfalls / Do-not

- **Do not trust npm `latest` over `expo install --check`.** Expo-pinned versions
  lag npm, and a `@latest` install routinely breaks the build or runtime for your
  SDK. Resolve every motion package through `expo install`.
- **Do not use Expo Go as proof for a native-module change.** Reanimated worklets,
  Skia, Rive, Lottie, and Gesture Handler need a development build. Expo Go cannot
  load them, so "it works in Expo Go" proves nothing about native motion.
- **Do not skip device proof.** `tsc`, lint, and Jest never exercise the native
  animation path. A native motion change is unvalidated until it has run on real
  iOS *and* Android hardware.
- **Do not assume the New Architecture is on.** Reanimated 4 requires it; read the
  config and confirm at runtime rather than inferring it from the SDK version.
- **Do not assert motion with `sleep` or snapshots.** Use fake timers and
  `toHaveAnimatedStyle` at explicit time offsets for the logic you can unit-test.

## Related references

- [Reanimated core](./reanimated-core.md)
- [Worklets & threading](./worklets-threading.md)
- [Accessibility & performance](./accessibility-performance.md)
