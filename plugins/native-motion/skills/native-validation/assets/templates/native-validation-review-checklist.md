# Native Validation Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Identify package/config/native-risk surface.
- [ ] Run audit doctor/scan and repo-native package checks.
- [ ] Choose local test, simulator/device, development build, or EAS proof based on risk.
- [ ] Report commands, skipped checks, and residual risk.
- [ ] Check gotcha: Expo-compatible package versions can lag npm latest.
- [ ] Check gotcha: Expo Go proof is not enough for modules requiring a development build.
- [ ] Check gotcha: Jest animation tests need Reanimated setup and fake-timer discipline.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
