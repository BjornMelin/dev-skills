---
name: gsap-timeline
description: Official GSAP skill for timelines — gsap.timeline(), position parameter, nesting, playback. Use when sequencing animations, choreographing keyframes, or when the user asks about animation sequencing, timelines, or animation order (in GSAP or when recommending a library that supports timelines).
license: MIT
---

# GSAP Timeline

## When to Use This Skill

Apply when building multi-step animations, coordinating several tweens in sequence or parallel, or when the user asks about timelines, sequencing, or keyframe-style animation in GSAP.

**Related skills:** For single tweens and eases use **gsap-core**; for scroll-driven timelines use **gsap-scrolltrigger**; for React use **gsap-react**.

## Creating a Timeline

```javascript
const tl = gsap.timeline();
tl.to(".a", { x: 100, duration: 1 })
  .to(".b", { y: 50, duration: 0.5 })
  .to(".c", { opacity: 0, duration: 0.3 });
```

By default, tweens are **appended** one after another. Use the **position parameter** to place tweens at specific times or relative to other tweens.

## Position Parameter

Third argument (or position property in vars) controls placement:

- **Absolute**: `1` — start at 1 second.
- **Relative (default)**: `"+=0.5"` — 0.5s after end; `"-=0.2"` — 0.2s before end.
- **Label**: `"labelName"` — at that label; `"labelName+=0.3"` — 0.3s after label.
- **Placement**: `"<"` — start when recently-added animation starts; `">"` — start when recently-added animation ends (default); `"<0.2"` — 0.2s after recently-added animation start.

Examples:

```javascript
tl.to(".a", { x: 100 }, 0);           // at 0
tl.to(".b", { y: 50 }, "+=0.5");      // 0.5s after last end
tl.to(".c", { opacity: 0 }, "<");     // same start as previous
tl.to(".d", { scale: 2 }, "<0.2");    // 0.2s after previous start
```

## Timeline Defaults

Pass defaults into the timeline so all child tweens inherit:

```javascript
const tl = gsap.timeline({ defaults: { duration: 0.5, ease: "power2.out" } });
tl.to(".a", { x: 100 }).to(".b", { y: 50 }); // both use 0.5s and power2.out
```

## Timeline Options (constructor)

- **paused: true** — create paused; call `.play()` to start.
- **repeat**, **yoyo** — same as tweens; apply to whole timeline.
- **onComplete**, **onStart**, **onUpdate** — timeline-level callbacks.
- **defaults** — vars merged into every child tween.

## Labels

Add and use labels for readable, maintainable sequencing:

```javascript
tl.addLabel("intro", 0);
tl.to(".a", { x: 100 }, "intro");
tl.addLabel("outro", "+=0.5");
tl.to(".b", { opacity: 0 }, "outro");
tl.play("outro");  // start from "outro"
tl.tweenFromTo("intro", "outro"); // pauses the timeline and returns a new Tween that animates the timeline's playhead from intro to outro with no ease.
```

## Nesting Timelines

Timelines can contain other timelines.

```javascript
const master = gsap.timeline();
const child = gsap.timeline();
child.to(".a", { x: 100 }).to(".b", { y: 50 });
master.add(child, 0);
master.to(".c", { opacity: 0 }, "+=0.2");
```

## Controlling Playback

- **tl.play()** / **tl.pause()**
- **tl.reverse()** / **tl.progress(1)** then **tl.reverse()**
- **tl.restart()** — from start.
- **tl.time(2)** — seek to 2 seconds.
- **tl.progress(0.5)** — seek to 50%.
- **tl.kill()** — kill timeline and (by default) its children.

## Official GSAP Best practices

- ✅ Prefer timelines for sequencing
- ✅ Use the **position parameter** (third argument) to place tweens at specific times or relative to labels.
- ✅ Add **labels** with `addLabel()` for readable, maintainable sequencing.
- ✅ Pass **defaults** into the timeline constructor so child tweens inherit duration, ease, etc.
- ✅ Put ScrollTrigger on the timeline (or top-level tween), not on tweens inside a timeline.

## Do Not

- ❌ Chain animations with **delay** when a **timeline** can sequence them; prefer `gsap.timeline()` and the position parameter for multi-step animation.
- ❌ Forget to pass **defaults** (e.g. `defaults: { duration: 0.5, ease: "power2.out" }`) when many child tweens share the same duration or ease.
- ❌ Forget that **duration** on the timeline constructor is not the same as tween duration; timeline “duration” is determined by its children.
- ❌ Nest animations that contain a ScrollTrigger; ScrollTriggers should only be on top-level Tweens/Timelines.

---

## Codex Web Motion Overlay

The upstream GreenSock official skill content above is the primary GSAP
guidance. This local overlay adds Codex-specific progressive-disclosure
resources, static audit scripts, evals, and portable source metadata. Keep GSAP
API behavior aligned with GreenSock's official skill and docs; use this overlay
for validation, local boundaries, and report shape.

### Local Boundaries

- Use gsap-core for one-off tweens.
- Use gsap-scrolltrigger when scroll owns the playhead.
- Use CSS keyframes only for simple fixed loops.

### Local Workflow

1. Model the sequence as labels, relative positions, and defaults.
2. Use position parameters instead of delay chains.
3. Store timeline handles when playback control or cleanup is needed.
4. Verify interruptions and reverse/restart behavior.

### Local Gotchas

- Timeline constructor duration is not child tween duration.
- Nested ScrollTriggers are usually wrong; attach scroll control to the top-level tween/timeline.
- Labels are a maintainability tool, not just comments.

<!-- skill-resources:start -->
### Bundled Resources

- `references/official-source.md` - Official GreenSock timeline skill source. Use this to verify copied upstream timeline behavior.
- `references/position-parameter.md` - Position parameter and labels guide. Use this for sequencing, overlaps, labels, and nested timeline design.
- `references/playback-control.md` - Timeline playback, cleanup, and testing. Use this when pause, reverse, seek, kill, or replay behavior matters.
- `references/sequence-design-taxonomy.md` - Timeline sequence design taxonomy. Read before converting delay chains, staggered entrances, or product choreography into timelines.
- `references/timeline-state-machine-patterns.md` - Timeline state-machine and playback patterns. Read when a timeline is controlled by app state, route state, media queries, or user playback controls.
- `references/docs-timeline-current-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Portable source list and copy policy.
- `references/provenance.json` - Machine-readable provenance and local-resource metadata.
- `scripts/audit.mjs` - Self-contained Codex audit CLI with domain-specific GSAP rules.
- `assets/templates/gsap-timeline-audit-report.md` - GSAP audit response template.
- `assets/templates/gsap-timeline-review-checklist.md` - GSAP manual review checklist.
- `assets/examples/gsap-timeline-starter.js` - Minimal starter fixture/example.
- `evals/trigger-queries.json` - Trigger/near-miss eval set.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

### Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output gsap-timeline-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.
