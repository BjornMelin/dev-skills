# GSAP with React & Next.js

The official, TypeScript-first way to run GSAP in React and Next.js (App Router) is the `useGSAP()` hook from `@gsap/react`: it scopes selectors, reverts every animation and ScrollTrigger on unmount, and is safe under React Strict Mode and SSR. This reference covers the canonical setup, the full `useGSAP` API, refs and scoping, `contextSafe()`, the `gsap.context()` fallback, and Next.js client-boundary and route-change rules.

> GSAP is 100% free, including every plugin (ScrollTrigger, SplitText, MorphSVG, Draggable, etc.), in the public `gsap` package on npm — no license gate, trial, or private registry. `@gsap/react` is GreenSock's official React integration. Use `gsap` v3.13+ with `@gsap/react`.

## Table of Contents

- [Install](#install)
- [Canonical Setup (`lib/gsap.ts`)](#canonical-setup-libgsapts)
- [Why `useGSAP()` over `useEffect()`](#why-usegsap-over-useeffect)
- [The `useGSAP()` API](#the-usegsap-api)
- [Refs and Scope](#refs-and-scope)
- [`contextSafe()` for Event Handlers & Async](#contextsafe-for-event-handlers--async)
- [`gsap.context()` in `useEffect` (Fallback)](#gsapcontext-in-useeffect-fallback)
- [SSR and the `'use client'` Boundary](#ssr-and-the-use-client-boundary)
- [React Strict Mode](#react-strict-mode)
- [Next.js App Router Specifics](#nextjs-app-router-specifics)
- [Pages Router Variant](#pages-router-variant)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## Install

```bash
npm i gsap @gsap/react
```

Both packages are free and ship their own TypeScript types — no `@types/*` needed.

## Canonical Setup (`lib/gsap.ts`)

Register `useGSAP` and any plugins **once** behind a client boundary, then import the configured `gsap` everywhere. Centralizing registration avoids "plugin not registered" errors and double-registration churn.

```ts
// lib/gsap.ts
'use client';

import { gsap } from 'gsap';
import { useGSAP } from '@gsap/react';
import { ScrollTrigger } from 'gsap/ScrollTrigger';
// import any other plugins you use, e.g.:
// import { SplitText } from 'gsap/SplitText';

// Register exactly once. useGSAP MUST be registered before any useGSAP call.
gsap.registerPlugin(useGSAP, ScrollTrigger /*, SplitText */);

// Optional: project-wide defaults
gsap.defaults({ ease: 'power2.out', duration: 0.6 });

export { gsap, useGSAP, ScrollTrigger };
```

Consume it from a client component, driving the hook with a `scope` ref:

```tsx
// components/Hero.tsx
'use client';

import { useRef } from 'react';
import { gsap, useGSAP } from '@/lib/gsap';

export function Hero() {
  const container = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      // Selectors are scoped to `container`, so `.title`/`.item`
      // only match elements inside this component.
      gsap.from('.title', { y: 40, opacity: 0 });
      gsap.from('.item', { opacity: 0, stagger: 0.1 });
    },
    { scope: container },
  );

  return (
    <div ref={container}>
      <h1 className="title">Hello GSAP</h1>
      <ul>
        <li className="item">One</li>
        <li className="item">Two</li>
        <li className="item">Three</li>
      </ul>
    </div>
  );
}
```

> Importing `useGSAP` from `@/lib/gsap` (which re-exports it) is equivalent to importing it from `@gsap/react`; both reference the same hook. The point of the module is the single `registerPlugin` call.

## Why `useGSAP()` over `useEffect()`

`useGSAP()` is a drop-in replacement for `useEffect`/`useLayoutEffect` that wraps your setup in a `gsap.context()` and reverts it automatically. Prefer it whenever `@gsap/react` is available.

- **Automatic cleanup via `gsap.context()`.** Every tween, timeline, and ScrollTrigger created synchronously inside the callback is tracked. On unmount (and on dependency changes when configured), the context reverts them — killing animations and removing inline styles GSAP added. No manual `kill()`/`revert()` bookkeeping.
- **Strict Mode safety.** React 18+ Strict Mode mounts, unmounts, and remounts components in development. Because `useGSAP` reverts on cleanup and re-runs setup cleanly, the double-invoke does not leave duplicate or "stuck" animations.
- **SSR-safe.** Internally `useGSAP` uses an `useIsomorphicLayoutEffect` (a `useLayoutEffect` on the client, `useEffect` on the server) so it never warns during server render and never executes GSAP on the server. Your callback runs **only** in the browser, after the DOM exists.

## The `useGSAP()` API

```tsx
useGSAP(callback, dependenciesOrConfig?);
```

The second argument is optional. By default, `useGSAP` passes an **empty dependency array** to its internal effect, so the callback runs once after mount and is not re-run on every render. You can pass either a dependency array (like `useEffect`) or a config object:

```tsx
const { context, contextSafe } = useGSAP(
  () => {
    gsap.to('.box', { x: endX });
  },
  {
    dependencies: [endX],   // dependency array (optional)
    scope: container,       // ref/element to scope selectors (optional, recommended)
    revertOnUpdate: true,   // revert + re-run cleanup every time a dependency changes
  },
);
```

Config object fields:

- **`scope`** — a ref (or element) that limits all selector strings in the callback to that subtree. Strongly recommended (see below).
- **`dependencies`** — array controlling when the hook re-synchronizes. `[]` (default) = run once; `undefined` = run on every render; `[a, b]` = re-run when `a` or `b` change.
- **`revertOnUpdate`** — when `true`, the context is reverted and the cleanup function runs **every time** the hook re-synchronizes (i.e. when a dependency changes), not just on unmount. Use it when changing dependencies should rebuild the scene from scratch; leave it `false` (default) when you want new animations to layer on existing state.

Return value: `{ context, contextSafe }`. `context` is the underlying `gsap.Context`; `contextSafe` wraps later-created callbacks (below).

**Registration:** call `gsap.registerPlugin(useGSAP)` before any `useGSAP` runs. The canonical `lib/gsap.ts` does this once. (Registering `useGSAP` is what lets it integrate with GSAP's context system; it is harmless and idempotent.)

## Refs and Scope

Use refs so GSAP targets the actual DOM nodes that React renders, and always provide a `scope` so selector strings cannot leak outside the component.

- **Scope selectors with a ref.** With `useGSAP`, pass the container ref as `scope`. Then `gsap.to('.box', ...)` only matches `.box` inside that container — never a `.box` elsewhere on the page.
- **Target a single node with a ref directly** when you do not need selector strings:

```tsx
const boxRef = useRef<HTMLDivElement>(null);

useGSAP(() => {
  gsap.to(boxRef.current, { rotation: 360 });
}, { scope: boxRef });

return <div ref={boxRef} className="box" />;
```

- **Multiple elements:** scope a container ref and select children (`'.item'`), or use `gsap.utils.toArray<HTMLElement>('.item')` inside the scoped callback. An array of refs also works when the set is fixed.

Never animate by an unscoped selector string in component code — it can match elements outside the component and break on re-render.

## `contextSafe()` for Event Handlers & Async

The context only tracks animations created **synchronously** while the callback runs. Animations created **later** — in event handlers, timeouts, promises, observers, or route callbacks — are not in the context and will not be reverted on unmount unless you wrap them with `contextSafe()`.

`contextSafe()` returns a function that creates its animations inside the component's context (so they are tracked and scoped) and no-ops after unmount (avoiding React state/DOM warnings). Also remove any listeners you added, in the cleanup return.

```tsx
'use client';

import { useRef } from 'react';
import { gsap, useGSAP } from '@/lib/gsap';

export function Toggle() {
  const container = useRef<HTMLDivElement>(null);
  const goodRef = useRef<HTMLButtonElement>(null);

  useGSAP(
    (_context, contextSafe) => {
      // ✅ Created synchronously -> tracked & reverted automatically.
      gsap.from(goodRef.current, { opacity: 0 });

      // ❌ DANGER: created in a handler AFTER setup runs, NOT wrapped.
      // Not added to the context, so it is never reverted, and the
      // listener below is never removed -> leaks across renders.
      // goodRef.current!.addEventListener('click', () => {
      //   gsap.to(goodRef.current, { y: 100 });
      // });

      // ✅ Wrapped in contextSafe -> tracked, scoped, safe after unmount.
      const onClick = contextSafe!(() => {
        gsap.to(goodRef.current, { rotation: '+=180' });
      });

      goodRef.current?.addEventListener('click', onClick);

      // 👍 Always remove listeners you added, in cleanup.
      return () => {
        goodRef.current?.removeEventListener('click', onClick);
      };
    },
    { scope: container },
  );

  return (
    <div ref={container}>
      <button ref={goodRef}>Spin</button>
    </div>
  );
}
```

You can also use the returned `contextSafe` outside the callback (e.g. to build a stable handler passed to JSX `onClick`); just ensure refs are populated before it runs. Store handles for animations that may outlive the synchronous callback so you can verify the component can unmount mid-flight without a leaked tween.

## `gsap.context()` in `useEffect` (Fallback)

When `@gsap/react` is not available, or you genuinely need `useEffect`'s trigger semantics, wrap setup in `gsap.context()` and **always** call `ctx.revert()` in the cleanup. Skipping `revert()` leaks animations and lets them update detached nodes after unmount.

```tsx
'use client';

import { useEffect, useRef } from 'react';
import { gsap } from '@/lib/gsap';

export function Fallback() {
  const container = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.to('.box', { x: 100 });
      gsap.from('.item', { opacity: 0, stagger: 0.1 });
    }, container); // 2nd arg scopes selectors to this node

    return () => ctx.revert(); // ALWAYS revert
  }, []);

  return (
    <div ref={container}>
      <div className="box" />
      <div className="item" />
    </div>
  );
}
```

`useGSAP` is essentially this pattern with the boilerplate (context creation, scoping, revert, Strict Mode handling, SSR-safe effect) handled for you — prefer it.

## SSR and the `'use client'` Boundary

GSAP reads and writes the DOM, so it must run only in the browser.

- **Mark client components with `'use client'`.** Any component that imports `gsap`, `useGSAP`, or a plugin must be a client component. Server components must never call `gsap.*` or `ScrollTrigger.*`.
- **Never run GSAP during server render.** Keep all GSAP calls inside `useGSAP` (or `useEffect`) so they execute after hydration. Do not call animations at module top level of a component or in the render body.
- **Importing is fine; executing is not.** Top-level `import { gsap } from 'gsap'` is safe — bundlers tree-shake and `gsap` does not touch `window` on import. The rule is: do not *execute* GSAP/ScrollTrigger during server render. If bundle size matters, you can dynamically `import()` heavy plugins inside the effect.
- **`useGSAP` is SSR-safe by design** via its isomorphic layout effect, so it will not emit the `useLayoutEffect` SSR warning.

## React Strict Mode

In development, React Strict Mode runs effects twice (mount → cleanup → mount) to surface impure side effects. With GSAP:

- **`useGSAP` handles it correctly:** cleanup reverts the first run before the second runs, so you do not get duplicated or frozen animations. This is the main reason to prefer it over a hand-rolled `useEffect`.
- **In the `useEffect` fallback,** the `ctx.revert()` cleanup is what makes the double-invoke safe — without it you would see doubled tweens in dev.
- Treat unexpected duplicate setup as a signal that scoping or cleanup is wrong, not as something to silence by disabling Strict Mode.
- **Avoid React state updates inside high-frequency GSAP callbacks** (`onUpdate`, ScrollTrigger `onUpdate`/`scrub`). Animate values directly or write to refs; per-frame `setState` causes re-render storms.

## Next.js App Router Specifics

- **Client boundary per animated component.** Put `'use client'` at the top of every component that uses GSAP. The `lib/gsap.ts` module is also `'use client'` so registration happens on the client.
- **ScrollTrigger and route changes.** App Router navigations unmount the old page's components, so `useGSAP` reverts their ScrollTriggers automatically — provided each ScrollTrigger was created inside `useGSAP` (or a `contextSafe` callback). ScrollTriggers created outside the context (e.g. globally, or in an un-wrapped handler) will *not* be cleaned up and will accumulate across navigations.
- **Refresh ScrollTrigger after layout settles.** When new route content changes page height after navigation/transition, call `ScrollTrigger.refresh()` so trigger positions recalculate. Doing this inside `useGSAP` after creating your triggers is usually enough; if a transition library animates layout, refresh once it completes.

```tsx
'use client';

import { useRef } from 'react';
import { gsap, useGSAP, ScrollTrigger } from '@/lib/gsap';

export function ScrollSection() {
  const container = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      gsap.to('.panel', {
        xPercent: -100,
        scrollTrigger: {
          trigger: '.panel',
          scrub: true,
          pin: true,
        },
      });

      // Recalculate positions once content is laid out.
      ScrollTrigger.refresh();
      // Cleanup (reverting these triggers) is automatic on route change/unmount.
    },
    { scope: container },
  );

  return (
    <div ref={container}>
      <section className="panel">Panel</section>
    </div>
  );
}
```

- **Per-component scope, not document scope.** Scope each component's selectors to its own ref so animations from one route do not target nodes from another.

## Pages Router Variant

The same code works in the Pages Router. The difference is there is no `'use client'` directive — every component already runs on the client after hydration, and you simply keep GSAP calls inside `useGSAP`/`useEffect` (never in `getServerSideProps`/`getStaticProps` or top-level module execution). Keep the single `registerPlugin` in a shared module (without `'use client'`) and import the configured `gsap` from it.

## Pitfalls / Do-not

- ❌ **Don't use unscoped selector strings.** Always pass `scope` to `useGSAP` (or as the 2nd arg to `gsap.context()`) so `.box` cannot match elements outside the component.
- ❌ **Don't create animations after setup without `contextSafe()`.** Event-handler/timeout/promise/observer tweens are not tracked by the context and will leak unless wrapped — and remove the listeners in cleanup.
- ❌ **Don't skip cleanup in the `useEffect` fallback.** Always `return () => ctx.revert()`; otherwise animations leak and update detached nodes (and double in Strict Mode).
- ❌ **Don't run GSAP or ScrollTrigger during SSR / in server components.** No `gsap.*` in render bodies, module top level, or server components. Keep it inside `useGSAP`/`useEffect`.
- ❌ **Don't forget `gsap.registerPlugin(useGSAP, …)`** before any `useGSAP` call or plugin use — register once in `lib/gsap.ts`.
- ❌ **Don't call `setState` in per-frame GSAP/ScrollTrigger callbacks.** Use refs or animate values directly.
- ❌ **Don't rebuild the whole scene on every render.** Use a precise `dependencies` array; set `revertOnUpdate: true` only when a dependency change should truly reset the animation.
- ❌ **Don't add license/premium/trial framing.** All plugins are free in the public `gsap` package (v3.13+).

## Related references

- [Core API](./core.md)
- [Timeline](./timeline.md)
- [ScrollTrigger](./scrolltrigger.md)
- [Plugins](./plugins.md)
- [Performance](./performance.md)
- [Recipes](./recipes.md)
