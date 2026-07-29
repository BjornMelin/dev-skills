# Expo Doctor And Install Checks

Sources:

- https://docs.expo.dev/develop/tools/#expo-doctor
- https://docs.expo.dev/more/expo-cli/#version-validation

Checked at: 2026-06-04.

## Validation Rules

- Expo Doctor diagnoses Expo projects from the project root and checks app
  config, `package.json`, dependency compatibility, config files, and overall
  project health.
- Expo Doctor validates packages against React Native Directory by default and
  checks whether app config properties are synced when native directories
  exist.
- `expo install --check` validates installed packages against versions known to
  work with the app's installed React Native/Expo SDK.
- In CI, `expo install --check` exits non-zero when installed packages are
  outdated or incompatible. Treat this as immutable validation.
- `expo install --fix` changes package versions. Do not use it as validation
  proof unless the user asked for remediation and the diff is reviewed.
- Expo CLI supports bun, npm, pnpm, and yarn detection. Use the target repo's
  wrapper/package manager rather than hard-coding a manager in skill output.

## Native Motion Implications

- Native motion packages need Doctor/install proof before accepting a package
  upgrade, config plugin change, or native binary proof.
- React Native Directory warnings matter for New Architecture readiness. Do not
  suppress unknown, incompatible, or unmaintained native packages without a
  package-specific exception.
- `expo.install.exclude` and `expo.doctor.reactNativeDirectoryCheck.exclude`
  are acceptable only when the repo explains why a package is intentionally out
  of Expo's compatibility lane.
- For app config, native folders, or config plugin changes, combine Doctor with
  `expo config --type prebuild` or a native build.

## Closeout Evidence

Record:

- exact repo command used for install check and Doctor;
- Expo SDK, React Native, Reanimated, and native motion package versions;
- Doctor findings fixed, accepted, or blocked;
- any excluded package and the reason it is safe for this repo.
