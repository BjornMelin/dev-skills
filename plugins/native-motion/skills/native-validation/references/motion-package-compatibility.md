# Motion package compatibility matrix

Skill: native-validation
Checked at: 2026-06-04

## When To Load

- Read before changing Expo/Reanimated/Worklets/Lottie/Skia/Rive packages.


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
