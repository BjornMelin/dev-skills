# Rive React Native Runtime Notes

Use this file for package choice, native setup, file loading, asset handling,
Expo constraints, error handling, and lifecycle. Use
`rive-state-machine.md` for data binding and state-machine contract design.

## Source Snapshot

- Upstream docs: https://rive.app/docs/runtimes/react-native/react-native
- Migration guide: https://rive.app/docs/runtimes/react-native/migration-guide
- Expo docs: https://rive.app/docs/runtimes/react-native/adding-rive-to-expo
- File loading: https://rive.app/docs/runtimes/react-native/loading-rive-files
- Props/errors/ref methods:
  - https://rive.app/docs/runtimes/react-native/props
  - https://rive.app/docs/runtimes/react-native/error-handling
  - https://rive.app/docs/runtimes/react-native/rive-ref-methods
- Loading assets/fonts/audio:
  - https://rive.app/docs/runtimes/react-native/loading-assets
  - https://rive.app/docs/runtimes/react-native/fonts
  - https://rive.app/docs/runtimes/react-native/playing-audio
- Source repo: https://github.com/rive-app/rive-nitro-react-native
- Package metadata inspected: `@rive-app/react-native` 0.4.10.

## Package Choice

- New work: use `@rive-app/react-native` and
  `react-native-nitro-modules`.
- Legacy runtime: `rive-react-native`. Keep only when the repo is intentionally
  pinned or a migration is explicitly out of scope.
- Current package metadata for `@rive-app/react-native` 0.4.10 declares:
  - `react-native-nitro-modules` peer range `>=0.35.0 <0.36`
  - native SDK defaults: iOS `6.20.4`, Android `11.4.0`
- Official prose docs can lag package metadata. Use the exact installed or
  fetched package metadata for dependency ranges and native runtime versions.

## Runtime Requirements

Check the installed package and official docs before changing dependencies. At
this snapshot, upstream docs/source require or recommend:

- React Native 0.78 or later; 0.79+ improves Android error messages.
- Expo SDK 53 or later for Expo apps.
- iOS 15.1 or later.
- Android SDK 24 or later.
- Xcode 16.4 or later.
- JDK 17 or later.
- Nitro Modules installed and compatible with the exact package peer range.

For Expo, Rive includes custom native code. Use a development build, prebuild,
or EAS lane. Expo Go is not validation for this runtime.

## Expo Setup

- Use the target repo's package policy rather than hard-coding a package manager
  command.
- Install or verify the app's development-build lane before claiming runtime
  proof.
- Expo SDK 53 Android builds can need `expo-build-properties` and
  `expo-custom-agp` so Android uses compile SDK 36 and Android Gradle Plugin
  8.9.1+.
- Set iOS deployment target to 15.1+ when the repo's Expo/RN baseline does not
  already do it.
- Rebuild native apps after adding Rive, Nitro Modules, native resource `.riv`
  files, native SDK version overrides, or Expo build-property plugins.

## Core API Shape

Prefer the new runtime API:

```tsx
import {
  Fit,
  RiveView,
  useRive,
  useRiveFile,
  useViewModelInstance,
} from '@rive-app/react-native';

const { riveFile, isLoading, error } = useRiveFile(
  require('./assets/status.riv')
);
const { riveViewRef, setHybridRef } = useRive();
const { instance } = useViewModelInstance(riveFile, {
  artboardName: RIVE_CONTRACT.artboard,
  viewModelName: RIVE_CONTRACT.viewModel,
  instanceName: RIVE_CONTRACT.instance,
  onInit: (vmi) => {
    vmi.numberProperty(RIVE_CONTRACT.properties.progress)?.set(0);
  },
});

return riveFile ? (
  <RiveView
    file={riveFile}
    hybridRef={setHybridRef}
    artboardName={RIVE_CONTRACT.artboard}
    stateMachineName={RIVE_CONTRACT.stateMachine}
    dataBind={instance}
    fit={Fit.Contain}
    onError={handleRiveError}
    style={styles.rive}
  />
) : null;
```

