# Native Validation Playbook

Load this when a task needs a proof plan, audit triage, or closeout report for
native motion work.

## Evidence Ladder

Pick the first level that covers the changed boundary. Do not escalate for
optics; escalate when lower proof cannot observe the risk.

| Level | Boundary | Evidence |
| --- | --- | --- |
| 0 Static | docs, comments, non-runtime metadata, test-only edits | repo format/lint/typecheck/docs validation |
| 1 JS runtime | component-only animation logic, pure state, style-only motion | focused unit/component tests; app smoke if user-visible |
| 2 Native runtime smoke | Reanimated behavior, NativeWind setup, gestures, reduced motion, existing native assets/packages | simulator/emulator/device proof on affected platforms |
| 3 Native binary | native dependency, config plugin, permissions, GL/Skia/Rive/Lottie linkage, app config, prebuild output | Expo Doctor/install check plus local native build, development build, or EAS build |
| 4 Release/E2E | runtimeVersion, OTA, release profiles, credentials, app links, push, auth, payments, critical flows | release runbook, EAS profile proof, and E2E or store-like manual proof |

## Command Selection

Use repo wrappers first. Translate examples to the repo package manager:

- Package compatibility: `expo install --check` or the repo's wrapper.
- Expo health: `expo-doctor` or the repo's doctor script.
- Config inspection: `expo config --type prebuild` for config plugins,
  permissions, entitlements, and native settings.
- Local compile proof: `expo run:ios`, `expo run:android`, or repo wrappers
  when the workstation supports the platform.
- Cloud/release parity: `eas build --platform ios|android|all --profile <name>`
  or repo release workflow.
- Animation tests: focused Jest/Vitest/RNTL tests plus Reanimated setup proof
  when they assert animation timing or animated style.

## Package Pin Review

Always inspect declared and locked versions before recommending package edits:

- Expo SDK and `expo` package.
- `react-native`.
- `react-native-reanimated`.
- `react-native-worklets`.
- NativeWind and `react-native-css` if styling boundaries are involved.
- `@shopify/react-native-skia`, `@rive-app/react-native`,
  `lottie-react-native`, `expo-gl`, `expo-dev-client`,
  `expo-build-properties`, and other native motion packages in scope.

For Expo apps, prefer Expo-compatible pins from `expo install --check` over
latest npm versions. Latest npm metadata is provenance only until compatibility
is verified for the app's SDK.

## Expo And New Architecture

- Run Expo Doctor for native-risk Expo changes. It covers app config,
  package compatibility, React Native Directory, and native-directory sync.
- SDK 55 and later always use the New Architecture. Treat
  `newArchEnabled: false` as stale config.
- SDK 53 and SDK 54 enable the New Architecture by default; SDK 54 is the last
  SDK where disabling it is allowed.
- Reanimated 4 only targets the New Architecture. Validate native packages
  and older third-party modules against this boundary.
- React Native Directory findings are proof inputs, not noise. Accept excludes
  only with package-specific rationale.

## Reanimated And Jest

- Reanimated 4 requires a declared `react-native-worklets` dependency.
- React Native CLI apps with explicit Babel config need
  `react-native-worklets/plugin` as the final plugin.
- Expo apps should normally rely on Expo-compatible versions and
  `babel-preset-expo`; do not add duplicate Babel config without repo evidence.
- Jest animation tests must initialize Reanimated with
  `require('react-native-reanimated').setUpTests()`.
- Use Jest fake timers for time-based animation assertions such as
  `advanceTimersByTime`.
- JS component tests do not prove native layout, gestures, native modules,
  device sensors, GL/canvas output, or app lifecycle behavior.

## Native Package Proof

- NativeWind/Metro/Tailwind: check config, native smoke, web smoke for
  universal apps, reduced motion, and route unmount.
- Lottie/Rive: prove asset load, playback, pause/restart, unmount, and
  missing-asset fallback on affected platforms.
- Skia/R3F/GL: prove native build, nonblank first render, interaction, frame
  stability enough for context, cleanup, background/foreground, and reduced
  motion.
- Permissions, app links, push, icon, splash, widgets, share extensions, and
  native assets require development-build or release-like proof.

## Audit Triage

For each finding:

1. Verify the file and installed version evidence.
2. Decide whether the finding is still valid for the repo's Expo/RN version.
3. Fix only valid issues unless the user requested broader cleanup.
4. Suppress only with a concrete rule id and package-specific rationale.
5. Rerun the same audit command after remediation.

## Skip Rationale

Accept only specific skip reasons:

- "iOS skipped: Android-only package/config path changed."
- "EAS skipped: local native build compiled the same development profile and
  repo policy does not require cloud proof."
- "Device skipped: simulator covers this JS-only interaction; no native module,
  permission, or platform API changed."
- "E2E skipped: manual native smoke covered the affected route and the flow is
  not release-critical."

Reject vague skips such as "not needed", "too slow", or "no device".
