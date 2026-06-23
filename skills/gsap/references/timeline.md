# GSAP Timeline

A timeline is GSAP's sequencer: a container that plays child tweens (and nested timelines) in order or in parallel, with a single playhead you can play, pause, reverse, seek, and scrub. GSAP is fully free and open source (v3.13+); all examples use the GSAP 3 signature `.to(target, vars)` — duration lives inside `vars`.

## Creating a Timeline + Method Chaining

`gsap.timeline()` returns a timeline whose tween methods return the timeline, so calls chain. By default each tween is **appended** after the previous one — no `delay` needed.

```ts
const tl = gsap.timeline();
tl.to(".a", { x: 100, duration: 1 })
  .to(".b", { y: 50, duration: 0.5 })
  .to(".c", { autoAlpha: 0, duration: 0.3 });
```

To overlap, parallelize, or place tweens at exact times, use the **position parameter** instead of stacking delays.

## Position Parameter (in depth)

The position parameter is the optional last argument on `.to()`, `.from()`, `.fromTo()`, `.add()`, `.addLabel()`, `.call()`, `.set()`, and `.addPause()`. It controls where the item is inserted on the timeline.

| Form | Meaning |
| --- | --- |
| _(omitted)_ | Append at the current end of the timeline. |
| `1` (number) | Absolute time — start at exactly 1 second from the timeline's start. |
| `"+=0.5"` | Relative — 0.5s **after** the current end (a gap). |
| `"-=0.2"` | Relative — 0.2s **before** the current end (an overlap). |
| `"label"` | At a named label. A missing label is **created at the end** of the timeline. |
| `"label+=0.3"` | 0.3s after the named label (`"label-=0.3"` for before). |
| `"<"` | At the **start** of the most recently inserted child. |
| `">"` | At the **end** of the most recently inserted child (this is the default append point). |
| `"<0.2"` | Shorthand for `"<+=0.2"` — 0.2s after the recent child's start. |
| `">-0.2"` | 0.2s before the recent child's end. |

Percentage forms (GSAP 3.7+) resolve against the inserting animation's own duration: `"-=25%"`, `"+=50%"`, `"<25%"`, `"<+=25%"`, `"label+=30%"`.

```ts
tl.to(".a", { x: 100 }, 0)          // absolute: at 0s
  .to(".b", { y: 50 }, "+=0.5")     // 0.5s after .a ends
  .to(".c", { autoAlpha: 0 }, "<")  // same start time as .b
  .to(".d", { scale: 2 }, "<0.2")   // 0.2s after .c's start
  .to(".e", { rotation: 90 }, "-=0.1"); // overlap previous end by 0.1s
```

`"<"` and `">"` anchor to the *most recently inserted child* — which may be a callback, label, pause, or nested timeline — not necessarily the end of the whole timeline.

## Timeline Defaults

Pass `defaults` to the constructor so every child tween inherits these vars. Override per-tween only where values differ; this is the canonical way to avoid repeating `duration`/`ease`.

```ts
const tl = gsap.timeline({ defaults: { duration: 0.5, ease: "power2.out" } });
tl.to(".a", { x: 100 })            // 0.5s, power2.out
  .to(".b", { y: 50 })             // 0.5s, power2.out
  .to(".c", { scale: 2, duration: 1 }); // overrides duration only
```

`defaults` commonly carries `duration`, `ease`, `overwrite`, and `stagger`.

## Constructor Options

```ts
const tl = gsap.timeline({
  paused: true,        // start paused; call tl.play() to begin
  repeat: 2,           // play 3 iterations total (count = extra plays)
  repeatDelay: 0.3,    // gap between repeat cycles
  yoyo: true,          // alternate direction on odd cycles
  defaults: { duration: 0.5, ease: "power2.out" },
  onStart: () => {},
  onUpdate: () => {},
  onComplete: () => {},
});
```

Other useful vars: `reversed`, `delay`, `repeatRefresh`, `onRepeat`, `onReverseComplete`, matching `*Params` arrays, `callbackScope`, `smoothChildTiming`, `autoRemoveChildren`, and `scrollTrigger` (valid only on a top-level timeline). A timeline's total duration is determined by its children — there is no meaningful `duration` var on the constructor.

## Labels

Labels mark meaningful UI phases, making sequences readable and edits safer. Reference them as position parameters and as playback targets.

```ts
tl.addLabel("intro", 0);
tl.to(".a", { x: 100 }, "intro");
tl.addLabel("outro", "+=0.5");      // relative to current end
tl.to(".b", { autoAlpha: 0 }, "outro");

tl.play("outro");                   // jump to "outro" and play
tl.seek("intro");                   // move playhead to a label (no play)
tl.tweenFromTo("intro", "outro");   // pause, then tween the playhead intro->outro (no ease)
```

`tweenFromTo(from, to, vars)` and `tweenTo(position, vars)` return a control tween that scrubs the playhead. For tweens built in advance for later user interaction, pass `{ immediateRender: false }`. Related label helpers: `currentLabel()`, `nextLabel(time)`, `previousLabel(time)`, `removeLabel(label)`.

## Nesting Timelines

Build sub-sequences as their own timelines, then drop them into a master with `.add()`. This composes complex choreography from reusable, testable parts.

```ts
const buildCard = (sel: string) =>
  gsap.timeline().to(sel, { y: 0, autoAlpha: 1 }).to(`${sel} .cta`, { scale: 1 }, "<0.1");

const master = gsap.timeline({ defaults: { duration: 0.4, ease: "power2.out" } });
master
  .add(buildCard(".card-1"))
  .add(buildCard(".card-2"), "-=0.2")  // overlap with previous child
  .to(".footer", { autoAlpha: 1 }, "+=0.1");
```

