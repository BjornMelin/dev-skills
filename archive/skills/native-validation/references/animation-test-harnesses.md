# Animation test harness and fixture selection

Skill: native-validation
Checked at: 2026-06-04

## When To Load

- Read when writing or choosing tests for Reanimated, Lottie, Rive, Skia, native navigation, or motion accessibility.

## Source Anchors

- https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/getting-started
- https://docs.expo.dev/workflow/diagnostics/

## Reference Notes

- Unit tests can prove deterministic mapping, reduced-motion branching, and lifecycle guards; they cannot prove native rendering, GPU output, or gesture feel alone.
- Use small fixtures for animation-state transitions and save device proof for runtime surfaces that need it.
- Fake timers and Reanimated/Jest setup must match the project test harness before relying on timing assertions.

## Focused Checks

- Check Jest/Reanimated setup and whether fake timers are already used.
- Pair tests with simulator/device proof for native modules, canvas/GPU, or navigation transitions.

## Failure Modes

- Testing only snapshots for motion behavior.
- Writing sleeps instead of deterministic animation-state assertions.


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