Key props and methods:

- `RiveView` renders a loaded `RiveFile` through the `file` prop.
- `artboardName` and `stateMachineName` select non-default contracts.
- `dataBind` accepts a `ViewModelInstance`, `DataBindMode`, or a binding
  selector such as `{ byName }`.
- `autoPlay` defaults to true. Make autoplay/loop ownership explicit.
- `fit`, `alignment`, and `layoutScaleFactor` control layout. `Fit.Layout` has
  layout-specific scaling/alignment behavior.
- `useRive()` provides a Nitro hybrid ref for `play`, `pause`, `reset`,
  `awaitViewReady`, `playIfNeeded`, `bindViewModelInstance`, and
  `getViewModelInstance`.
- `onError` should exist at wrapper boundaries that can fail from missing files,
  wrong artboards, wrong state machines, or data-binding contract drift.

## Loading `.riv` Files

`useRiveFile` supports local Metro assets, URLs, native resource names,
`{ uri }` objects, and `ArrayBuffer`.

Prefer `require('./asset.riv')` for local app assets because Metro can bundle
the file and Expo OTA updates can carry `.riv` changes. Add Metro support:

```js
const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);
config.resolver.assetExts.push('riv');

module.exports = config;
```

Native resource-name loading requires adding the `.riv` file to iOS bundle
resources and Android `res/raw`, then rebuilding native apps.

Remote URL and `ArrayBuffer` loading require explicit loading UI, retry,
caching, offline behavior, privacy/integrity policy, and failure fallback.
Prefer local assets unless remote delivery is a product requirement.

## Referenced Assets, Fonts, And Audio

Rive assets can be embedded, hosted, or referenced:

- Embedded assets are simplest but increase `.riv` size.
- Hosted assets add network dependency.
- Referenced assets require a runtime `referencedAssets` mapping.

Use full referenced asset keys exported by the Rive editor. Keep keys in a
typed contract map:

```tsx
const { riveFile } = useRiveFile(require('./profile.riv'), {
  referencedAssets: {
    [RIVE_CONTRACT.assets.avatar]: { source: { uri: avatarUrl } },
    [RIVE_CONTRACT.assets.fontInter]: {
      source: require('./fonts/Inter-594377.ttf'),
    },
  },
});
```

For dynamic image properties, load images through Rive runtime utilities when
the installed API exposes them, update the view-model property, then call
`playIfNeeded()` if the visual must advance while settled.

## Error Handling

Handle both hook and view errors:

- `useRiveFile` exposes loading/error state for file loading.
- `RiveView` exposes `onError` for runtime contract and render errors.
- Error sources include file-not-found, malformed files, incorrect artboards,
  incorrect state machines, missing view-model instances, missing referenced
  assets, and incorrect legacy state-machine input names.

User-facing copy should be app/domain-specific. Do not surface raw native,
Rive, package, or internal contract names to end users unless the app is a
developer tool.

## Native SDK Version Overrides

Avoid overrides unless a documented upstream issue requires them. If unavoidable:

- Record the upstream issue/release note and target native SDK versions.
- For vanilla RN, update `ios/Podfile.properties.json` and
  `android/gradle.properties`.
- For Expo, use config plugins such as `withPodfileProperties` and
  `withGradleProperties`.
- Re-check and retest overrides on every `@rive-app/react-native` upgrade.

## Cleanup And Lifecycle

- `useRiveFile` owns and disposes loaded `RiveFile` instances on hook cleanup.
- View-model property hooks dispose native property handles.
- Legacy event/input methods and custom refs need manual listener cleanup.
- Pause/reset on route exit when the animation owns audio, heavy loops, or
  state that must not continue off-screen.
- Stale async callbacks can still fire after unmount; gate async asset loading
  and remote fetch updates accordingly.