`.add()` accepts a Tween, Timeline, label string, callback, or an array of those, each with a position parameter.

## Controlling Playback

```ts
tl.play();          // play forward from current position
tl.pause();         // pause at current position
tl.resume();        // resume in current direction
tl.reverse();       // play backward
tl.restart();       // jump to start and play
tl.seek(2);         // move playhead to 2s (or a label string)
tl.time(2);         // get/set current-iteration time in seconds
tl.progress(0.5);   // get/set 0..1 within the current iteration
tl.timeScale(2);    // 2x speed; 0.5 = half speed; negative reverses
tl.kill();          // kill the timeline and (by default) its children
```

Use `totalTime()` / `totalProgress()` when controls must account for repeats and repeat delays; `time()` / `progress()` operate on the current iteration only. `tl.revert()` kills the timeline **and** restores the inline styles it created — prefer it over `kill()` when the owner won't restore state separately.

## React & Next.js Note

In React, never create a timeline in render, in event handlers, or inside a `requestAnimationFrame`/`useFrame` loop — that leaks timelines on every render. Build it once inside `useGSAP()` (from `@gsap/react`), scoped to a container ref, and store the handle in a ref when you need to drive playback later. `useGSAP()` records every animation created inside it and reverts them automatically on unmount or dependency change.

```tsx
"use client";
import { useRef } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";

export function Reveal() {
  const root = useRef<HTMLDivElement>(null);
  const tl = useRef<gsap.core.Timeline | null>(null);

  useGSAP(
    () => {
      tl.current = gsap
        .timeline({ paused: true, defaults: { duration: 0.5, ease: "power2.out" } })
        .from(".headline", { y: 24, autoAlpha: 0 })
        .from(".sub", { y: 16, autoAlpha: 0 }, "<0.1")
        .from(".cta", { scale: 0.9, autoAlpha: 0 }, "-=0.2");
    },
    { scope: root } // selectors resolve within root; auto-cleanup on unmount
  );

  return (
    <div ref={root}>
      <h1 className="headline">Ship faster</h1>
      <p className="sub">Sequenced with one timeline.</p>
      <button className="cta" onClick={() => tl.current?.restart()}>
        Replay
      </button>
    </div>
  );
}
```

Keep authoritative playback state in either the app or the timeline, not both: map a UI toggle cleanly to `play`/`reverse`/`seek`/`progress`. Rebuild a timeline only after killing/reverting the previous instance, and rebuild at the same boundary where layout ownership changes (e.g. inside `gsap.matchMedia()` for responsive breakpoints).

## Realistic Sequenced Example

A hero entrance: container fades in, heading rises, items stagger, CTA pops, with a label so the CTA segment can be replayed on demand.

```tsx
"use client";
import { useRef } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";

export function Hero() {
  const root = useRef<HTMLDivElement>(null);
  const tl = useRef<gsap.core.Timeline | null>(null);

  useGSAP(
    () => {
      tl.current = gsap
        .timeline({ defaults: { duration: 0.6, ease: "power3.out" } })
        .from(".hero", { autoAlpha: 0, duration: 0.3 })
        .from(".hero__title", { y: 40, autoAlpha: 0 }, "<0.1")
        .from(".hero__item", { y: 24, autoAlpha: 0, stagger: 0.12 }, "-=0.3")
        .addLabel("cta")
        .from(".hero__cta", { scale: 0.85, autoAlpha: 0, ease: "back.out(1.7)" }, "cta");
    },
    { scope: root }
  );

  return (
    <section className="hero" ref={root}>
      <h1 className="hero__title">Choreograph it once</h1>
      <ul>
        <li className="hero__item">Position parameter</li>
        <li className="hero__item">Labels</li>
        <li className="hero__item">Nesting</li>
      </ul>
      <button
        className="hero__cta"
        onClick={() => tl.current?.tweenFromTo("cta", tl.current.duration())}
      >
        Get started
      </button>
    </section>
  );
}
```

## Pitfalls / Do-not

- **Don't chain with `delay` to sequence.** A timeline appends children automatically; use the position parameter (`"+=0.5"`, `"<"`, labels) for overlaps and gaps instead of smuggling per-tween delays that obscure order.
- **Don't use the GSAP 2 signature.** It's `.to(target, { x: 100, duration: 1 })`, never `.to(target, 1, { x: 100 })`. Duration is a var.
- **Don't treat constructor `duration` as child duration.** Timeline length comes from its children; put `duration` in `defaults` or each tween's vars.
- **Don't nest ScrollTriggered tweens.** Attach `scrollTrigger` to the top-level timeline (or top-level tween), never to a child tween inside a timeline.
- **Don't recreate timelines every render.** Build once in `useGSAP()`/`gsap.context()`; rebuild only after killing/reverting the old instance, or you leak live handles.
- **Don't animate the same properties with CSS transitions** that a timeline's children control — the two fight and produce jank.
- **Mind `from`/`fromTo` immediate render.** Stacked from-type tweens on the same target can flash start values; set `immediateRender: false` on later ones (and on advance-built `tweenFromTo` scrub tweens).
- **A missing label is created at the end** of the timeline — convenient, but it silently hides typos in position strings.

## Related references

- [Core API](./core.md)
- [React & Next.js](./react-nextjs.md)
- [ScrollTrigger](./scrolltrigger.md)
- [Plugins](./plugins.md)
- [Recipes](./recipes.md)
