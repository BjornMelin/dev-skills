# native-validation Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- Expo Doctor, EAS Build, Expo development build, and SDK compatibility docs.
- React Native testing and Reanimated Jest docs.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-eas-build.md`
- `references/docs-expo-development-builds.md`
- `references/docs-expo-doctor.md`
- `references/docs-expo-new-architecture.md`
- `references/docs-reanimated-jest-and-worklets.md`
- `references/docs-rn-testing-overview.md`
- `references/validation-playbook.md`

## Tailored Reference Files

- `references/expo-doctor-eas-gates.md` - Expo Doctor, install check, and EAS gates
- `references/motion-package-compatibility.md` - Motion package compatibility matrix
- `references/test-and-device-matrix.md` - Native test and device proof matrix
- `references/risk-tier-validation-ladder.md` - Native motion risk-tier validation ladder
- `references/animation-test-harnesses.md` - Animation test harness and fixture selection

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
