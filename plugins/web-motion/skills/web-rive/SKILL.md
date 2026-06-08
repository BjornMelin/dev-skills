---
name: web-rive
description: >-
  Use this skill for Rive web and React runtime integration, .riv assets, state machines, inputs, lifecycle cleanup, accessibility, remote asset security, and fallback behavior. Trigger on Rive, .riv, state machine input, @rive-app/react-webgl2, @rive-app/webgl2, @rive-app/react-canvas, @rive-app/canvas, Rive web. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web Rive

Rive web and React runtime integration, .riv assets, state machines, inputs, lifecycle cleanup, accessibility, remote asset security, and fallback behavior.

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

- Use web-lottie for Lottie/dotLottie assets.
- Use native-rive for React Native.
- Use Motion/GSAP/CSS when no .riv asset or state machine is involved.

## Workflow

1. Inspect asset ownership, state machine names, inputs, autoplay, and runtime package.
2. Bind inputs through stable component state and cleanup runtime instances.
3. Add fallback and accessible semantics outside canvas.
4. Review URL actions and remote asset policy.

## Gotchas

- Canvas output is not self-describing to assistive tech.
- State machine input names are asset contracts; verify against the asset.
- Remote .riv files and URL actions need explicit allowlisting.

<!-- skill-resources:start -->
## Bundled Resources

- `references/rive-react-runtime.md` - Rive React/runtime lifecycle. Read when using @rive-app/react-webgl2, @rive-app/webgl2, @rive-app/react-canvas, or @rive-app/canvas.
- `references/state-machine-inputs.md` - State machine input contract guide. Read when setting boolean, number, or trigger inputs.
- `references/asset-security-and-fallbacks.md` - Rive asset security, accessibility, and fallback review. Read before shipping remote assets, URL actions, or canvas-only states.
- `references/layout-fit-and-resize.md` - Rive layout, fit, resize, and DPR behavior. Read when sizing a Rive canvas/component, changing artboards, or debugging cropped/blurred assets.
- `references/data-binding-and-events.md` - Rive data binding and event contracts. Read when Rive state machines, inputs, events, or data binding connect to app state.
- `references/docs-rive-react.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rive-state-machine.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rive-web-js.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/implementation-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-rive-audit-report.md` - Audit response/report template.
- `assets/templates/web-rive-review-checklist.md` - Manual review checklist.
- `assets/examples/web-rive-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-rive-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
