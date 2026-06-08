---
name: web-lottie
description: >-
  Use this skill for lottie-web, dotLottie web components, animation JSON/dotLottie assets, player lifecycle, cleanup, renderer choice, accessibility, and asset validation. Trigger on lottie-web, dotLottie, .lottie, Lottie JSON, After Effects animation, Bodymovin. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web Lottie

lottie-web, dotLottie web components, animation JSON/dotLottie assets, player lifecycle, cleanup, renderer choice, accessibility, and asset validation.

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

- Use native-lottie for React Native.
- Use Rive for interactive state machines.
- Use CSS/WAAPI for simple UI motion that does not need designer-authored assets.

## Workflow

1. Inspect asset format, player package, renderer, autoplay/loop, and hosting path.
2. Create and destroy player instances at the owner boundary.
3. Respect reduced motion and provide non-canvas semantics.
4. Validate asset size, remote URLs, and event listeners.

## Gotchas

- Canvas-rendered animation needs external accessible text or labels.
- Remote animation URLs need CSP/cache/security review.
- Looping/autoplay assets require reduced-motion and pause behavior.

<!-- skill-resources:start -->
## Bundled Resources

- `references/lottie-player-lifecycle.md` - lottie-web player lifecycle. Read when creating, updating, or destroying lottie-web animation instances.
- `references/dotlottie-web-component.md` - dotLottie web component and worker notes. Read when using .lottie assets, dotLottie players, workers, or web components.
- `references/asset-accessibility-security.md` - Asset accessibility and security review. Read before accepting remote assets, canvas-only output, autoplay loops, or URL actions.
- `references/authoring-and-export-compatibility.md` - Authoring and export compatibility. Read when accepting designer-authored Lottie JSON/dotLottie assets or debugging mismatch between After Effects preview and runtime output.
- `references/runtime-event-contracts.md` - Runtime events, markers, and playback contracts. Read when code controls Lottie playback, segments, markers, events, or synchronization with app state.
- `references/docs-dotlottie-web.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-lottie-web-load-animation-options.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-lottie-web-readme.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/implementation-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-lottie-audit-report.md` - Audit response/report template.
- `assets/templates/web-lottie-review-checklist.md` - Manual review checklist.
- `assets/examples/web-lottie-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-lottie-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
