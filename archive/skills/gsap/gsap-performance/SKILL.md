---
name: gsap-performance
description: Official GSAP skill for performance — prefer transforms, avoid layout thrashing, will-change, batching. Use when optimizing GSAP animations, reducing jank, or when the user asks about animation performance, FPS, or smooth 60fps.
license: MIT
---

# GSAP Performance

## When to Use This Skill

Apply when optimizing GSAP animations for smooth 60fps, reducing layout/paint cost, or when the user asks about performance, jank, or best practices for fast animations.

**Related skills:** Build animations with **gsap-core** (transforms, autoAlpha) and **gsap-timeline**; for ScrollTrigger performance see **gsap-scrolltrigger**.

## Prefer Transform and Opacity

Animating **transform** (`x`, `y`, `scaleX`, `scaleY`, `rotation`, `rotationX`, `rotationY`, `skewX`, `skewY`) and **opacity** keeps work on the compositor and avoids layout and most paint. Avoid animating layout-heavy properties when a transform can achieve the same effect.

- ✅ Prefer: **x**, **y**, **scale**, **rotation**, **opacity**.
- ❌ Avoid when possible: **width**, **height**, **top**, **left**, **margin**, **padding** (they trigger layout and can cause jank).

GSAP’s **x** and **y** use transforms (translate) by default; use them instead of **left**/**top** for movement.

## will-change

Use **will-change** in CSS on elements that will animate. It hints the browser to promote the layer.

```css
will-change: transform;
```

## Batch Reads and Writes

GSAP batches updates internally. When mixing GSAP with direct DOM reads/writes or layout-dependent code, avoid interleaving reads and writes in a way that causes repeated layout thrashing. Prefer doing all reads first, then all writes (or let GSAP handle the writes in one go).

## Many Elements (Stagger, Lists)

- Use **stagger** instead of many separate tweens with manual delays when the animation is the same; it’s more efficient.
- For long lists, consider **virtualization** or animating only visible items; avoid creating hundreds of simultaneous tweens if it causes jank.
- Reuse timelines where possible; avoid creating new timelines every frame.

## Frequently updated properties (e.g. mouse followers)

Prefer **gsap.quickTo()** for properties that are updated often (e.g. mouse-follower x/y). It reuses a single tween instead of creating new tweens on each update. 

```javascript
let xTo = gsap.quickTo("#id", "x", { duration: 0.4, ease: "power3" }),
    yTo = gsap.quickTo("#id", "y", { duration: 0.4, ease: "power3" });

document.querySelector("#container").addEventListener("mousemove", (e) => {
  xTo(e.pageX);
  yTo(e.pageY);
});
```

## ScrollTrigger and Performance

- **pin: true** promotes the pinned element; pin only what’s needed.
- **scrub** with a small value (e.g. `scrub: 1`) can reduce work during scroll; test on low-end devices.
- Call **ScrollTrigger.refresh()** only when layout actually changes (e.g. after content load), not on every resize; debounce when possible.

## Reduce Simultaneous Work

- Pause or kill off-screen or inactive animations when they’re not visible (e.g. when the user navigates away).
- Avoid animating huge numbers of properties on many elements at once; simplify or sequence if needed.

## Best practices

- ✅ Animate **transform** and **opacity**; use **will-change** in CSS only on elements that animate.
- ✅ Use **stagger** instead of many separate tweens with manual delays when the animation is the same.
- ✅ Use **gsap.quickTo()** for frequently updated properties (e.g. mouse followers).
- ✅ Clean up or kill off-screen animations; call **ScrollTrigger.refresh()** when layout changes, debounced when possible.

## Do Not

- ❌ Animate **width**/ **height**/ **top**/ **left** for movement when **x**/ **y**/ **scale** can achieve the same look.
- ❌ Set **will-change** or **force3D** on every element “just in case”; use for elements that are actually animating.
- ❌ Create hundreds of overlapping tweens or ScrollTriggers without testing on low-end devices.
- ❌ Ignore cleanup; stray tweens and ScrollTriggers keep running and can hurt performance and correctness.

---

## Codex Web Motion Overlay

The upstream GreenSock official skill content above is the primary GSAP
guidance. This local overlay adds Codex-specific progressive-disclosure
resources, static audit scripts, evals, and portable source metadata. Keep GSAP
API behavior aligned with GreenSock's official skill and docs; use this overlay
for validation, local boundaries, and report shape.

### Local Boundaries

- Use web-three-r3f or typegpu for GPU/canvas rendering performance.
- Use web-css-animations for CSS-only transition audits.
- Use gsap-scrolltrigger for scroll scene semantics.

### Local Workflow

1. Find hot paths and animated properties.
2. Classify layout, paint, composite, and JavaScript costs.
3. Run audit scan and inspect high-confidence findings.
4. Recommend transform/opacity, batching, throttling, or engine changes only with evidence.

### Local Gotchas

- will-change is a scoped hint, not a global optimization.
- ScrollTrigger refresh calls after layout changes need ordering, not random timeouts.
- Infinite tweens need reduced-motion and cleanup behavior.

<!-- skill-resources:start -->
### Bundled Resources

- `references/official-source.md` - Official GreenSock performance skill source. Use this to verify upstream performance guidance.
- `references/property-cost-matrix.md` - Property cost and rendering matrix. Use this when classifying transform, opacity, layout, paint, filter, shadow, and text animation risk.
- `references/scroll-performance.md` - ScrollTrigger and scroll workload review. Use this for pinned scenes, scrubbed timelines, refresh timing, and scroll callback costs.
- `references/profiling-playbook.md` - GSAP profiling and evidence playbook. Read when a GSAP issue is described as jank, dropped frames, layout thrash, or slow scroll.
- `references/layer-budget-and-will-change.md` - Layer budget and will-change discipline. Read before adding transform hacks, force3D, will-change, or GPU promotion advice.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Portable source list and copy policy.
- `references/provenance.json` - Machine-readable provenance and local-resource metadata.
- `scripts/audit.mjs` - Self-contained Codex audit CLI with domain-specific GSAP rules.
- `assets/templates/gsap-performance-audit-report.md` - GSAP audit response template.
- `assets/templates/gsap-performance-review-checklist.md` - GSAP manual review checklist.
- `assets/examples/gsap-performance-starter.md` - Minimal starter fixture/example.
- `evals/trigger-queries.json` - Trigger/near-miss eval set.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

### Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output gsap-performance-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.
