# EAS Build Proof

Source: https://docs.expo.dev/build/introduction/

Checked at: 2026-06-04.

## Validation Rules

- EAS Build creates Android and iOS binaries for Expo and React Native projects.
- EAS Build supports platform-specific builds with `--platform android`,
  `--platform ios`, or `--platform all`.
- EAS build profiles in `eas.json` define named build settings for development,
  preview, and production workflows.
- EAS can manage signing credentials or use credentials supplied by the team.
- EAS local builds (`eas build --local`) can provide native compile proof on a
  machine that supports the target platform.

## Native Motion Implications

Use EAS or local native build proof for:

- new native dependencies or native package major upgrades;
- config plugins, permissions, entitlements, app links, icons, splash, widgets,
  share extensions, or push;
- runtimeVersion/OTA semantics, release profile changes, or credentials;
- native modules that must be validated in the same environment as the team or
  release pipeline;
- changes where local platform tooling is unavailable or not representative.

## Closeout Evidence

Record:

- command, platform, and EAS profile;
- local or cloud build;
- build URL or artifact identifier when the repo policy allows it;
- whether the build installed and launched;
- manual or E2E proof performed against the built binary.
