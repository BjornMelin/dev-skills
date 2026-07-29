---
name: native-validation
description: >-
  Use this skill for Validation for native motion changes: Expo Doctor, expo install --check, EAS/development build risk, Jest/Reanimated setup, RN tests, platform smoke, and audit report closeout. Trigger on Expo Doctor, expo install --check, EAS build validation, native motion validation, Reanimated Jest, development build proof. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Validation

Validation for native motion changes: Expo Doctor, expo install --check, EAS/development build risk, Jest/Reanimated setup, RN tests, platform smoke, and audit report closeout.

## Operating Contract

Use this skill as a compact router plus domain checklist. Load references only
when the current task matches their condition. Do not cite local scrape paths,
machine cache paths, or hidden source locations. Verify API details against the
target repo's installed package versions before editing.

## Source Order

1. Inspect the target repo's installed packages, framework/runtime versions,
   local design tokens, accessibility policy, and existing motion patterns.
2. Use the bundled references below for skill-specific gotchas and copied source
   excerpts.
3. Use official current docs/package source as API truth when local code or
   bundled notes are version-sensitive.

## Decision Boundaries

- Use implementation skills for code changes first.
- Use this skill whenever native package/config/runtime risk is part of the task.
- Do not replace device proof with lint/typecheck for native runtime changes.

## Workflow

1. Identify package/config/native-risk surface.
2. Run audit doctor/scan and repo-native package checks.
3. Choose local test, simulator/device, development build, or EAS proof based on risk.
4. Report commands, skipped checks, and residual risk.

## Gotchas

- Expo-compatible package versions can lag npm latest.
- Expo Go proof is not enough for modules requiring a development build.
- Jest animation tests need Reanimated setup and fake-timer discipline.

<!-- skill-resources:start -->
## Bundled Resources

- `references/expo-doctor-eas-gates.md` - Expo Doctor, install check, and EAS gates. Read to choose validation commands for package/config/native-risk changes.
- `references/motion-package-compatibility.md` - Motion package compatibility matrix. Read before changing Expo/Reanimated/Worklets/Lottie/Skia/Rive packages.
- `references/test-and-device-matrix.md` - Native test and device proof matrix. Read for Jest, RN tests, simulator/device, and development build selection.
- `references/risk-tier-validation-ladder.md` - Native motion risk-tier validation ladder. Read when deciding whether lint/typecheck, Expo Doctor, simulator/device, development build, or EAS proof is required.
- `references/animation-test-harnesses.md` - Animation test harness and fixture selection. Read when writing or choosing tests for Reanimated, Lottie, Rive, Skia, native navigation, or motion accessibility.
- `references/docs-eas-build.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-development-builds.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-doctor.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-new-architecture.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-jest-and-worklets.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rn-testing-overview.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/validation-playbook.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-validation-audit-report.md` - Audit response/report template.
- `assets/templates/native-validation-review-checklist.md` - Manual review checklist.
- `assets/examples/native-validation-starter.md` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-validation-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
