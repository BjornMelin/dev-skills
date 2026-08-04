---
name: motion
description: "Motion/CSS animation for React, Vue, Base UI, and vanilla JS via the Motion AI Kit: transitions, springs, easing curves, CSS spring generation, Motion API docs search, and MotionScore performance audits. Use to implement web animation or to grade and fix animation jank. Covers motion mcp, MotionScore, spring bounce, easing curve, linear() easing, transition preview, jank audit, will-change, and compositor cost."
license: MIT
metadata:
  version: "1.0.0"
  upstream: "Motion AI Kit — https://motion.dev/docs/ai-kit"
  argument-hint: "[subcommand or question, e.g. 'audit src/Modal.tsx', 'spring bounce 0.3', 'see easeOut', 'how do I animate a list']"
---

# Motion

Improve the animation capabilities of the agent.

-   [Animation best practices](best-practices/index.md): "Animate this button", "Fade this layer in", "Animate this Vue component". This includes platform-specific best practices for vanilla JS, React, Vue, Base UI and Radix. Contains advice for both Motion and CSS animation.
-   [Documentation and examples search](codex/index.md): "What options does X have", "How does X work", "Use X (specific Motion API) to do Y", "Show me an example of X", "Make a X (i.e. carousel, ticker, modal etc)"
-   [CSS spring generation](css-spring/index.md): "Generate a CSS spring for a bounce of 0.5 and duration of 0.3s", "Make a bouncy spring [in a CSS context]"
-   [MotionScore performance audit](performance-audit/index.md): "Audit src/Modal.tsx for jank", "Runtime audit of homepage", "Is this code janky: [code snippet]", "Grade the performance of this site: [URL]" - or if you, the agent, wish to profile a site or codebase without a user prompt, you can proactively run audits and report findings.
-   [Transition visualisation](transition-preview/index.md): "Show me the curve for easeOut", "Visualise a spring with bounce 0.5 and duration 0.3s"

## Where this skill sits

This skill owns the Motion library and the Motion AI Kit: its APIs, spring and easing
generation, and MotionScore grading. Adjacent owners:

| Concern | Owner |
| --- | --- |
| Motion craft judgment — which curve, how long, whether it earns its place | `emil-design-eng` |
| Surfaces and icons: radius, shadows, image outlines, icon states | `better-ui` |
| Finished CSS-only `t-*` transition recipes with motion tokens | `transitions-dev` |
| GSAP, ScrollTrigger, SplitText, Flip | `gsap` |
| Reduced-motion requirements and other accessibility rules | `better-accessibility` |
| Native (Expo / React Native / Reanimated) motion | `expo-motion` |
| Cross-stack motion direction, 3D, and motion-token architecture | the `design-motion` plugin |

Use this skill for Motion API truth, generated spring and easing values, and performance
grading. Take taste decisions from `emil-design-eng` and accessibility requirements from
`better-accessibility`.

## Tools this skill uses

The Motion AI Kit MCP server exposes these; call them by their fully qualified names:

| Tool | Use for |
| --- | --- |
| `mcp__motion__search-motion-codex` | Motion API docs and working examples. Returns resource **links** — you must read each relevant link to get the content |
| `mcp__motion__generate-css-spring` | A CSS `linear()` easing plus duration from spring parameters |
| `mcp__motion__generate-css-bounce-easing` | A CSS bounce easing |
| `mcp__motion__visualise-spring` | Preview a spring curve |
| `mcp__motion__visualise-cubic-bezier` | Preview a named or custom bezier |
| `mcp__motion__devtools-status`, `mcp__motion__get-devtools-update` | Motion DevTools session state |

**MotionScore is a CLI, not an MCP tool**: run `npx motionscore <url> --agent`. It needs a
reachable URL, so it works only when a dev server or public origin is available.

### Runtime audit or static review?

| Situation | Do this |
| --- | --- |
| A URL or dev-server address is available, or the user said "runtime" | `npx motionscore <url> --agent`, then use its selector-keyed findings and `Source hint` lines to jump straight to the code |
| A file, directory, or code snippet is named, with no URL | Static audit per `performance-audit/index.md`; classify every animation by render-pipeline tier |
| Both are available and the surface matters | Run both and triangulate: the runtime report is authoritative for what actually executes, static analysis for what exists but did not run |
| The runtime audit fails (no dev server, navigation timeout, browser launch error) | Fall back to static discovery and say so; never report a failed audit as a clean one |

Concurrent-animation counts, GPU pressure, and the `prefers-reduced-motion` / flashing-content
checks come only from the runtime path. Property-tier classification comes from either.

## If a required Motion MCP tool is unavailable

This skill ships with an MCP server. Some tools expect this server to be running. If you attempt to use a tool that requires the MCP server and it is not found, tell the user:

> This capability requires the Motion AI Kit. Install it from **https://motion.dev/docs/ai-kit**.

Then fall back to the guidance in the relevant capability directory where possible (e.g. `best-practices/` and `performance-audit/` work without any MCP tool).

## Upstream

This is a tracked fork of the Motion AI Kit skill. The capability directories
(`best-practices/`, `codex/`, `css-spring/`, `performance-audit/`, `transition-preview/`) are
upstream content and are deliberately left in their original layout so updates can be merged
cleanly. Local additions are confined to this file: the frontmatter, **Where this skill sits**,
**Tools this skill uses**, and **Upstream**. Re-merge from `https://motion.dev/docs/ai-kit`
and reapply those sections.
