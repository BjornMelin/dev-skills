# native-accessibility-performance Source Ledger

Checked at: 2026-06-04

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, copied excerpts, generated evals, examples, and audit
rules. It intentionally does not reference local scrape paths or machine cache
locations.

## Primary Sources

- React Native accessibility/performance docs and Expo docs.
- Software Mansion React Native skill notes where license-gated.
- Agent Skills specification and best practices.

## Bundled Source Excerpts

- `references/docs-reanimated-performance.md`
- `references/docs-rn-accessibility.md`
- `references/docs-rn-accessibilityinfo.md`
- `references/docs-rn-performance.md`
- `references/docs-expo-animation.md`
- `references/docs-expo-haptics.md`
- `references/docs-reanimated-accessibility.md`
- `references/docs-reanimated-source-notes.md`

## Tailored Reference Files

- `references/reduced-motion-haptics-policy.md` - Reduced motion and haptics policy
- `references/native-performance-audit.md` - Native performance audit guide
- `references/manual-device-checks.md` - Manual iOS/Android proof checklist
- `references/gesture-feedback-and-motion-sickness.md` - Gesture feedback and motion sickness review
- `references/frame-budget-instrumentation.md` - Native frame budget instrumentation

## Local Additions

- `references/provenance.json` records source URLs, package facts, and copy policy.
- `scripts/audit.mjs` provides the repeatable static audit CLI for this skill.
- `assets/templates/` contains output templates and review checklists.
- `assets/examples/` contains small starter examples or fixtures.
- `evals/` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
