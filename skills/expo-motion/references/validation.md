# Validation

Motion work in Expo/React Native crosses JavaScript, native, GPU, and navigation
boundaries. A typecheck or lint pass cannot exercise every boundary, and Expo Go
only contains the native modules supported by the target Expo SDK. This reference
is the validation contract for motion changes: check project health, resolve
Expo-compatible package versions, confirm the target architecture, choose between
Expo Go and a development build from the target package list, scale proof to the
blast radius, write Reanimated-aware tests, and record a closeout.

## Project health: Expo Doctor

Run Expo Doctor first. It surfaces dependency-version mismatches, New
Architecture issues, and config problems before you spend time chasing a runtime
symptom that is really a setup problem.

```bash
# Replace <repo-expo-doctor> with the target repo's package-manager wrapper for
# the Expo Doctor command used by that project.
<repo-expo-doctor>
```

Treat every warning as a lead, not a verdict. Fix the ones that touch your motion
surface (a mismatched `react-native-reanimated`, `react-native-gesture-handler`,
or `react-native-worklets` version is the usual culprit) and document the rest
with a reason in your closeout.

## Expo-compatible package versions

Each Expo SDK publishes a compatible version range for the native packages it
manages. Do not resolve native motion packages from an arbitrary registry
`latest`; use the target repo's Expo version resolver and lockfile.

Check what is drifting:

```bash
<repo-expo> install --check
```

Then let Expo fix or add packages at the SDK-correct version:

```bash
# Reconcile existing packages to target-SDK-compatible versions
<repo-expo> install --fix

# Add motion packages through the target SDK's resolver. Choose one lane:
# Reanimated 4 + New Architecture
<repo-expo> install react-native-reanimated react-native-worklets
# Reanimated 3 / legacy architecture (do not add react-native-worklets)
<repo-expo> install react-native-reanimated
<repo-expo> install react-native-gesture-handler
<repo-expo> install @shopify/react-native-skia
```

When a doc, changelog, or this skill's examples show a version number, treat it
as illustrative. The authority for *your* repo is what the wrapper resolves
against the installed Expo SDK and lockfile.

## New Architecture verification

**Reanimated 4 requires the New Architecture (Fabric + TurboModules).** A repo
running the legacy architecture cannot run Reanimated 4; it must either enable
the New Architecture or stay on the Reanimated 3 line. Confirm before you write
or review Reanimated 4 code.

The flag lives in `app.json` / `app.config.*`:

```jsonc
{
  "expo": {
    "newArchEnabled": true
  }
}
```

Expo's [New Architecture guide](https://docs.expo.dev/guides/new-architecture/)
is the authority for the target release. When the target release mandates the
New Architecture, `newArchEnabled: false` is ignored: use the Reanimated 4 and
Worklets lane. When the target release still supports the legacy architecture,
inspect `newArchEnabled` in the app config and computed manifest; a legacy target
must stay on a compatible Reanimated 3 lane until migrated. Never infer the
architecture from a package version alone. `<repo-expo-doctor>` flags
compatibility problems, and a development build will fail fast if a native
module is incompatible.

## Development build vs Expo Go

Expo Go ships a fixed set of native modules for each target SDK. It may include
Reanimated, Gesture Handler, Skia, or `@expo/ui`, but support is target-version
and platform-specific. Custom/unsupported native packages still require a
development build. Check the target Expo SDK's supported-package list before
choosing the proof path.

Use a development build — a custom dev client compiled with the target native
dependencies — when the package is not in that list, native configuration changes,
or production-quality device proof is required.

```bash
# Build and run a local development build on a connected device/simulator
<repo-expo> run:ios
<repo-expo> run:android

# Or produce a development-client build via EAS
<repo-eas> build --profile development --platform ios
<repo-eas> build --profile development --platform android
```

An Expo Go smoke run proves only the code path covered by that SDK's bundled
modules. It does not prove a custom native module, native configuration, or
production build; record the exact SDK, platform, and package support used.

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
| 2. Expo Go or simulator | JS-driven motion, or a package listed as supported by the target Expo SDK | Tier 1 + target-platform smoke run |
| 3. Physical device | Gesture feel, frame pacing, haptics, scroll-linked motion | Tier 2 + iOS **and** Android device run |
| 4. Development build | Unsupported/custom native module, native config, or production-quality proof | Tier 3 on a **development build** |
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
- <repo-expo-doctor>
- <repo-expo> install --check
- tsc --noEmit && <lint>
- <jest command>
- <repo-expo> run:ios  (development build, when required)

Findings fixed:
- Resolved react-native-reanimated and Worklets through the target SDK wrapper.

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

- **Do not trust registry `latest` over the target SDK resolver.** Resolve every
  Expo-managed motion package through the repo's wrapper and lockfile.
- **Do not treat Expo Go as universal native proof.** Its bundled package list is
  target-SDK/platform-specific; unsupported or custom native modules need a
  development build.
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
