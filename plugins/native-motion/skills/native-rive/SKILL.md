---
name: native-rive
description: >-
  Use this skill for Rive React Native/Nitro runtime, .riv assets, state machines, inputs, asset loading, platform compatibility, accessibility, and iOS/Android proof. Trigger on @rive-app/react-native, Rive React Native, .riv native, Rive state machine native, rive-nitro. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Rive

Rive React Native/Nitro runtime, .riv assets, state machines, inputs, asset loading, platform compatibility, accessibility, and iOS/Android proof.

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

- Use web-rive for browser runtime.
- Use native-lottie for Lottie/dotLottie assets.
- Use native-motion-core for code-driven Reanimated motion.

## Workflow

1. Check package/runtime compatibility and native build requirements.
2. Verify asset path, state machine name, input names, and autoplay.
3. Map app state to inputs with cleanup on unmount.
4. Validate iOS/Android build/rendering and accessibility fallback.

## Gotchas

- State machine names and inputs are asset contracts.
- Native Rive runtime may require development build proof, not only Expo Go.
- Canvas-like output needs surrounding accessible semantics.

<!-- skill-resources:start -->
## Bundled Resources

- `references/rive-native-state-machines.md` - Native Rive state-machine contract. Read when binding inputs, triggers, or state machine names.
- `references/rive-native-asset-loading.md` - Rive native asset loading and lifecycle. Read for .riv bundling, runtime setup, and cleanup.
- `references/nitro-platform-validation.md` - Nitro/platform validation notes. Read before closing native build/runtime changes.
- `references/rive-file-caching-and-assets.md` - Native Rive file caching and asset loading. Read when `.riv` files are bundled, cached, loaded remotely, reused across views, or include out-of-band assets.
- `references/state-machine-input-protocol.md` - Native Rive state-machine input protocol. Read when app state drives boolean, number, or trigger inputs in a native Rive state machine.
- `references/docs-rive-react-native.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rive-state-machine.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-rive-audit-report.md` - Audit response/report template.
- `assets/templates/native-rive-review-checklist.md` - Manual review checklist.
- `assets/examples/native-rive-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-rive-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
