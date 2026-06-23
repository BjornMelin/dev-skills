# Native motion risk-tier validation ladder

Skill: native-validation
Checked at: 2026-06-04

## When To Load

- Read when deciding whether lint/typecheck, Expo Doctor, simulator/device, development build, or EAS proof is required.

## Source Anchors

- https://docs.expo.dev/workflow/diagnostics/
- https://docs.expo.dev/develop/development-builds/introduction/
- https://docs.expo.dev/build/setup/

## Reference Notes

- Tier validation by blast radius: JS-only view motion can often use tests plus simulator proof; package/config/native module/GPU changes require device or development-build proof.
- Expo-compatible package versions can lag npm latest. Use Expo install/check paths before manual version changes.
- Document skipped device proof explicitly with the reason and residual risk.

## Focused Checks

- Classify touched files into JS-only, package/config, native module, GPU/canvas, navigation, or release-risk.
- Run the smallest validation set that actually proves the changed runtime surface.

## Failure Modes

- Using Expo Go as proof for a module requiring a development build.
- Treating `tsc` as validation for native runtime or GPU changes.


## Operating Guidance

Validation for native motion changes: Expo Doctor, expo install --check, EAS/development build risk, Jest/Reanimated setup, RN tests, platform smoke, and audit report closeout.

### Decision Boundaries

- Use implementation skills for code changes first.
- Use this skill whenever native package/config/runtime risk is part of the task.
- Do not replace device proof with lint/typecheck for native runtime changes.

### Workflow Details

1. Identify package/config/native-risk surface.
2. Run audit doctor/scan and repo-native package checks.
3. Choose local test, simulator/device, development build, or EAS proof based on risk.
4. Report commands, skipped checks, and residual risk.

### Gotchas

- Expo-compatible package versions can lag npm latest.
- Expo Go proof is not enough for modules requiring a development build.
- Jest animation tests need Reanimated setup and fake-timer discipline.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
