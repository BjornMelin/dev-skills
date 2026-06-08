# Reanimated Jest And Worklets

Sources:

- https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/getting-started/
- https://docs.swmansion.com/react-native-reanimated/docs/guides/testing/

Checked at: 2026-06-04.

## Validation Rules

- Reanimated 4 works only with the React Native New Architecture.
- Reanimated 4 requires the `react-native-worklets` dependency.
- Expo projects should validate Expo-compatible Reanimated/worklets versions
  before adding or changing Babel config.
- React Native CLI projects with explicit Babel config need
  `react-native-worklets/plugin` in `babel.config.js`, listed last.
- Reanimated Jest support requires
  `require('react-native-reanimated').setUpTests()` in the Jest setup file.
- Time-based animation tests should use Jest fake timers and advance timers to
  the assertion point.

## Native Motion Implications

- A Reanimated major upgrade is at least Level 2 and becomes Level 3 when
  package pins or native binaries change.
- `runOnJS` in new Reanimated 4 code deserves review against the installed
  Reanimated/worklets boundary.
- Animated style assertions are useful regression tests, but they do not prove
  native platform rendering, gesture handling, or New Architecture linkage.
- When a repo has both Jest and Reanimated but no setup file calling
  `setUpTests`, audit the test coverage before trusting animation assertions.

## Closeout Evidence

Record:

- `react-native-reanimated` and `react-native-worklets` versions;
- Babel plugin state, including whether Expo owns it;
- Jest setup file and timer strategy when animation tests are present;
- native build/smoke proof when native runtime behavior changed.
