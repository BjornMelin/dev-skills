# GSAP Plugin Current Notes

Load this file for import decisions, SSR/lazy-loading boundaries, cleanup, or
plugin-specific gotchas. Use official docs and the installed package version
for final API truth.

## Import Matrix

Prefer ESM imports:

```javascript
import { gsap } from "gsap";
import { ScrollToPlugin } from "gsap/ScrollToPlugin";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { ScrollSmoother } from "gsap/ScrollSmoother";
import { Flip } from "gsap/Flip";
import { Draggable } from "gsap/Draggable";
import { InertiaPlugin } from "gsap/InertiaPlugin";
import { Observer } from "gsap/Observer";
import { SplitText } from "gsap/SplitText";
import { ScrambleTextPlugin } from "gsap/ScrambleTextPlugin";
import { TextPlugin } from "gsap/TextPlugin";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { MorphSVGPlugin } from "gsap/MorphSVGPlugin";
import { MotionPathPlugin } from "gsap/MotionPathPlugin";
import { MotionPathHelper } from "gsap/MotionPathHelper";
import { CustomEase } from "gsap/CustomEase";
import { CustomBounce } from "gsap/CustomBounce";
import { CustomWiggle } from "gsap/CustomWiggle";
import { Physics2DPlugin } from "gsap/Physics2DPlugin";
import { PhysicsPropsPlugin } from "gsap/PhysicsPropsPlugin";
import { PixiPlugin } from "gsap/PixiPlugin";
import { GSDevTools } from "gsap/GSDevTools";

gsap.registerPlugin(Flip, ScrollToPlugin);
```

Use `gsap/dist/<PluginName>` only when a framework or bundler needs UMD files.
Do not use `gsap-trial` or private registry tokens for current GSAP.

## Routing Boundaries

- Use `gsap-scrolltrigger` for ScrollTrigger-only trigger/progress work. This
  skill covers `ScrollSmoother` because it depends on ScrollTrigger and has
  distinct wrapper/content and lifecycle rules.
- Use `gsap-react` for React-only `useGSAP()` and `contextSafe()` questions.
  Keep this skill active when a React task uses a plugin instance such as
  `Draggable`, `SplitText`, `Observer`, or `ScrollSmoother`.
- Use CSS, Lottie, Rive, or Three.js sibling skills when no GSAP plugin API is
  involved.

## SSR and Lazy Loading

Most plugin modules guard direct `window` access, but plugin instances are
browser/DOM objects. Create them only after mount or inside client-only code.

For lazy loading:

```javascript
const [{ gsap }, { Draggable }] = await Promise.all([
  import("gsap"),
  import("gsap/Draggable"),
]);

gsap.registerPlugin(Draggable);
const [draggable] = Draggable.create(node, { type: "x", bounds: container });
```

Keep the created instance in lifecycle-local scope and call `kill()` during
cleanup. In React, prefer `useGSAP()` and `contextSafe()` for GSAP-created work;
pair non-GSAP listeners/observers with explicit cleanup.

## Cleanup Map

| Plugin | Cleanup |
| --- | --- |
| `SplitText` | `split.revert()` restores markup; `split.kill()` also stops `autoSplit` listeners. |
| `Draggable` | `draggable.kill()` removes listeners and disables the instance. |
| `Observer` | `observer.kill()` removes listeners. |
| `ScrollSmoother` | `ScrollSmoother.get()?.kill()` reverts wrapper/content inline CSS and listeners. |
| `MotionPathHelper` | `helper.kill()` removes editing controls. |
| `GSDevTools` | Keep development-only; remove from production bundles unless explicitly gated. |

Cleanup can live in framework lifecycle code, `gsap.context()`/`ctx.revert()`,
or returned cleanup functions. If cleanup is centralized in another file,
document that before suppressing audit findings.

## Gotchas

- ScrollSmoother must be created before downstream ScrollTriggers that depend
  on its transformed content. It uses native page scrolling but transforms the
  content wrapper, so fixed-position elements usually belong outside the
  wrapper/content pair.
- Draggable returns an array from `Draggable.create()`. Destructure or keep the
  array intentionally.
- Draggable inertia needs `InertiaPlugin` registered and `inertia: true`.
- Observer is for input direction and gestures. Use ScrollTrigger when animation
  progress should be tied to scroll position.
- `MotionPathHelper` is the exported helper/plugin name. Do not register
  `MotionPathHelperPlugin` or call `MotionPathHelper.create(target, path, vars)`;
  pass the path inside the vars object.
- SplitText line splitting is sensitive to font loading and container width.
  Use `autoSplit: true` plus returned `onSplit()` animations when splitting
  lines responsively.
- SplitText's default accessibility strategy may hide nested link semantics from
  screen readers. Use `aria: "none"` or `aria: "hidden"` plus an accessible
  duplicate when nested interactive semantics must remain exposed.
- DrawSVG needs an actual stroke (`stroke` and `stroke-width`). It animates
  stroke visibility, not fill.
- MorphSVG first-frame calculations can be expensive for complex paths. Use
  `precompile` only for startup cost; simplify SVGs when per-frame rendering is
  janky.
- MotionPathHelper and GSDevTools are development helpers; avoid shipping them
  in production unless the product explicitly exposes that UI.
