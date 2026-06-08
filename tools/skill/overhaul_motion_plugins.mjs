#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';

const repoRoot = path.resolve(import.meta.dirname, '..', '..');
const checkedAt = '2026-06-04';

const pluginRoots = {
  web: path.join(repoRoot, 'plugins', 'web-motion'),
  native: path.join(repoRoot, 'plugins', 'native-motion'),
};

const gsapSkills = new Set([
  'gsap-core',
  'gsap-frameworks',
  'gsap-performance',
  'gsap-plugins',
  'gsap-react',
  'gsap-scrolltrigger',
  'gsap-timeline',
  'gsap-utils',
]);

const sharedNegativeQueries = [
  'make this API route return paginated JSON from Postgres',
  'fix this Zod schema for a nested form payload',
  'write a SQL migration for an orders table',
  'debug why Clerk middleware redirects on localhost',
  'convert this CSV file into YAML',
  'add Open Graph metadata to this marketing page',
  'optimize a server function cold start without changing UI animation',
  'write a unit test for a pure date formatter',
  'create a Dockerfile for this Python service',
  'review this payment webhook signature verification',
];

const skills = {
  web: [
    skill({
      name: 'gsap-core',
      title: 'GSAP Core',
      family: 'GSAP',
      short: 'Official GSAP core plus Codex audit routing',
      focus: 'Core GSAP tweens, transforms, eases, staggers, matchMedia, accessibility, and DOM/SVG tween review.',
      triggers: ['gsap.to()', 'gsap.from()', 'fromTo', 'stagger', 'autoAlpha', 'matchMedia', 'GSAP tween'],
      boundaries: ['Use CSS for simple declarative state transitions.', 'Use gsap-timeline for multi-step choreography.', 'Use gsap-react when React lifecycle owns the target nodes.'],
      workflow: ['Inspect installed GSAP version and framework ownership.', 'Prefer official transform aliases and explicit eases/durations.', 'Add matchMedia or reduced-motion handling for nonessential motion.', 'Run the audit CLI and verify findings manually.'],
      gotchas: ['Do not animate raw transform strings when GSAP aliases express the same effect.', 'Set immediateRender intentionally when stacking from/fromTo tweens.', 'Store returned tween handles when playback control or cleanup is needed.'],
      references: [
        ref('official-source.md', 'Official GreenSock skill source and license gate', ['Use this when verifying which upstream GSAP skill text was copied and how the MIT skill license differs from the GSAP package license.']),
        ref('codex-overlay.md', 'Codex-local GSAP implementation overlay', ['Use this when adapting official GSAP examples to repo-specific validation, reduced motion, and static audit expectations.']),
        ref('audit-rules.md', 'Core tween audit rules', ['Use this when reviewing transform aliases, layout-property animation, repeat/yoyo behavior, and cleanup ownership.']),
      ],
      exampleExt: 'js',
      example: `import { gsap } from 'gsap';\n\nconst tween = gsap.to('.card', {\n  autoAlpha: 1,\n  y: 0,\n  duration: 0.45,\n  ease: 'power2.out',\n});\n\nexport function cleanup() {\n  tween.kill();\n}\n`,
    }),
    skill({
      name: 'gsap-frameworks',
      title: 'GSAP Frameworks',
      family: 'GSAP',
      short: 'Official GSAP framework lifecycle guidance',
      focus: 'GSAP in Vue, Svelte, Nuxt, Astro, and vanilla component lifecycles with scoped selectors and cleanup.',
      triggers: ['GSAP Vue', 'GSAP Svelte', 'Nuxt GSAP', 'framework lifecycle', 'component cleanup'],
      boundaries: ['Use gsap-react for React and Next.js React code.', 'Use vanilla GSAP core when no framework lifecycle is involved.', 'Keep this skill explicit-only for non-React frameworks.'],
      workflow: ['Identify framework lifecycle hooks and mount/unmount boundaries.', 'Scope selectors to the component root.', 'Register plugins once at module/app boundary.', 'Revert contexts, kill timelines, and clean listeners on unmount.'],
      gotchas: ['Do not let selector text escape a component root.', 'Hydration/client-only boundaries matter in SSR frameworks.', 'Framework transitions may already own enter/exit timing; avoid duplicate animation owners.'],
      references: [
        ref('official-source.md', 'Official GreenSock framework skill source and license gate', ['Use this when verifying copied upstream framework guidance.']),
        ref('framework-lifecycle.md', 'Vue, Svelte, Nuxt, and Astro lifecycle notes', ['Use this when deciding where GSAP setup and cleanup should live.']),
        ref('component-scoping.md', 'Scoped selectors and cleanup contracts', ['Use this when selectors, refs, or framework component boundaries are involved.']),
      ],
      exampleExt: 'svelte',
      example: `<script>\n  import { onMount } from 'svelte';\n  import { gsap } from 'gsap';\n\n  let root;\n\n  onMount(() => {\n    const ctx = gsap.context(() => {\n      gsap.from('.item', { y: 16, autoAlpha: 0, stagger: 0.06 });\n    }, root);\n    return () => ctx.revert();\n  });\n</script>\n\n<section bind:this={root}>\n  <slot />\n</section>\n`,
      implicit: false,
    }),
    skill({
      name: 'gsap-performance',
      title: 'GSAP Performance',
      family: 'GSAP',
      short: 'Official GSAP performance review plus audit rules',
      focus: 'GSAP performance audits: transform/opacity, layout thrash, ScrollTrigger batching, repeat loops, will-change, and frame pressure.',
      triggers: ['GSAP performance', 'jank', 'layout thrash', 'will-change', 'too many tweens', 'slow ScrollTrigger'],
      boundaries: ['Use web-three-r3f or typegpu for GPU/canvas rendering performance.', 'Use web-css-animations for CSS-only transition audits.', 'Use gsap-scrolltrigger for scroll scene semantics.'],
      workflow: ['Find hot paths and animated properties.', 'Classify layout, paint, composite, and JavaScript costs.', 'Run audit scan and inspect high-confidence findings.', 'Recommend transform/opacity, batching, throttling, or engine changes only with evidence.'],
      gotchas: ['will-change is a scoped hint, not a global optimization.', 'ScrollTrigger refresh calls after layout changes need ordering, not random timeouts.', 'Infinite tweens need reduced-motion and cleanup behavior.'],
      references: [
        ref('official-source.md', 'Official GreenSock performance skill source', ['Use this to verify upstream performance guidance.']),
        ref('property-cost-matrix.md', 'Property cost and rendering matrix', ['Use this when classifying transform, opacity, layout, paint, filter, shadow, and text animation risk.']),
        ref('scroll-performance.md', 'ScrollTrigger and scroll workload review', ['Use this for pinned scenes, scrubbed timelines, refresh timing, and scroll callback costs.']),
      ],
      exampleExt: 'md',
      example: `# GSAP performance note\n\n- Prefer x/y/scale/rotation/autoAlpha for hot-path UI motion.\n- Treat width, height, top, left, filter, box-shadow, and text layout as measured exceptions.\n- Verify reduced-motion and cleanup before closeout.\n`,
    }),
    skill({
      name: 'gsap-plugins',
      title: 'GSAP Plugins',
      family: 'GSAP',
      short: 'Official GSAP plugin registration and usage',
      focus: 'GSAP plugin registration, public package imports, Flip, Draggable, Observer, MotionPath, ScrollTo, SVG/text/ease plugins, and plugin boundary review.',
      triggers: ['GSAP plugin', 'Flip', 'Draggable', 'Observer', 'MotionPath', 'ScrollToPlugin', 'SplitText', 'plugin registration'],
      boundaries: ['Use gsap-scrolltrigger for ScrollTrigger-specific scene control.', 'Respect GSAP package license and plugin availability.', 'Do not invent imports for plugins absent from the public package or local dependency set.'],
      workflow: ['Check installed gsap version and plugin availability.', 'Register plugins exactly once in the runtime boundary.', 'Keep plugin setup out of hot render paths.', 'Verify lifecycle cleanup and license/package constraints.'],
      gotchas: ['Plugin imports differ by plugin and package availability; verify before generating code.', 'SplitText-style text effects need accessibility and reduced-motion fallbacks.', 'Flip needs measured before/after state ownership; do not mix with separate layout animators.'],
      references: [
        ref('official-source.md', 'Official GreenSock plugin skill source', ['Use this to verify upstream plugin coverage and license.']),
        ref('plugin-availability.md', 'Plugin import and availability matrix', ['Use this before writing imports, registration, or premium-plugin examples.']),
        ref('plugin-lifecycle.md', 'Plugin setup, cleanup, and accessibility review', ['Use this for Flip, Draggable, Observer, SVG, and text plugin implementation review.']),
      ],
      exampleExt: 'js',
      example: `import { gsap } from 'gsap';\nimport { Flip } from 'gsap/Flip';\n\ngsap.registerPlugin(Flip);\n\nexport function flipToNextState(target, mutateDom) {\n  const state = Flip.getState(target);\n  mutateDom();\n  return Flip.from(state, { duration: 0.35, ease: 'power2.out' });\n}\n`,
    }),
    skill({
      name: 'gsap-react',
      title: 'GSAP React',
      family: 'GSAP',
      short: 'Official @gsap/react lifecycle guidance',
      focus: 'React and Next.js GSAP integration using @gsap/react, useGSAP(), refs, scoped contexts, cleanup, SSR/client boundaries, and dependency-safe animation setup.',
      triggers: ['@gsap/react', 'useGSAP', 'GSAP React', 'Next.js GSAP', 'gsap.context', 'React animation cleanup'],
      boundaries: ['Use gsap-core for framework-free tween semantics.', 'Use gsap-scrolltrigger when scroll scene semantics dominate.', 'Use web-motion-react if the implementation should use Motion instead of GSAP.'],
      workflow: ['Confirm React/client boundary and installed @gsap/react.', 'Register useGSAP where the project pattern expects plugin registration.', 'Scope selectors to refs or context.', 'Revert contexts and kill manually owned animations on cleanup.'],
      gotchas: ['Do not run GSAP setup in a server component.', 'Avoid unscoped string selectors in component code.', 'Dependency changes should either rebuild inside useGSAP safely or use contextSafe callbacks.'],
      references: [
        ref('official-source.md', 'Official GreenSock React skill source', ['Use this to verify upstream @gsap/react guidance.']),
        ref('react-lifecycle.md', 'useGSAP, refs, context, and cleanup', ['Use this for React/Next lifecycle and scoped selector decisions.']),
        ref('next-client-boundary.md', 'Next.js and SSR/client boundary notes', ['Use this when WebGL/browser APIs or GSAP code crosses server/client boundaries.']),
      ],
      exampleExt: 'tsx',
      example: `'use client';\n\nimport { useRef } from 'react';\nimport { gsap } from 'gsap';\nimport { useGSAP } from '@gsap/react';\n\ngsap.registerPlugin(useGSAP);\n\nexport function Intro() {\n  const root = useRef<HTMLElement>(null);\n  useGSAP(() => {\n    gsap.from('.item', { y: 16, autoAlpha: 0, stagger: 0.05 });\n  }, { scope: root });\n  return <section ref={root} />;\n}\n`,
    }),
    skill({
      name: 'gsap-scrolltrigger',
      title: 'GSAP ScrollTrigger',
      family: 'GSAP',
      short: 'Official ScrollTrigger scene guidance',
      focus: 'GSAP ScrollTrigger scroll-linked animation, pinning, scrub, trigger callbacks, refresh/invalidation, responsive matchMedia scenes, and cleanup.',
      triggers: ['ScrollTrigger', 'scroll-linked animation', 'scrub', 'pinning', 'start end markers', 'ScrollTrigger.refresh'],
      boundaries: ['Use CSS scroll-driven animations only when native browser support and declarative semantics fit.', 'Use gsap-timeline for non-scroll sequencing.', 'Use web-three-r3f for 3D scroll scenes.'],
      workflow: ['Identify scroll container, trigger, start/end, pin, scrub, and responsive ownership.', 'Attach ScrollTrigger to a top-level tween or timeline.', 'Plan refresh ordering after fonts/images/layout changes.', 'Verify resize, route unmount, reduced motion, and mobile scroll.'],
      gotchas: ['Do not put ScrollTriggers inside child tweens of a nested timeline.', 'Pinned scenes affect layout and need refresh proof.', 'Route transitions must kill/revert triggers or scope them to a context.'],
      references: [
        ref('official-source.md', 'Official GreenSock ScrollTrigger skill source', ['Use this to verify upstream ScrollTrigger guidance.']),
        ref('scene-geometry.md', 'Trigger geometry, pin, scrub, and refresh rules', ['Use this when start/end, pin spacing, markers, refresh, or layout changes are involved.']),
        ref('scroll-validation.md', 'Scroll scene validation checklist', ['Use this for route unmount, mobile scroll, resize, reduced-motion, and visual proof.']),
      ],
      exampleExt: 'js',
      example: `import { gsap } from 'gsap';\nimport { ScrollTrigger } from 'gsap/ScrollTrigger';\n\ngsap.registerPlugin(ScrollTrigger);\n\nconst timeline = gsap.timeline({\n  scrollTrigger: {\n    trigger: '.panel',\n    start: 'top center',\n    end: 'bottom center',\n    scrub: true,\n  },\n});\n\ntimeline.to('.panel', { x: 120, ease: 'none' });\n`,
    }),
    skill({
      name: 'gsap-timeline',
      title: 'GSAP Timeline',
      family: 'GSAP',
      short: 'Official GSAP timeline sequencing',
      focus: 'GSAP timelines, sequencing, position parameter, labels, nesting, playback controls, defaults, and timeline review.',
      triggers: ['gsap.timeline', 'position parameter', 'timeline labels', 'sequence animations', 'animation order', 'playhead'],
      boundaries: ['Use gsap-core for one-off tweens.', 'Use gsap-scrolltrigger when scroll owns the playhead.', 'Use CSS keyframes only for simple fixed loops.'],
      workflow: ['Model the sequence as labels, relative positions, and defaults.', 'Use position parameters instead of delay chains.', 'Store timeline handles when playback control or cleanup is needed.', 'Verify interruptions and reverse/restart behavior.'],
      gotchas: ['Timeline constructor duration is not child tween duration.', 'Nested ScrollTriggers are usually wrong; attach scroll control to the top-level tween/timeline.', 'Labels are a maintainability tool, not just comments.'],
      references: [
        ref('official-source.md', 'Official GreenSock timeline skill source', ['Use this to verify copied upstream timeline behavior.']),
        ref('position-parameter.md', 'Position parameter and labels guide', ['Use this for sequencing, overlaps, labels, and nested timeline design.']),
        ref('playback-control.md', 'Timeline playback, cleanup, and testing', ['Use this when pause, reverse, seek, kill, or replay behavior matters.']),
      ],
      exampleExt: 'js',
      example: `import { gsap } from 'gsap';\n\nconst tl = gsap.timeline({ defaults: { duration: 0.4, ease: 'power2.out' } });\ntl.addLabel('intro')\n  .from('.title', { y: 16, autoAlpha: 0 }, 'intro')\n  .from('.body', { y: 12, autoAlpha: 0 }, '<0.08')\n  .to('.cta', { scale: 1 }, '+=0.1');\n`,
    }),
    skill({
      name: 'gsap-utils',
      title: 'GSAP Utils',
      family: 'GSAP',
      short: 'Official gsap.utils helper guidance',
      focus: 'gsap.utils helpers such as clamp, mapRange, normalize, interpolate, random, snap, toArray, selector, wrap, pipe, unitize, and function-based value review.',
      triggers: ['gsap.utils', 'clamp', 'mapRange', 'normalize', 'snap', 'wrap', 'selector', 'function-based values'],
      boundaries: ['Use plain JavaScript helpers when GSAP is not already part of the animation stack.', 'Use gsap-core when helper values feed tweens.', 'Use gsap-scrolltrigger when helpers map scroll progress.'],
      workflow: ['Identify whether the helper should return a reusable function or immediate value.', 'Keep unit handling explicit.', 'Scope selector helpers in component code.', 'Test boundary inputs and cyclic values.'],
      gotchas: ['mapRange and normalize operate on numbers, not unit strings.', 'Omitting the final value returns a reusable function; this is often the intended pattern.', 'selector(scope) prevents cross-component targeting mistakes.'],
      references: [
        ref('official-source.md', 'Official GreenSock utils skill source', ['Use this to verify copied upstream helper guidance.']),
        ref('numeric-mapping.md', 'Numeric mapping and snapping reference', ['Use this for clamp, mapRange, normalize, interpolate, snap, wrap, and wrapYoyo.']),
        ref('selectors-and-units.md', 'Selector scoping and unit helper notes', ['Use this for toArray, selector, unitize, getUnit, and function-based values.']),
      ],
      exampleExt: 'js',
      example: `import { gsap } from 'gsap';\n\nconst progressToRotation = gsap.utils.pipe(\n  gsap.utils.clamp(0, 1),\n  gsap.utils.mapRange(0, 1, -12, 12),\n  gsap.utils.snap(0.5),\n);\n\nexport const rotationForProgress = (progress) => progressToRotation(progress);\n`,
    }),
    skill({
      name: 'typegpu',
      title: 'TypeGPU',
      family: 'WebGPU',
      short: 'Type-safe WebGPU with TypeGPU',
      focus: 'TypeGPU schemas, typed buffers/textures, shader functions, pipelines, WebGPU capability checks, and CPU/GPU resource ownership.',
      triggers: ['TypeGPU', 'tgpu', 'd.struct', 'use gpu', 'unplugin-typegpu', 'typed WebGPU', 'shader functions'],
      boundaries: ['Do not trigger for raw WebGPU without TypeGPU imports.', 'Use web-three-r3f for Three/R3F scenes.', 'Use native-three-r3f or native-skia for React Native GPU surfaces.'],
      workflow: ['Check installed typegpu, unplugin-typegpu, @webgpu/types, tsover, and browser/runtime support.', 'Define schemas before resources and shader signatures.', 'Keep root/device/resource ownership explicit.', 'Validate unsupported-browser fallback, reduced-motion/static quality, and GPU cleanup.'],
      gotchas: ['A d.* schema is the CPU layout, GPU layout, and TypeScript type source of truth.', 'TypeScript shader functions require unplugin-typegpu; WGSL-only usage may not.', 'Do not allocate buffers, textures, bind groups, or pipelines per frame unless measured and cached.'],
      references: [
        ref('typegpu-codex-playbook.md', 'Codex workflow for TypeGPU tasks', ['Read before writing TypeGPU app code, shader functions, or compute/render pipelines.']),
        ref('shader-resource-boundaries.md', 'Shader/resource ownership rules', ['Read when code mixes CPU buffers, GPU resources, schemas, bind groups, and shader functions.']),
        ref('webgpu-runtime-validation.md', 'Browser WebGPU validation and fallbacks', ['Read for secure context, adapter/device, unsupported browser, reduced motion, and teardown proof.']),
      ],
      sourceDocs: ['setup.md', 'types.md', 'shaders.md', 'pipelines.md', 'textures.md', 'matrices.md'],
      exampleExt: 'ts',
      example: `import tgpu, { d } from 'typegpu';\n\nconst Particle = d.struct({\n  position: d.vec2f,\n  velocity: d.vec2f,\n});\n\nexport async function createParticles(count: number) {\n  const root = await tgpu.init();\n  const particles = root.createBuffer(d.arrayOf(Particle, count)).$usage('storage');\n  return { root, particles };\n}\n`,
    }),
    skill({
      name: 'web-css-animations',
      title: 'Web CSS Animations',
      family: 'CSS',
      short: 'CSS transitions, keyframes, and reduced motion',
      focus: 'Browser CSS transitions, keyframes, scroll-driven animations, registered properties, discrete transitions, reduced motion, and performance-safe CSS motion.',
      triggers: ['CSS transition', '@keyframes', 'animation-timeline', 'prefers-reduced-motion', '@starting-style', 'transition-behavior', 'CSS animation'],
      boundaries: ['Use CSS first for two-state UI motion.', 'Move to WAAPI when an Animation object or seeking is needed.', 'Move to GSAP for complex imperative choreography.'],
      workflow: ['Identify the state driver and animated properties.', 'Use explicit transition-property lists and product motion tokens.', 'Add reduced-motion behavior beside the motion.', 'Guard new CSS with @supports or local browser policy.'],
      gotchas: ['transition: all hides expensive accidental properties.', 'Unregistered custom properties animate discretely.', 'animation shorthand resets animation-timeline, so set timeline after shorthand.'],
      references: [
        ref('css-motion-field-guide.md', 'CSS transition/keyframe field guide', ['Read before implementing ordinary CSS state motion.']),
        ref('browser-support-and-accessibility.md', 'Support, @supports, and reduced-motion notes', ['Read for newer CSS features or accessibility review.']),
        ref('property-performance-matrix.md', 'CSS property performance matrix', ['Read for transform, opacity, layout, paint, filter, shadow, and text animation risk.']),
      ],
      sourceDocs: ['mdn-css-animations.md', 'mdn-css-transitions.md', 'mdn-prefers-reduced-motion.md', 'css-modern-motion-notes.md'],
      exampleExt: 'css',
      example: `.panel {\n  opacity: 0;\n  transform: translateY(0.5rem);\n  transition:\n    opacity 180ms ease,\n    transform 180ms ease;\n}\n\n.panel[data-open='true'] {\n  opacity: 1;\n  transform: translateY(0);\n}\n\n@media (prefers-reduced-motion: reduce) {\n  .panel {\n    transition-duration: 1ms;\n    transform: none;\n  }\n}\n`,
    }),
    skill({
      name: 'web-lottie',
      title: 'Web Lottie',
      family: 'Lottie',
      short: 'Web Lottie and dotLottie asset integration',
      focus: 'lottie-web, dotLottie web components, animation JSON/dotLottie assets, player lifecycle, cleanup, renderer choice, accessibility, and asset validation.',
      triggers: ['lottie-web', 'dotLottie', '.lottie', 'Lottie JSON', 'After Effects animation', 'Bodymovin'],
      boundaries: ['Use native-lottie for React Native.', 'Use Rive for interactive state machines.', 'Use CSS/WAAPI for simple UI motion that does not need designer-authored assets.'],
      workflow: ['Inspect asset format, player package, renderer, autoplay/loop, and hosting path.', 'Create and destroy player instances at the owner boundary.', 'Respect reduced motion and provide non-canvas semantics.', 'Validate asset size, remote URLs, and event listeners.'],
      gotchas: ['Canvas-rendered animation needs external accessible text or labels.', 'Remote animation URLs need CSP/cache/security review.', 'Looping/autoplay assets require reduced-motion and pause behavior.'],
      references: [
        ref('lottie-player-lifecycle.md', 'lottie-web player lifecycle', ['Read when creating, updating, or destroying lottie-web animation instances.']),
        ref('dotlottie-web-component.md', 'dotLottie web component and worker notes', ['Read when using .lottie assets, dotLottie players, workers, or web components.']),
        ref('asset-accessibility-security.md', 'Asset accessibility and security review', ['Read before accepting remote assets, canvas-only output, autoplay loops, or URL actions.']),
      ],
      sourceDocs: ['lottie-web-readme.md', 'lottie-web-load-animation-options.md', 'dotlottie-web.md'],
      exampleExt: 'tsx',
      example: `import { useEffect, useRef } from 'react';\nimport lottie from 'lottie-web';\n\nexport function LottieBadge({ path }: { path: string }) {\n  const host = useRef<HTMLDivElement>(null);\n  useEffect(() => {\n    if (!host.current) return;\n    const animation = lottie.loadAnimation({ container: host.current, renderer: 'svg', loop: false, autoplay: true, path });\n    return () => animation.destroy();\n  }, [path]);\n  return <div ref={host} aria-label=\"Status animation\" />;\n}\n`,
    }),
    skill({
      name: 'web-motion-react',
      title: 'Web Motion React',
      family: 'Motion React',
      short: 'Motion for React presence, layout, and scroll',
      focus: 'Motion React components and hooks: motion, AnimatePresence, layout animations, useScroll, useReducedMotion, gestures, variants, and React/Next boundaries.',
      triggers: ['Motion React', 'motion/react', 'AnimatePresence', 'layout animation', 'useScroll', 'useReducedMotion', 'variants'],
      boundaries: ['Use GSAP for imperative timelines and plugin-heavy scenes.', 'Use CSS for simple static transitions.', 'Use WAAPI for low-level Animation object control outside React.'],
      workflow: ['Confirm package import path and React/client boundary.', 'Choose presence, layout, gesture, scroll, or value-based motion deliberately.', 'Respect reduced motion and state ownership.', 'Verify layout projection with resize, interruption, route changes, and hydration.'],
      gotchas: ['AnimatePresence requires stable keys and actual unmounts.', 'Layout animations depend on stable layout boxes and should not fight CSS transitions.', 'Do not push high-frequency motion values through React state.'],
      references: [
        ref('motion-react-presence-layout.md', 'Presence and layout workflow', ['Read for AnimatePresence, layout, shared layout, and exit transitions.']),
        ref('scroll-gestures-reduced-motion.md', 'Scroll, gesture, and reduced-motion hooks', ['Read for useScroll, useTransform, whileHover/tap/drag, and useReducedMotion.']),
        ref('react-ssr-client-boundaries.md', 'React and SSR/client boundaries', ['Read for Next.js, server components, hydration, and route-level validation.']),
      ],
      sourceDocs: ['motion-react.md', 'motion-animate-presence.md', 'motion-layout-animations.md', 'motion-use-scroll.md', 'motion-use-reduced-motion.md'],
      exampleExt: 'tsx',
      example: `'use client';\n\nimport { AnimatePresence, motion, useReducedMotion } from 'motion/react';\n\nexport function Notice({ open }: { open: boolean }) {\n  const reduce = useReducedMotion();\n  return (\n    <AnimatePresence>\n      {open ? <motion.div initial={{ opacity: 0, y: reduce ? 0 : 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0 }} /> : null}\n    </AnimatePresence>\n  );\n}\n`,
    }),
    skill({
      name: 'web-rive',
      title: 'Web Rive',
      family: 'Rive',
      short: 'Rive web animation asset workflows',
      focus: 'Rive web and React runtime integration, .riv assets, state machines, inputs, lifecycle cleanup, accessibility, remote asset security, and fallback behavior.',
      triggers: ['Rive', '.riv', 'state machine input', '@rive-app/react-canvas', '@rive-app/canvas', 'Rive web'],
      boundaries: ['Use web-lottie for Lottie/dotLottie assets.', 'Use native-rive for React Native.', 'Use Motion/GSAP/CSS when no .riv asset or state machine is involved.'],
      workflow: ['Inspect asset ownership, state machine names, inputs, autoplay, and runtime package.', 'Bind inputs through stable component state and cleanup runtime instances.', 'Add fallback and accessible semantics outside canvas.', 'Review URL actions and remote asset policy.'],
      gotchas: ['Canvas output is not self-describing to assistive tech.', 'State machine input names are asset contracts; verify against the asset.', 'Remote .riv files and URL actions need explicit allowlisting.'],
      references: [
        ref('rive-react-runtime.md', 'Rive React/runtime lifecycle', ['Read when using @rive-app/react-canvas or @rive-app/canvas.']),
        ref('state-machine-inputs.md', 'State machine input contract guide', ['Read when setting boolean, number, or trigger inputs.']),
        ref('asset-security-and-fallbacks.md', 'Rive asset security, accessibility, and fallback review', ['Read before shipping remote assets, URL actions, or canvas-only states.']),
      ],
      sourceDocs: ['rive-react.md', 'rive-state-machine.md', 'rive-web-js.md'],
      exampleExt: 'tsx',
      example: `import { useRive } from '@rive-app/react-canvas';\n\nexport function RiveLogo() {\n  const { RiveComponent } = useRive({\n    src: '/motion/logo.riv',\n    stateMachines: 'hero',\n    autoplay: true,\n  });\n  return <RiveComponent aria-label=\"Animated logo\" />;\n}\n`,
    }),
    skill({
      name: 'web-tailwind-motion',
      title: 'Web Tailwind Motion',
      family: 'Tailwind CSS',
      short: 'Tailwind v4 motion utilities and tokens',
      focus: 'Tailwind CSS v4 transition, animation, duration, easing, motion-safe/motion-reduce, @theme motion tokens, and static class safety.',
      triggers: ['Tailwind animation', 'transition-all', 'motion-safe', 'motion-reduce', '@theme', 'animate-', 'duration-'],
      boundaries: ['Use web-css-animations for raw CSS keyframes or browser support policy.', 'Use Motion/GSAP when React state or imperative sequencing owns motion.', 'Never generate unbounded runtime class strings for Tailwind.'],
      workflow: ['Inspect Tailwind version, CSS entrypoint, theme tokens, and class-generation policy.', 'Prefer explicit transition properties and tokenized durations/eases.', 'Use motion-safe/motion-reduce variants for user preference.', 'Validate generated classes are statically discoverable.'],
      gotchas: ['transition-all can hide expensive properties.', 'Tailwind v4 tokens usually live in CSS @theme, not only JS config.', 'Dynamic class concatenation can be purged or unsupported by local policy.'],
      references: [
        ref('tailwind-v4-motion-utilities.md', 'Tailwind transition and animation utilities', ['Read before adding transition, duration, ease, delay, animate, or motion variants.']),
        ref('token-and-theme-motion.md', 'Tailwind v4 @theme motion tokens', ['Read when adding or reviewing reusable motion tokens.']),
        ref('class-safety-audit.md', 'Static class and runtime string safety', ['Read when classes are generated from props, CMS data, or maps.']),
      ],
      sourceDocs: ['tailwind-animation.md', 'tailwind-transition-property.md', 'tailwind-theme.md', 'tailwind-v4-release.md'],
      exampleExt: 'html',
      example: `<button class=\"transition-[opacity,transform] duration-200 ease-out motion-safe:hover:-translate-y-0.5 motion-reduce:transition-none\">\n  Save\n</button>\n`,
    }),
    skill({
      name: 'web-three-r3f',
      title: 'Web Three R3F',
      family: 'Three.js/R3F',
      short: 'Web Three.js and React Three Fiber scenes',
      focus: 'Three.js, React Three Fiber, Drei, Canvas/createRoot lifecycle, loaders, GLTF, useFrame, disposal, SSR/client boundaries, DPR, and browser proof.',
      triggers: ['three', '@react-three/fiber', '@react-three/drei', 'R3F Canvas', 'useFrame', 'GLTF', 'WebGLRenderer'],
      boundaries: ['Use typegpu for typed WebGPU pipelines.', 'Use native-three-r3f for Expo/React Native.', 'Use CSS 3D transforms only for simple DOM transforms.'],
      workflow: ['Inspect Three/R3F/Drei/React versions and asset pipeline.', 'Choose Canvas/R3F or plain Three renderer based on local ownership.', 'Reserve stable layout, DPR, fallback, and reduced-motion behavior.', 'Verify nonblank pixels, resize, interaction, asset errors, and cleanup.'],
      gotchas: ['Undefined parent height causes blank canvases.', 'React state inside useFrame causes frame-loop reconciliation.', 'R3F auto-disposal does not cover every primitive or externally created object.'],
      references: [
        ref('r3f-scene-lifecycle.md', 'Canvas/createRoot and scene lifecycle', ['Read for Canvas props, custom root ownership, SSR/client boundaries, and resize.']),
        ref('asset-loaders-and-fallbacks.md', 'GLTF/texture loaders and fallbacks', ['Read for useGLTF/useTexture, Suspense, decoder paths, and asset errors.']),
        ref('three-disposal-performance.md', 'Three.js disposal and performance guide', ['Read for renderer cleanup, render targets, materials, textures, DPR, frameloop, and profiling.']),
      ],
      sourceDocs: ['r3f-canvas.md', 'r3f-pitfalls.md', 'r3f-scaling-performance.md', 'three-disposal.md', 'drei-gltf-use-gltf.md'],
      exampleExt: 'tsx',
      example: `'use client';\n\nimport { Canvas } from '@react-three/fiber';\nimport { Suspense } from 'react';\n\nexport function SceneSurface() {\n  return (\n    <Canvas dpr={[1, 1.5]} frameloop=\"demand\" camera={{ position: [0, 0, 6], fov: 45 }} fallback={<img src=\"/scene-fallback.jpg\" alt=\"\" />}>\n      <Suspense fallback={null}>{/* scene */}</Suspense>\n    </Canvas>\n  );\n}\n`,
    }),
    skill({
      name: 'web-waapi',
      title: 'Web WAAPI',
      family: 'WAAPI',
      short: 'Web Animations API lifecycle and playback',
      focus: 'Browser Web Animations API: Element.animate(), Animation, KeyframeEffect, playback control, generated keyframes, cancel/finish, commitStyles, and cleanup.',
      triggers: ['Element.animate', 'WAAPI', 'Web Animations API', 'KeyframeEffect', 'Animation object', 'commitStyles'],
      boundaries: ['Use CSS for simple state transitions.', 'Use Motion/GSAP when framework state or timelines dominate.', 'Use WAAPI when code needs an Animation object, seeking, cancellation, or generated keyframes.'],
      workflow: ['Check browser support and local fallback policy.', 'Create keyframes/options with explicit duration, fill, easing, and composite behavior.', 'Own animation cancellation and finish behavior.', 'Verify rapid interruptions, route unmount, reduced motion, and commitStyles usage.'],
      gotchas: ['commitStyles persists computed styles and should be followed by cancel when appropriate.', 'fill: forwards can retain stacking/style side effects.', 'Multiple animations on the same property need composite/replace intent.'],
      references: [
        ref('waapi-lifecycle.md', 'Animation object lifecycle', ['Read for play, pause, reverse, finish, cancel, commitStyles, and cleanup.']),
        ref('keyframe-effect-options.md', 'Keyframes and timing options', ['Read for KeyframeEffect, composite, fill, pseudoElement, iterations, and generated values.']),
        ref('playback-testing.md', 'Playback and interruption validation', ['Read for reduced motion, rapid toggles, route unmount, and deterministic testing.']),
      ],
      sourceDocs: ['mdn-web-animations-api.md'],
      exampleExt: 'ts',
      example: `export function fadeIn(node: HTMLElement, reduceMotion: boolean) {\n  const animation = node.animate(\n    [{ opacity: 0, transform: reduceMotion ? 'none' : 'translateY(8px)' }, { opacity: 1, transform: 'none' }],\n    { duration: reduceMotion ? 1 : 180, easing: 'ease-out', fill: 'both' },\n  );\n  return () => animation.cancel();\n}\n`,
    }),
  ],
  native: [
    skill({
      name: 'native-accessibility-performance',
      title: 'Native Accessibility Performance',
      family: 'React Native',
      short: 'Native motion accessibility and performance review',
      focus: 'Expo/React Native motion accessibility, reduced motion, haptics, UI-thread performance, frame pressure, gestures, and manual device proof.',
      triggers: ['React Native animation performance', 'reduced motion native', 'AccessibilityInfo', 'haptics', 'dropped frames', 'UI thread'],
      boundaries: ['Use native-motion-core for implementation patterns.', 'Use native-validation for command and device validation gates.', 'Use native-skia or native-three-r3f for canvas/GPU-specific performance.'],
      workflow: ['Inspect platform, Expo SDK, animation engine, and accessibility setting ownership.', 'Classify UI-thread, JS-thread, layout, and GPU work.', 'Add reduced-motion or static behavior without removing functional feedback.', 'Validate on iOS/Android or document skipped device proof.'],
      gotchas: ['Reduced motion should not remove essential progress, focus, pressed, or error feedback.', 'Haptics are feedback, not a substitute for visible state.', 'Per-frame React state updates can destroy native animation performance.'],
      references: [
        ref('reduced-motion-haptics-policy.md', 'Reduced motion and haptics policy', ['Read when user preference, haptics, or accessible feedback is involved.']),
        ref('native-performance-audit.md', 'Native performance audit guide', ['Read for JS/UI thread, layout, canvas, GPU, and list animation review.']),
        ref('manual-device-checks.md', 'Manual iOS/Android proof checklist', ['Read before finalizing changes that need simulator/device evidence.']),
      ],
      sourceDocs: ['rn-accessibility.md', 'rn-accessibilityinfo.md', 'rn-performance.md', 'reanimated-performance.md'],
      exampleExt: 'tsx',
      example: `import { AccessibilityInfo } from 'react-native';\n\nexport async function shouldReduceNativeMotion() {\n  return AccessibilityInfo.isReduceMotionEnabled();\n}\n`,
    }),
    skill({
      name: 'native-controls-transitions',
      title: 'Native Controls Transitions',
      family: 'Expo UI',
      short: 'Expo Router, screens, and native controls',
      focus: 'Expo Router Stack/native-stack transitions, react-native-screens boundaries, Expo UI SwiftUI/Jetpack Compose controls, native control animation ownership, and validation.',
      triggers: ['Expo Router Stack transition', 'native-stack animation', 'react-native-screens', 'Expo UI', 'SwiftUI control', 'Jetpack Compose control'],
      boundaries: ['Use native-motion-core for Reanimated-owned product motion.', 'Use native-styling-boundaries for NativeWind style ownership.', 'Use native-validation for EAS/device gates.'],
      workflow: ['Identify whether navigation, native control, or app state owns the transition.', 'Prefer platform-native transition knobs before custom overlays.', 'Keep Expo UI controls as leaf native controls.', 'Validate iOS and Android behavior when native controls or navigation config change.'],
      gotchas: ['Navigation transitions can fight screen-level Reanimated transitions.', 'Expo UI control props are not arbitrary React Native View animation surfaces.', 'Route params and unmount timing affect transition cleanup.'],
      references: [
        ref('expo-router-and-screens-transitions.md', 'Expo Router and screens transition guide', ['Read for Stack/native-stack options, route transitions, and screen lifecycle.']),
        ref('expo-ui-control-boundaries.md', 'Expo UI native control boundaries', ['Read when SwiftUI or Jetpack Compose leaf controls are involved.']),
        ref('native-navigation-validation.md', 'Navigation transition validation', ['Read before finalizing route, stack, or native control animation changes.']),
      ],
      sourceDocs: ['expo-router-stack.md', 'react-native-screens.md', 'react-navigation-native-stack.md', 'expo-ui.md'],
      exampleExt: 'tsx',
      example: `import { Stack } from 'expo-router';\n\nexport default function Layout() {\n  return <Stack screenOptions={{ animation: 'slide_from_right' }} />;\n}\n`,
    }),
    skill({
      name: 'native-lottie',
      title: 'Native Lottie',
      family: 'Lottie',
      short: 'React Native Lottie asset integration',
      focus: 'lottie-react-native and dotLottie native assets, Expo compatibility, asset bundling, refs, playback control, accessibility, reduced motion, and platform validation.',
      triggers: ['lottie-react-native', 'dotLottie React Native', 'LottieView', '.lottie native', 'After Effects native animation'],
      boundaries: ['Use web-lottie for browser Lottie.', 'Use native-rive for interactive Rive state machines.', 'Use native-motion-core for code-driven Reanimated motion.'],
      workflow: ['Check Expo SDK package compatibility and asset format.', 'Bundle assets through the app asset pipeline.', 'Own playback refs, pause/stop behavior, and unmount cleanup.', 'Validate Android/iOS rendering, reduced motion, and accessibility labels.'],
      gotchas: ['Large JSON animations can hurt startup and memory.', 'Autoplay loops need pause/reduced-motion behavior.', 'Native asset paths differ from web URLs and need bundler-safe imports.'],
      references: [
        ref('native-lottie-asset-lifecycle.md', 'Native Lottie asset lifecycle', ['Read for LottieView refs, source formats, asset imports, and playback control.']),
        ref('dotlottie-native-boundaries.md', 'dotLottie native boundaries', ['Read when using .lottie assets or LottieFiles native packages.']),
        ref('accessibility-performance.md', 'Native Lottie accessibility and performance', ['Read for labels, reduced motion, looping, asset size, and platform proof.']),
      ],
      sourceDocs: ['lottie-react-native-readme.md'],
      exampleExt: 'tsx',
      example: `import LottieView from 'lottie-react-native';\n\nexport function SuccessAnimation() {\n  return <LottieView source={require('./success.json')} autoPlay loop={false} accessibilityLabel=\"Success\" />;\n}\n`,
    }),
    skill({
      name: 'native-motion-core',
      title: 'Native Motion Core',
      family: 'Reanimated',
      short: 'Expo/RN Reanimated and Worklets motion',
      focus: 'Expo and React Native product motion with Reanimated 4, Worklets, shared values, animated styles/props, gestures, scroll handlers, layout animations, CSS transitions, and migration boundaries.',
      triggers: ['react-native-reanimated', 'react-native-worklets', 'useSharedValue', 'withTiming', 'withSpring', 'scheduleOnRN', 'layout animation'],
      boundaries: ['Use native-validation for command/device proof.', 'Use native-skia for canvas-heavy effects.', 'Use native-lottie/native-rive for designer-authored assets.'],
      workflow: ['Inspect Expo SDK, RN, Reanimated, Worklets, Gesture Handler, Babel config, and New Architecture mode.', 'Pick the smallest primitive: RN state/style, Reanimated CSS, shared values, gestures, scroll, or layout animation.', 'Keep product state in React/store and transient motion in shared values.', 'Validate interruption, unmount, reduced motion, and iOS/Android behavior.'],
      gotchas: ['Reanimated 4 requires compatible Worklets and New Architecture; Reanimated 3 advice differs.', 'Functions passed to scheduleOnRN must be defined on the RN runtime, not inside a worklet.', 'Reading sharedValue.value on JS can block; derive/consume on the UI thread.'],
      references: [
        ref('reanimated-worklets-core.md', 'Reanimated 4 and Worklets core workflow', ['Read before adding shared values, worklets, threading callbacks, or migration changes.']),
        ref('expo-sdk-compatibility.md', 'Expo SDK compatibility matrix', ['Read before changing package versions or applying npm-latest examples in Expo projects.']),
        ref('layout-scroll-gesture-patterns.md', 'Layout, scroll, and gesture patterns', ['Read for layout animations, scroll handlers, Gesture Handler 2, and frame callbacks.']),
      ],
      sourceDocs: ['expo-reanimated.md', 'reanimated-compatibility.md', 'reanimated-getting-started.md', 'reanimated-performance.md', 'reanimated-4-migration-testing.md'],
      exampleExt: 'tsx',
      example: `import Animated, { ReduceMotion, useAnimatedStyle, useSharedValue, withTiming } from 'react-native-reanimated';\n\nconst progress = useSharedValue(0);\nconst style = useAnimatedStyle(() => ({ opacity: progress.value }));\nprogress.value = withTiming(1, { duration: 180, reduceMotion: ReduceMotion.System });\n`,
    }),
    skill({
      name: 'native-rive',
      title: 'Native Rive',
      family: 'Rive',
      short: 'Rive React Native state-machine assets',
      focus: 'Rive React Native/Nitro runtime, .riv assets, state machines, inputs, asset loading, platform compatibility, accessibility, and iOS/Android proof.',
      triggers: ['@rive-app/react-native', 'Rive React Native', '.riv native', 'Rive state machine native', 'rive-nitro'],
      boundaries: ['Use web-rive for browser runtime.', 'Use native-lottie for Lottie/dotLottie assets.', 'Use native-motion-core for code-driven Reanimated motion.'],
      workflow: ['Check package/runtime compatibility and native build requirements.', 'Verify asset path, state machine name, input names, and autoplay.', 'Map app state to inputs with cleanup on unmount.', 'Validate iOS/Android build/rendering and accessibility fallback.'],
      gotchas: ['State machine names and inputs are asset contracts.', 'Native Rive runtime may require development build proof, not only Expo Go.', 'Canvas-like output needs surrounding accessible semantics.'],
      references: [
        ref('rive-native-state-machines.md', 'Native Rive state-machine contract', ['Read when binding inputs, triggers, or state machine names.']),
        ref('rive-native-asset-loading.md', 'Rive native asset loading and lifecycle', ['Read for .riv bundling, runtime setup, and cleanup.']),
        ref('nitro-platform-validation.md', 'Nitro/platform validation notes', ['Read before closing native build/runtime changes.']),
      ],
      sourceDocs: ['rive-react-native.md', 'rive-state-machine.md'],
      exampleExt: 'tsx',
      example: `import Rive from '@rive-app/react-native';\n\nexport function NativeRiveBadge() {\n  return <Rive resourceName=\"badge\" stateMachineName=\"badgeState\" autoplay />;\n}\n`,
    }),
    skill({
      name: 'native-skia',
      title: 'Native Skia',
      family: 'Skia',
      short: 'React Native Skia canvas motion',
      focus: 'React Native Skia canvas animations, drawing primitives, Reanimated integration, CanvasKit/web caveats, performance, memory, and Expo/native validation.',
      triggers: ['React Native Skia', '@shopify/react-native-skia', 'Skia Canvas', 'CanvasKit', 'Skia animation'],
      boundaries: ['Use native-motion-core for ordinary view motion.', 'Use native-three-r3f for 3D scenes.', 'Use native-lottie/native-rive for designer-authored assets.'],
      workflow: ['Check Skia, Expo SDK, platform support, and web requirements.', 'Keep drawing state and animation values in the appropriate runtime.', 'Avoid many animated native views when one canvas is the right surface.', 'Validate memory, resize, background/foreground, and platform rendering.'],
      gotchas: ['Skia web may require CanvasKit setup.', 'Canvas output needs accessible surrounding UI.', 'Large paths/images/shaders need memory and lifecycle ownership.'],
      references: [
        ref('skia-canvas-patterns.md', 'Skia canvas animation patterns', ['Read when building or reviewing Skia drawing/animation code.']),
        ref('skia-performance-lifecycle.md', 'Skia performance and lifecycle', ['Read for memory, resource, image/path/shader, and frame pressure review.']),
        ref('skia-web-expo-boundaries.md', 'Skia web and Expo boundaries', ['Read for CanvasKit, Expo package compatibility, and platform proof.']),
      ],
      sourceDocs: ['react-native-skia-installation.md', 'react-native-skia-api-notes.md', 'software-mansion-canvas-animations.md'],
      exampleExt: 'tsx',
      example: `import { Canvas, Circle } from '@shopify/react-native-skia';\n\nexport function Dot() {\n  return <Canvas style={{ width: 96, height: 96 }}><Circle cx={48} cy={48} r={24} color=\"black\" /></Canvas>;\n}\n`,
    }),
    skill({
      name: 'native-styling-boundaries',
      title: 'Native Styling Boundaries',
      family: 'NativeWind',
      short: 'NativeWind and Tailwind boundaries for RN',
      focus: 'NativeWind, react-native-css, Tailwind-style classes in React Native, static class safety, design tokens, Reanimated/CSS transition boundaries, and Expo setup.',
      triggers: ['NativeWind', 'react-native-css', 'Tailwind React Native', 'className native', 'motion-safe native', 'NativeWind animation'],
      boundaries: ['Use native-motion-core for Reanimated implementation logic.', 'Use web-tailwind-motion for browser Tailwind.', 'Do not generate untrusted runtime class strings.'],
      workflow: ['Inspect NativeWind/react-native-css versions and Babel/Metro setup.', 'Keep classes statically discoverable and token-driven.', 'Choose style/class ownership before adding Reanimated or CSS transitions.', 'Validate iOS/Android/web behavior where NativeWind support differs.'],
      gotchas: ['Web Tailwind assumptions do not always map to React Native style props.', 'Runtime string concatenation can break class extraction and policy.', 'Animation ownership should not be split between NativeWind classes and Reanimated shared values.'],
      references: [
        ref('nativewind-v4-boundaries.md', 'NativeWind setup and version boundaries', ['Read for NativeWind/react-native-css setup and class ownership.']),
        ref('react-native-css-tailwind.md', 'React Native CSS and Tailwind compatibility', ['Read when CSS-like transitions or Tailwind utilities cross native/web boundaries.']),
        ref('class-safety-and-tokens.md', 'Native class safety and token policy', ['Read for dynamic class generation, theme tokens, and design-system constraints.']),
      ],
      sourceDocs: ['nativewind-installation.md', 'expo-tailwind-setup.md'],
      exampleExt: 'tsx',
      example: `import { Pressable, Text } from 'react-native';\n\nexport function NativeButton() {\n  return <Pressable className=\"rounded-md bg-black px-4 py-3 active:opacity-80\"><Text className=\"text-white\">Save</Text></Pressable>;\n}\n`,
    }),
    skill({
      name: 'native-three-r3f',
      title: 'Native Three R3F',
      family: 'Three.js/R3F',
      short: 'Native Three/R3F GPU scene boundaries',
      focus: 'React Three Fiber native, Three.js in Expo/React Native, expo-gl/WebGPU boundaries, GLTF/assets, native GPU lifecycle, and platform validation.',
      triggers: ['@react-three/fiber/native', 'Expo Three', 'expo-gl', 'react-native-wgpu', 'native Three.js', 'R3F native'],
      boundaries: ['Use web-three-r3f for browser Three/R3F.', 'Use typegpu for web TypeGPU code.', 'Use native-skia for 2D canvas drawing.'],
      workflow: ['Inspect Expo SDK, GL/WebGPU package, R3F/Three versions, and asset pipeline.', 'Choose native R3F only when a 3D scene is the product surface.', 'Own canvas dimensions, DPR/quality, loaders, and cleanup.', 'Validate on device/development build for native GPU risk.'],
      gotchas: ['Browser R3F examples often assume DOM/WebGL APIs absent on native.', 'Native asset loading and decoder paths differ from web.', 'GPU runtime changes need device proof, not just TypeScript.'],
      references: [
        ref('r3f-native-installation.md', 'R3F native installation and lifecycle', ['Read for @react-three/fiber/native setup and scene ownership.']),
        ref('expo-webgpu-three-boundary.md', 'Expo GL/WebGPU/Three boundary notes', ['Read when expo-gl, react-native-wgpu, or WebGPU interop appears.']),
        ref('native-gpu-validation.md', 'Native GPU validation checklist', ['Read before closing R3F/Three native changes.']),
      ],
      sourceDocs: ['r3f-react-native-installation.md', 'r3f-native-api-notes.md', 'expo-webgpu-three.md', 'three-creating-a-scene.md'],
      exampleExt: 'tsx',
      example: `import { Canvas } from '@react-three/fiber/native';\n\nexport function NativeScene() {\n  return <Canvas>{/* native scene */}</Canvas>;\n}\n`,
    }),
    skill({
      name: 'native-validation',
      title: 'Native Validation',
      family: 'Expo validation',
      short: 'Expo/RN motion validation gates',
      focus: 'Validation for native motion changes: Expo Doctor, expo install --check, EAS/development build risk, Jest/Reanimated setup, RN tests, platform smoke, and audit report closeout.',
      triggers: ['Expo Doctor', 'expo install --check', 'EAS build validation', 'native motion validation', 'Reanimated Jest', 'development build proof'],
      boundaries: ['Use implementation skills for code changes first.', 'Use this skill whenever native package/config/runtime risk is part of the task.', 'Do not replace device proof with lint/typecheck for native runtime changes.'],
      workflow: ['Identify package/config/native-risk surface.', 'Run audit doctor/scan and repo-native package checks.', 'Choose local test, simulator/device, development build, or EAS proof based on risk.', 'Report commands, skipped checks, and residual risk.'],
      gotchas: ['Expo-compatible package versions can lag npm latest.', 'Expo Go proof is not enough for modules requiring a development build.', 'Jest animation tests need Reanimated setup and fake-timer discipline.'],
      references: [
        ref('expo-doctor-eas-gates.md', 'Expo Doctor, install check, and EAS gates', ['Read to choose validation commands for package/config/native-risk changes.']),
        ref('motion-package-compatibility.md', 'Motion package compatibility matrix', ['Read before changing Expo/Reanimated/Worklets/Lottie/Skia/Rive packages.']),
        ref('test-and-device-matrix.md', 'Native test and device proof matrix', ['Read for Jest, RN tests, simulator/device, and development build selection.']),
      ],
      sourceDocs: ['expo-doctor.md', 'eas-build.md', 'expo-development-builds.md', 'expo-new-architecture.md', 'reanimated-jest-and-worklets.md'],
      exampleExt: 'md',
      example: `# Native validation note\n\n- Run package compatibility checks first.\n- Use Expo Doctor for Expo projects.\n- Require simulator/device proof for native runtime, GPU, config, or package changes.\n`,
    }),
  ],
};

const sourceAnchors = {
  agentSkillsBestPractices: 'https://agentskills.io/skill-creation/best-practices',
  agentSkillsSpec: 'https://agentskills.io/specification',
  gsapCore: 'https://gsap.com/docs/v3/GSAP/gsap.to()',
  gsapMatchMedia: 'https://gsap.com/docs/v3/GSAP/gsap.matchMedia()',
  gsapContext: 'https://gsap.com/docs/v3/GSAP/gsap.context()',
  gsapScrollTrigger: 'https://gsap.com/docs/v3/Plugins/ScrollTrigger/',
  gsapPlugins: 'https://gsap.com/docs/v3/Plugins/',
  gsapTimeline: 'https://gsap.com/docs/v3/GSAP/Timeline/',
  gsapUtils: 'https://gsap.com/docs/v3/GSAP/UtilityMethods/',
  mdnCssTransitions: 'https://developer.mozilla.org/docs/Web/CSS/CSS_Transitions/Using_CSS_transitions',
  mdnCssAnimations: 'https://developer.mozilla.org/docs/Web/CSS/CSS_Animations/Using_CSS_animations',
  mdnTransitionBehavior: 'https://developer.mozilla.org/docs/Web/CSS/transition-behavior',
  mdnWaapi: 'https://developer.mozilla.org/docs/Web/API/Web_Animations_API/Using_the_Web_Animations_API',
  mdnCommitStyles: 'https://developer.mozilla.org/docs/Web/API/Animation/commitStyles',
  mdnWebGpu: 'https://developer.mozilla.org/docs/Web/API/WebGPU_API',
  tailwindTransitions: 'https://tailwindcss.com/docs/transition-property',
  tailwindAnimation: 'https://tailwindcss.com/docs/animation',
  tailwindTheme: 'https://tailwindcss.com/docs/theme',
  motionReact: 'https://motion.dev/react',
  motionComponent: 'https://motion.dev/motion/component',
  lottieWeb: 'https://github.com/airbnb/lottie-web/wiki/Usage',
  dotlottieWeb: 'https://github.com/LottieFiles/dotlottie-web',
  riveWeb: 'https://rive.app/docs/runtimes/web',
  riveWebStateMachines: 'https://rive.app/docs/runtimes/web/state-machines',
  r3fCanvas: 'https://r3f.docs.pmnd.rs/api/canvas',
  r3fPitfalls: 'https://r3f.docs.pmnd.rs/advanced/pitfalls',
  r3fScaling: 'https://r3f.docs.pmnd.rs/advanced/scaling-performance',
  dreiGltf: 'https://drei.docs.pmnd.rs/loaders/gltf-use-gltf',
  threeCleanup: 'https://threejs.org/manual/#en/cleanup',
  typegpuSchemas: 'https://docs.swmansion.com/TypeGPU/apis/data-schemas/',
  typegpuDocs: 'https://docs.swmansion.com/TypeGPU/',
  expoReanimated: 'https://docs.expo.dev/versions/latest/sdk/reanimated/',
  expoRouterStack: 'https://docs.expo.dev/router/advanced/stack/',
  expoUi: 'https://docs.expo.dev/versions/latest/sdk/ui/',
  expoDoctor: 'https://docs.expo.dev/workflow/diagnostics/',
  expoDevelopmentBuilds: 'https://docs.expo.dev/develop/development-builds/introduction/',
  easBuild: 'https://docs.expo.dev/build/setup/',
  expoTailwind: 'https://docs.expo.dev/guides/tailwind/',
  reactNativeAnimations: 'https://reactnative.dev/docs/animations',
  reactNativeAccessibility: 'https://reactnative.dev/docs/accessibility',
  reanimatedGettingStarted: 'https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/getting-started',
  reanimatedAccessibility: 'https://docs.swmansion.com/react-native-reanimated/docs/guides/accessibility/',
  reanimatedPerformance: 'https://docs.swmansion.com/react-native-reanimated/docs/guides/performance/',
  reanimatedWorklets: 'https://docs.swmansion.com/react-native-reanimated/docs/guides/worklets',
  nativewindInstall: 'https://www.nativewind.dev/docs/getting-started/installation',
  nativewindMigrate: 'https://www.nativewind.dev/v5/guides/migrate-from-v4',
  lottieNativeExpo: 'https://docs.expo.dev/versions/latest/sdk/lottie/',
  dotlottieNative: 'https://github.com/LottieFiles/dotlottie-react-native',
  riveNative: 'https://rive.app/docs/runtimes/react-native',
  riveNativeStateMachines: 'https://rive.app/docs/runtimes/react-native/state-machines',
  skiaInstall: 'https://shopify.github.io/react-native-skia/docs/getting-started/installation',
  skiaAnimations: 'https://shopify.github.io/react-native-skia/docs/animations/animations',
  r3fNative: 'https://docs.pmnd.rs/react-three-fiber/getting-started/installation#react-native',
};

const supplementalReferences = {
  'gsap-core': [
    supplemental('tween-interruption-patterns.md', 'Tween interruption and overwrite patterns', 'Read when tweens can overlap, replay, reverse, or leave inline styles behind.', [
      'Store returned Tween handles when a user interaction, route transition, or component cleanup must pause, reverse, kill, or inspect progress.',
      'Use overwrite intent deliberately. `overwrite: "auto"` is useful for overlapping active tweens on the same target, but it is not a substitute for one clear animation owner.',
      'Use `clearProps` only when CSS classes or external state should regain ownership after the tween completes.',
    ], [
      'Find multiple tweens targeting the same element and property and identify which one owns interruption behavior.',
      'Verify rapid toggles, replay, route unmount, and final inline styles.',
    ], [
      'Delay chains that should be a timeline.',
      'Fire-and-forget tweens in event handlers with no handle or cleanup path.',
    ], [sourceAnchors.gsapCore, sourceAnchors.gsapTimeline]),
    supplemental('responsive-accessibility-patterns.md', 'Responsive and reduced-motion GSAP patterns', 'Read when GSAP motion depends on breakpoints, user preference, or nonessential decorative motion.', [
      'Prefer `gsap.matchMedia()` for breakpoint-specific setup because animations created in matching callbacks are collected and reverted with the media context.',
      'Treat reduced motion as a behavioral branch, not just a shorter duration. Decorative movement can be skipped; functional feedback should remain visible and understandable.',
      'Keep focus, pointer, and visibility semantics aligned with `autoAlpha`, display changes, and hidden states.',
    ], [
      'Check `(prefers-reduced-motion: reduce)` handling and manual product motion toggles.',
      'Verify resize across breakpoints and cleanup with `mm.revert()` or framework lifecycle cleanup.',
    ], [
      'Animating off-screen entrances for reduced-motion users without a static final state.',
      'Duplicating matchMedia and context ownership around the same DOM nodes.',
    ], [sourceAnchors.gsapMatchMedia, sourceAnchors.agentSkillsBestPractices]),
  ],
  'gsap-frameworks': [
    supplemental('ssr-hydration-boundaries.md', 'SSR and hydration boundaries for framework GSAP', 'Read when Vue, Svelte, Astro, Nuxt, or islands code crosses server/client runtime boundaries.', [
      'Keep GSAP setup inside client-only lifecycle hooks or island hydration boundaries. Server-rendered code can prepare static state but cannot touch DOM, window, or measured layout.',
      'Use a component root ref for scoped selectors so framework reactivity and hydration do not animate matching nodes outside the component.',
      'Treat route transitions and island teardown as cleanup boundaries; revert contexts and kill timelines there.',
    ], [
      'Confirm the code path cannot run during server render.',
      'Verify hydration does not flash from the animated start state.',
    ], [
      'Top-level `window`, `document`, or DOM measurement in SSR-capable modules.',
      'Global selectors in reusable framework components.',
    ], [sourceAnchors.gsapContext, sourceAnchors.gsapCore]),
    supplemental('framework-composables-and-actions.md', 'Framework composables, actions, and directive wrappers', 'Read before wrapping GSAP in a Vue composable, Svelte action, Astro component script, or Nuxt plugin.', [
      'A wrapper should own one lifecycle boundary and return cleanup. Avoid helper APIs that hide cleanup or selector scope from the caller.',
      'Register plugins at an app/module boundary when possible, not inside every render or reactive effect.',
      'Reactive dependency changes should recreate only the animation that depends on that data, not the whole page motion system.',
    ], [
      'Inspect the wrapper API for explicit target, scope, dependency, and cleanup inputs.',
      'Run a mount/unmount/remount check in the framework runtime when the wrapper is shared.',
    ], [
      'Composable APIs that accept raw selector strings without a scope.',
      'Repeated plugin registration inside reactive render/update paths.',
    ], [sourceAnchors.gsapPlugins, sourceAnchors.agentSkillsSpec]),
  ],
  'gsap-performance': [
    supplemental('profiling-playbook.md', 'GSAP profiling and evidence playbook', 'Read when a GSAP issue is described as jank, dropped frames, layout thrash, or slow scroll.', [
      'Start from evidence: browser performance profile, visible reproduction, affected route, device/browser, and animation trigger path.',
      'Classify costs as JavaScript, style recalculation, layout, paint, composite, image/decode, or scroll workload before suggesting an engine change.',
      'Use the audit CLI for leads, then verify each finding against runtime behavior.',
    ], [
      'Capture before/after evidence for at least the hot interaction.',
      'Check frame budget under reduced CPU or representative device when the surface is user-facing.',
    ], [
      'Replacing a library without first identifying the bottleneck.',
      'Treating `will-change` as a blanket fix.',
    ], [sourceAnchors.gsapCore, sourceAnchors.gsapScrollTrigger]),
    supplemental('layer-budget-and-will-change.md', 'Layer budget and will-change discipline', 'Read before adding transform hacks, force3D, will-change, or GPU promotion advice.', [
      'Promote only hot animated elements and only for the lifetime of the effect when possible.',
      'Large layers, filters, shadows, video, and text rendering can shift cost from layout to memory/compositing instead of making it free.',
      'Use transform/opacity as the default fast path, but measure paint-heavy visual effects.',
    ], [
      'Search for persistent `will-change` on many elements.',
      'Review memory, layer count, text clarity, and paint cost when adding GPU promotion.',
    ], [
      'Global CSS that sets `will-change` across component classes.',
      'Animating filter or shadow while assuming transform-like compositing cost.',
    ], [sourceAnchors.gsapCore, sourceAnchors.mdnCssTransitions]),
  ],
  'gsap-plugins': [
    supplemental('premium-plugin-license-boundaries.md', 'Plugin availability and license boundaries', 'Read before generating imports or examples for SplitText, MorphSVG, DrawSVG, Club plugins, or any plugin absent from local dependencies.', [
      'Verify whether the plugin ships in the public `gsap` package, local project dependency, or a private/Club distribution before writing code.',
      'Keep the official GreenSock skill license separate from GSAP runtime/package licensing and plugin availability.',
      'If a premium plugin is unavailable, route to a public plugin or a different implementation only after explaining the tradeoff.',
    ], [
      'Inspect `package.json`, lockfile, and existing import paths for plugin availability.',
      'Confirm registration happens once and is tree-shake/bundle safe for the project runtime.',
    ], [
      'Invented import paths for plugins not present in the installed package.',
      'Copying premium-plugin examples into a repo without confirming license/access.',
    ], [sourceAnchors.gsapPlugins, 'https://gsap.com/standard-license/']),
    supplemental('plugin-specific-test-fixtures.md', 'Plugin-specific fixtures and verification', 'Read when validating Flip, Draggable, Observer, MotionPath, ScrollTo, SVG, or text plugin behavior.', [
      'Build tiny fixtures around the plugin contract: before/after DOM for Flip, pointer input for Draggable/Observer, path coordinates for MotionPath, and scroll container identity for ScrollTo.',
      'Plugin behavior often depends on measured layout or device input, so typecheck-only validation is not enough.',
      'Prefer stable selectors/refs and deterministic fixture sizes for repeatable audits.',
    ], [
      'Verify resize, interruption, cleanup, and reduced-motion alternatives.',
      'Run pointer/keyboard accessibility checks for interactive plugins.',
    ], [
      'Testing Flip without a real before/after state change.',
      'Shipping drag/observer interactions without keyboard or pointer fallback review.',
    ], [sourceAnchors.gsapPlugins, sourceAnchors.gsapCore]),
  ],
  'gsap-react': [
    supplemental('contextsafe-event-handlers.md', 'contextSafe event handlers and callbacks', 'Read when GSAP code runs from React event handlers, timeouts, observers, or async callbacks after initial setup.', [
      'Use context-safe callback patterns so animations created after the initial hook setup are still associated with the component context and cleanup boundary.',
      'Keep event-driven tweens scoped to refs or a GSAP context. Avoid document-level selectors from React handlers.',
      'Store handles for animations that can outlive the synchronous event callback.',
    ], [
      'Check event listeners, observers, timers, promises, and route callbacks for cleanup.',
      'Verify the component can unmount during an in-flight animation without leaked tweens.',
    ], [
      'Creating tweens in click handlers with unscoped selectors.',
      'Assuming hook cleanup covers async-created animations automatically.',
    ], [sourceAnchors.gsapContext, sourceAnchors.gsapCore]),
    supplemental('strict-mode-and-route-transitions.md', 'React Strict Mode, dependency, and route transition checks', 'Read when GSAP setup reruns, double-renders in development, or participates in Next.js route transitions.', [
      'React development Strict Mode can expose side effects that were hidden by single-mount assumptions. Treat duplicate setup as a signal to scope and cleanup correctly.',
      'Dependencies should rebuild only the animation affected by changed data. Recreating the whole scene on every render is usually a bug.',
      'Next.js App Router components that touch GSAP need client boundaries and route-level cleanup proof.',
    ], [
      'Run mount/remount and route navigation checks.',
      'Inspect dependency arrays and ref identity for unnecessary animation reconstruction.',
    ], [
      'Running GSAP in server components.',
      'Using React state updates inside high-frequency GSAP callbacks.',
    ], [sourceAnchors.gsapContext, sourceAnchors.gsapMatchMedia]),
  ],
  'gsap-scrolltrigger': [
    supplemental('smooth-scroll-and-scroller-proxy.md', 'Smooth-scroll and scroller proxy boundary', 'Read when ScrollTrigger is combined with Lenis, Locomotive, custom scroll containers, overflow panels, or transformed parents.', [
      'Identify the real scroll owner before adding triggers. Window scroll, nested overflow containers, and smooth-scroll libraries have different refresh and proxy requirements.',
      'Keep smooth-scroll integration centralized; do not configure proxy behavior ad hoc in individual components.',
      'Transformed ancestors, pinned elements, and custom scrollers change measurement assumptions.',
    ], [
      'Verify trigger positions after resize, content load, route transition, and smooth-scroll start/stop.',
      'Check mobile touch scroll, keyboard scroll, and reduced-motion behavior.',
    ], [
      'Multiple components each creating their own smooth-scroll proxy.',
      'Pins inside transformed containers without geometry proof.',
    ], [sourceAnchors.gsapScrollTrigger, sourceAnchors.gsapMatchMedia]),
    supplemental('responsive-refresh-playbook.md', 'Responsive refresh and invalidation playbook', 'Read when ScrollTrigger scenes depend on images, fonts, breakpoints, async content, or route-level layout shifts.', [
      'Use refresh/invalidation as an ordered response to layout changes, not a random timeout. Fonts, images, CMS content, accordions, and virtualized lists all need explicit reasoning.',
      'Responsive scenes should be created and reverted through matchMedia or framework cleanup.',
      'Pinned scenes need extra proof because pin spacing mutates page geometry.',
    ], [
      'Test reload, resize, route navigation, late image/font load, and content expansion.',
      'Enable markers temporarily during development, then remove them before closeout.',
    ], [
      'Leaving `markers: true` in production code.',
      'Nested ScrollTriggers inside timeline child tweens.',
    ], [sourceAnchors.gsapScrollTrigger, sourceAnchors.gsapMatchMedia]),
  ],
  'gsap-timeline': [
    supplemental('sequence-design-taxonomy.md', 'Timeline sequence design taxonomy', 'Read before converting delay chains, staggered entrances, or product choreography into timelines.', [
      'Model the sequence as named phases, labels, and relative positions. Labels document product intent and make later edits safer.',
      'Use timeline defaults for repeated duration/ease values and explicit child vars only when they differ.',
      'A timeline should own order; individual child tweens should not smuggle independent delays that make the sequence hard to reason about.',
    ], [
      'Inspect whether labels map to meaningful UI phases.',
      'Verify replay, reverse, seek, pause, and interruption semantics if exposed to users.',
    ], [
      'Constructor `duration` used as if it controlled every child tween.',
      'Long delay chains instead of position parameters.',
    ], [sourceAnchors.gsapTimeline, sourceAnchors.gsapCore]),
    supplemental('timeline-state-machine-patterns.md', 'Timeline state-machine and playback patterns', 'Read when a timeline is controlled by app state, route state, media queries, or user playback controls.', [
      'Keep the authoritative state in the app or the timeline, not both. A UI toggle should map clearly to play, reverse, seek, progress, or rebuild.',
      'Use labels and progress values for deterministic tests and reproducible bug reports.',
      'When timeline state depends on layout, rebuild at the same boundary where layout ownership changes.',
    ], [
      'Check rapid toggle behavior and final visual state.',
      'Verify cleanup kills or reverts the timeline and owned child tweens.',
    ], [
      'Mixing CSS transitions on the same properties controlled by timeline children.',
      'Recreating timelines every render while keeping old handles alive.',
    ], [sourceAnchors.gsapTimeline, sourceAnchors.gsapContext]),
  ],
  'gsap-utils': [
    supplemental('function-composition-recipes.md', 'gsap.utils function composition recipes', 'Read when clamp, mapRange, normalize, interpolate, snap, pipe, unitize, wrap, or selector utilities feed animation values.', [
      'Prefer reusable composed functions for pointer, scroll, drag, and progress mappings so boundaries are explicit and testable.',
      'Keep unit conversion at the edge. Numeric helpers should not silently accept CSS unit strings unless `unitize` or a specific parser owns that boundary.',
      'Use scoped selector helpers in component systems to avoid cross-component targeting.',
    ], [
      'Test min, max, below-range, above-range, and NaN inputs for mapping helpers.',
      'Check whether helper functions allocate work inside frame loops.',
    ], [
      'Mapping unit strings with numeric helpers.',
      'Creating new helper functions on every frame or render when a stable function would work.',
    ], [sourceAnchors.gsapUtils, sourceAnchors.gsapCore]),
    supplemental('random-determinism-testing.md', 'Randomness, wrapping, and deterministic testing', 'Read when `random`, `shuffle`, `wrap`, `wrapYoyo`, or function-based values affect visual output.', [
      'Use deterministic inputs or seeded alternatives in tests and visual review fixtures when random output would make failures hard to reproduce.',
      'Document whether randomness is product behavior, decorative variance, or a placeholder to remove.',
      'Wrap helpers are useful for cyclic indexes and carousel-like ranges, but they need explicit bounds and behavior at negative values.',
    ], [
      'Check snapshot/visual tests for nondeterministic animation values.',
      'Verify cyclic values at first, last, overflow, and underflow indexes.',
    ], [
      'Random values in SSR-rendered markup or hydration-sensitive initial styles.',
      'Function-based values with side effects.',
    ], [sourceAnchors.gsapUtils, sourceAnchors.agentSkillsBestPractices]),
  ],
  typegpu: [
    supplemental('browser-capability-and-adapter-selection.md', 'Browser capability and adapter selection', 'Read before creating TypeGPU roots, requesting WebGPU adapters/devices, or designing fallbacks.', [
      'WebGPU requires browser/runtime support and a secure context in ordinary web deployments. Treat unsupported adapters as a product state, not an exception path users should see.',
      'Separate capability detection from scene/pipeline creation so fallback UI can render without allocating GPU resources.',
      'Record required limits/features if a shader or texture path needs more than default device capabilities.',
    ], [
      'Test unsupported-browser fallback, device lost handling, and cleanup.',
      'Check SSR/client boundaries before touching `navigator.gpu`.',
    ], [
      'Allocating buffers before adapter/device success is known.',
      'Assuming WebGPU availability in all Chromium-based embedded browsers.',
    ], [sourceAnchors.typegpuDocs, sourceAnchors.mdnWebGpu]),
    supplemental('compute-vs-render-pipeline-design.md', 'Compute versus render pipeline design', 'Read when choosing TypeGPU compute, render, storage buffer, texture, or shader-function patterns.', [
      'Use schemas as the shared contract between CPU data layout, TypeScript types, and shader access.',
      'Choose compute when the output is data for later stages, and render when the output is pixels for a render target or canvas.',
      'Cache pipelines, bind groups, and buffers by shape/usage; avoid per-frame structural allocation.',
    ], [
      'Verify buffer usage flags match read/write/bind behavior.',
      'Add small deterministic fixtures for schema packing and shader input/output shape.',
    ], [
      'Duplicating layout definitions in TypeScript and WGSL.',
      'Recreating pipelines or bind groups inside frame loops without measurement.',
    ], [sourceAnchors.typegpuSchemas, sourceAnchors.typegpuDocs]),
  ],
  'web-css-animations': [
    supplemental('scroll-driven-and-view-timelines.md', 'Scroll-driven and view-timeline CSS motion', 'Read when using `animation-timeline`, `scroll()`, `view()`, timeline ranges, or CSS scroll-driven animation instead of JavaScript scroll handlers.', [
      'Use native scroll-driven animation when declarative support matches the browser policy and the effect does not need imperative playback state.',
      'Feature-detect newer syntax with `@supports` and route unsupported browsers to a static or simpler fallback.',
      'Keep scroll effects nonessential unless accessibility and keyboard/assistive navigation have been verified.',
    ], [
      'Check browser support policy and a reduced-motion branch.',
      'Verify behavior with keyboard scroll, mobile scroll, and content resizing.',
    ], [
      'Using scroll-driven CSS for critical state changes that need ARIA/state synchronization.',
      'Combining CSS scroll timelines and JS scroll libraries on the same element without an owner.',
    ], [sourceAnchors.mdnCssAnimations, sourceAnchors.mdnCssTransitions]),
    supplemental('discrete-entry-exit-transitions.md', 'Discrete entry and exit transitions', 'Read when animating `display`, `content-visibility`, popovers, dialogs, `@starting-style`, or `transition-behavior`.', [
      'Discrete transitions need explicit support reasoning. `@starting-style` handles entry setup; `transition-behavior: allow-discrete` handles discrete property transitions where supported.',
      'Do not let visual entry/exit become the only source of open/closed state. DOM state, focus, and ARIA still need to be correct.',
      'Prefer simple opacity/transform transitions when discrete support is outside the repo browser policy.',
    ], [
      'Test first render, reopen, close, focus return, and reduced motion.',
      'Check dialog/popover semantics separately from animation state.',
    ], [
      '`transition: all` around `display` and layout properties.',
      'Hiding focusable content visually while leaving it reachable.',
    ], [sourceAnchors.mdnTransitionBehavior, sourceAnchors.mdnCssTransitions]),
  ],
  'web-lottie': [
    supplemental('authoring-and-export-compatibility.md', 'Authoring and export compatibility', 'Read when accepting designer-authored Lottie JSON/dotLottie assets or debugging mismatch between After Effects preview and runtime output.', [
      'Treat the animation asset as a contract: dimensions, frame rate, markers, image assets, fonts/text, expressions, and unsupported effects should be reviewed before integration.',
      'Prefer local bundled assets or pinned package versions for production-critical animation. Remote library/CDN paths need supply-chain, CSP, cache, and outage review.',
      'Large vector assets can hurt startup, parse time, and memory even when rendering is GPU/canvas-backed.',
    ], [
      'Check asset size, renderer type, external images/fonts, and unsupported Bodymovin features.',
      'Verify loop/autoplay behavior under reduced motion.',
    ], [
      'Accepting arbitrary remote Lottie URLs from users or CMS data.',
      'Using canvas output without surrounding accessible semantics.',
    ], [sourceAnchors.lottieWeb, sourceAnchors.dotlottieWeb]),
    supplemental('runtime-event-contracts.md', 'Runtime events, markers, and playback contracts', 'Read when code controls Lottie playback, segments, markers, events, or synchronization with app state.', [
      'Use the player instance as the playback owner. Store it, destroy it at the component boundary, and avoid duplicate instances in the same container.',
      'Segments and markers should come from the asset contract and be validated against actual asset metadata.',
      'Synchronizing Lottie with app state requires interruption behavior for pause, stop, seek, route unmount, and asset replacement.',
    ], [
      'Verify load, error, complete, loop, and destroy behavior.',
      'Test replacing the `path` or `src` while the animation is playing.',
    ], [
      'Calling global lottie commands by name when a local instance handle is available.',
      'Leaving event listeners attached after destroy.',
    ], [sourceAnchors.lottieWeb, sourceAnchors.dotlottieWeb]),
  ],
  'web-motion-react': [
    supplemental('variants-and-motion-values.md', 'Variants and MotionValue state ownership', 'Read when Motion React variants, MotionValues, transforms, gestures, or high-frequency values are involved.', [
      'Use variants for named visual states shared across related components; use MotionValues for high-frequency values that should not flow through React state on every frame.',
      'Keep app state and animation state boundaries explicit. React state should choose modes; MotionValues can drive continuous values.',
      'Use `useReducedMotion` to branch nonessential movement while preserving affordances and final state.',
    ], [
      'Inspect whether high-frequency updates call React setters.',
      'Verify variant keys, initial/animate/exit states, and reduced-motion behavior.',
    ], [
      'Using variants as an untyped dump of unrelated states.',
      'Pushing scroll or pointer progress through React render state.',
    ], [sourceAnchors.motionReact, sourceAnchors.motionComponent]),
    supplemental('next-router-presence-boundaries.md', 'Next.js and router presence boundaries', 'Read when AnimatePresence, route transitions, layout animations, or shared layout effects cross routing/SSR boundaries.', [
      'Motion components that depend on browser runtime belong in client components. Server components can choose data and static layout, not animation objects.',
      'Exit animations require real unmounts and stable keys. Route-level presence should be placed where the framework actually changes children.',
      'Layout projection needs stable boxes; CSS display changes, suspense fallback swaps, and content loading can change measurements.',
    ], [
      'Test navigation, back/forward, suspense/loading states, and hydration.',
      'Verify exit animations fire exactly once and cleanup occurs after route changes.',
    ], [
      'Wrapping a subtree in AnimatePresence where children never unmount.',
      'Using unstable keys that force unwanted remounts and lost state.',
    ], [sourceAnchors.motionReact, sourceAnchors.agentSkillsBestPractices]),
  ],
  'web-rive': [
    supplemental('layout-fit-and-resize.md', 'Rive layout, fit, resize, and DPR behavior', 'Read when sizing a Rive canvas/component, changing artboards, or debugging cropped/blurred assets.', [
      'Canvas size, CSS size, device pixel ratio, and Rive layout fit/alignment all contribute to perceived framing and sharpness.',
      'Use the runtime layout options and surrounding CSS together; do not rely on the canvas default size.',
      'Resize observers or framework layout effects should update the runtime at the owner boundary.',
    ], [
      'Check desktop, mobile, high-DPR, and container resize.',
      'Verify fallback dimensions so layout does not jump before the asset loads.',
    ], [
      'A zero-height or auto-height container around the Rive canvas.',
      'Cropping interactive state-machine hit areas by CSS overflow without review.',
    ], [sourceAnchors.riveWeb, sourceAnchors.riveWebStateMachines]),
    supplemental('data-binding-and-events.md', 'Rive data binding and event contracts', 'Read when Rive state machines, inputs, events, or data binding connect to app state.', [
      'State machine names, artboards, inputs, and event names are asset contracts. Verify them against the `.riv` file rather than guessing from code.',
      'Map app state to Rive inputs at a stable boundary and debounce or gate high-frequency updates.',
      'URL actions and asset-driven events need allowlisting and product/security review.',
    ], [
      'Exercise boolean, numeric, and trigger inputs plus reset/replay behavior.',
      'Verify event listeners are removed or runtime instances are cleaned up.',
    ], [
      'Using a trigger input as durable application state.',
      'Trusting URL actions from unreviewed remote `.riv` files.',
    ], [sourceAnchors.riveWebStateMachines, sourceAnchors.riveWeb]),
  ],
  'web-tailwind-motion': [
    supplemental('responsive-motion-variants.md', 'Responsive and motion preference variants', 'Read when adding Tailwind responsive, hover/focus/active, `motion-safe`, or `motion-reduce` animation classes.', [
      'Use `motion-safe` for nonessential motion and `motion-reduce` to preserve meaningful state while removing or simplifying movement.',
      'Responsive variants should not create different semantic behavior across breakpoints; they should adapt timing, distance, or affordance density.',
      'Prefer explicit transitioned property utilities over broad transition helpers when performance or maintainability matters.',
    ], [
      'Check hover/focus/keyboard/touch behavior across breakpoints.',
      'Verify reduced-motion class branch in the rendered CSS/class output.',
    ], [
      '`transition-all` applied to components with layout-affecting class changes.',
      'Hover-only animated affordances with no focus or touch equivalent.',
    ], [sourceAnchors.tailwindTransitions, sourceAnchors.tailwindAnimation]),
    supplemental('semantic-token-naming.md', 'Semantic motion token naming', 'Read when adding Tailwind v4 `@theme` motion tokens or component-specific animation utilities.', [
      'Name motion tokens by product intent and scope, not implementation detail alone. `--ease-enter-panel` is more useful than another generic cubic-bezier token.',
      'Keep duration/ease/distance token ownership close to the design system; component overrides should be measured exceptions.',
      'Token changes are shared behavior changes and should be validated across representative components.',
    ], [
      'Search for existing duration/ease/animation tokens before adding new ones.',
      'Check generated utility names and static class discoverability.',
    ], [
      'Hard-coded one-off `duration-[...]` values repeated across components.',
      'Dynamic class strings assembled from untrusted CMS or user data.',
    ], [sourceAnchors.tailwindTheme, sourceAnchors.tailwindTransitions]),
  ],
  'web-three-r3f': [
    supplemental('interaction-and-event-boundaries.md', 'R3F interaction and event boundaries', 'Read when a Three/R3F scene handles pointer, keyboard, scroll, controls, raycasting, or HTML overlays.', [
      'R3F pointer events are scene events, not DOM events with identical propagation. Keep overlay DOM, canvas events, and controls ownership explicit.',
      'Camera controls can fight page scroll and mobile touch. Decide whether the canvas captures, passes through, or conditionally handles gestures.',
      'Accessible controls and fallback content must exist outside the WebGL canvas when interaction is meaningful.',
    ], [
      'Test pointer, touch, keyboard, scroll, and overlay hit-testing.',
      'Verify nonblank canvas pixels and fallback behavior when WebGL fails.',
    ], [
      'Full-screen canvas swallowing page scroll on mobile without product intent.',
      'Important labels or controls rendered only inside WebGL text.',
    ], [sourceAnchors.r3fCanvas, sourceAnchors.r3fPitfalls]),
    supplemental('asset-pipeline-compression.md', '3D asset pipeline, compression, and loader policy', 'Read when loading GLTF/GLB, textures, Draco/Meshopt/KTX2 assets, or remote 3D content.', [
      'Model loading is both a runtime and build/deploy concern. Decoder paths, public asset URLs, cache headers, suspense fallback, and bundle size all matter.',
      'Use the project asset pipeline and avoid unreviewed remote models unless the security/cache policy allows them.',
      'Dispose of externally created geometries, materials, textures, render targets, and controls when the owner unmounts.',
    ], [
      'Verify missing asset, decode failure, slow network, and route unmount behavior.',
      'Check texture size, compression format, DPR, and memory budget.',
    ], [
      'Loading large uncompressed GLB/textures directly in the first route without fallback.',
      'Creating Three objects outside R3F ownership without cleanup.',
    ], [sourceAnchors.dreiGltf, sourceAnchors.threeCleanup, sourceAnchors.r3fScaling]),
  ],
  'web-waapi': [
    supplemental('promise-events-and-cancellation.md', 'Animation promises, events, and cancellation', 'Read when WAAPI code awaits `finished`, reacts to `finish`/`cancel`, or coordinates interruption.', [
      'WAAPI animations have lifecycle state. Treat `finished` promises, event listeners, cancellation, and route/component teardown as first-class control flow.',
      'A cancelled animation and a finished animation are different outcomes; cleanup and final styles should reflect that distinction.',
      'Avoid dangling promises that update UI after the owner unmounts.',
    ], [
      'Test cancel, finish, reverse, rapid restart, and unmount.',
      'Check that event listeners are removed or tied to the owner lifecycle.',
    ], [
      'Awaiting `animation.finished` without handling cancellation.',
      'Calling `commitStyles()` after a cancelled animation without verifying desired final state.',
    ], [sourceAnchors.mdnWaapi, sourceAnchors.mdnCommitStyles]),
    supplemental('testing-with-getanimations.md', 'Testing with getAnimations and deterministic fixtures', 'Read when testing or auditing WAAPI effects programmatically.', [
      '`Element.getAnimations()` and document-level animation inspection can confirm whether animations are still running, cancelled, or leaking.',
      'Use deterministic timing options in tests and isolate generated keyframes from layout-dependent values unless the test owns dimensions.',
      'Reduced-motion branches should still produce a stable final state that tests can assert.',
    ], [
      'Assert no orphaned animations after unmount or route change.',
      'Test fill/commit/cancel semantics with short durations or controlled clocks.',
    ], [
      'Using real-time sleeps as the only validation for animation completion.',
      'Leaving `fill: forwards` as hidden persistent state across tests.',
    ], [sourceAnchors.mdnWaapi, sourceAnchors.mdnCommitStyles]),
  ],
  'native-accessibility-performance': [
    supplemental('gesture-feedback-and-motion-sickness.md', 'Gesture feedback and motion sickness review', 'Read when gestures, haptics, parallax, carousels, shared transitions, or large native motion can affect comfort.', [
      'Reduced motion should preserve orientation, focus, progress, pressed, success, and error feedback while reducing vestibular movement.',
      'Haptics can reinforce state but cannot replace visible or screen-reader-accessible feedback.',
      'Large parallax, zoom, rotation, and full-screen movement deserve stricter device proof than small opacity/scale affordances.',
    ], [
      'Check system reduced-motion setting and any app-level motion preference.',
      'Verify screen reader labels and focus movement around animated state changes.',
    ], [
      'Removing all feedback for reduced-motion users.',
      'Using haptics as the only confirmation of a state change.',
    ], [sourceAnchors.reactNativeAccessibility, sourceAnchors.reanimatedAccessibility]),
    supplemental('frame-budget-instrumentation.md', 'Native frame budget instrumentation', 'Read when reports mention dropped frames, slow gestures, list jank, or JS/UI thread contention.', [
      'Classify work by JS thread, UI thread, layout, image decode, list virtualization, and GPU/canvas load before proposing a fix.',
      'Per-frame React state updates are a common source of animation jank; prefer UI-thread animation values where appropriate.',
      'Measure on representative devices or simulators when native motion is the user-visible surface.',
    ], [
      'Check development mode versus release/development-build behavior.',
      'Inspect lists, gestures, images, and expensive derived values near the animation.',
    ], [
      'Treating simulator-only proof as enough for GPU-heavy native surfaces.',
      'Adding more animation wrappers around a list instead of fixing virtualization/layout work.',
    ], [sourceAnchors.reactNativeAnimations, sourceAnchors.reanimatedPerformance]),
  ],
  'native-controls-transitions': [
    supplemental('platform-transition-option-map.md', 'Platform transition option map', 'Read before changing Expo Router Stack, native-stack, presentation, gesture, header, or modal transition options.', [
      'Navigation transitions are platform contracts. Prefer native-stack options before custom screen overlays when the transition is navigation-owned.',
      'iOS and Android can expose different transition names, gestures, and modal behavior. Validate both when product behavior is cross-platform.',
      'Screen options should be centralized where route ownership lives, not scattered across leaf content components.',
    ], [
      'Verify back gesture, deep link entry, modal dismiss, and reduced-motion settings.',
      'Check route params and unmount timing for transition cleanup.',
    ], [
      'Screen-level Reanimated transitions fighting native-stack transitions.',
      'Assuming iOS modal presentation behavior exists on Android.',
    ], [sourceAnchors.expoRouterStack, 'https://reactnavigation.org/docs/native-stack-navigator/']),
    supplemental('expo-ui-worklets-state.md', 'Expo UI worklets and native state boundaries', 'Read when Expo UI controls, SwiftUI/Jetpack Compose leaf controls, or worklet-backed native state are involved.', [
      'Expo UI controls are native leaves with their own platform behavior. Animate around them unless the package exposes a supported native/worklet state path.',
      'Keep app state, native control state, and worklet/native-state updates separated so events do not bounce unnecessarily through JS.',
      'When a native control becomes part of a transition, device proof matters more than static checks.',
    ], [
      'Inspect Expo SDK version and package docs before copying examples.',
      'Validate iOS and Android visual/state behavior for controls with platform-specific implementations.',
    ], [
      'Treating Expo UI controls as arbitrary Animated.View surfaces.',
      'Adding gesture/animation wrappers that break native control accessibility.',
    ], [sourceAnchors.expoUi, sourceAnchors.expoRouterStack]),
  ],
  'native-lottie': [
    supplemental('designer-handoff-and-feature-support.md', 'Native Lottie designer handoff and feature support', 'Read when a Lottie asset is new, visually wrong on device, large, or different from the design preview.', [
      'Review designer export settings, frame rate, dimensions, image/font dependencies, masks, mattes, expressions, and unsupported runtime features before blaming code.',
      'Native renderers can diverge from web previews. Device proof on iOS and Android is part of asset acceptance when visual fidelity matters.',
      'Large JSON assets affect app bundle size, parse time, and memory.',
    ], [
      'Check asset size, external files, playback speed, loop mode, and platform rendering.',
      'Verify reduced-motion behavior for autoplay and loops.',
    ], [
      'Using remote JSON assets in native without cache/security policy.',
      'Accepting a web-only Lottie preview as native proof.',
    ], [sourceAnchors.lottieNativeExpo, sourceAnchors.lottieWeb]),
    supplemental('native-playback-control-refs.md', 'Native Lottie playback refs and lifecycle', 'Read when code controls LottieView refs, imperative playback, progress, segments, or component unmount.', [
      'The owning component should control playback refs and stop/reset behavior. Avoid globally reachable animation handles.',
      'Progress-driven Lottie should have a stable source of truth and avoid per-frame React re-renders where native animation values can own the update.',
      'Autoplay/loop should pause or simplify under reduced motion and when screens lose focus.',
    ], [
      'Test screen blur/focus, unmount, app background/foreground, and asset replacement.',
      'Check accessible label/state outside the animation view.',
    ], [
      'Leaving looping animations active behind a hidden route.',
      'Using decorative animation progress as the only indication of completion.',
    ], [sourceAnchors.lottieNativeExpo, sourceAnchors.reactNativeAccessibility]),
  ],
  'native-motion-core': [
    supplemental('threading-runonjs-scheduleonrn.md', 'Reanimated threading and RN callback boundaries', 'Read when worklets call back to React Native, schedule JS work, or move data between UI and RN runtimes.', [
      'Worklets execute on the UI runtime. Functions that touch React state, navigation, analytics, or ordinary JS objects must be scheduled through the supported RN/JS boundary.',
      'Define callbacks on the RN runtime before passing them into scheduling helpers; do not create non-worklet functions inside worklets and call them synchronously.',
      'Avoid reading shared values on JS hot paths when derived UI-thread values would work.',
    ], [
      'Search for `runOnJS`, `scheduleOnRN`, shared value reads, and functions declared inside worklets.',
      'Verify callbacks cannot fire after component unmount or route change.',
    ], [
      'Synchronous calls from UI worklets to non-worklet functions.',
      'Using shared values as a general cross-thread state store.',
    ], [sourceAnchors.reanimatedWorklets, sourceAnchors.reanimatedGettingStarted]),
    supplemental('gesture-handler-integration.md', 'Gesture Handler and Reanimated integration', 'Read when pan, fling, tap, scroll, sheet, carousel, or drag interactions drive Reanimated values.', [
      'Gestures should update UI-thread animation state directly when possible and cross to RN only for durable product state changes.',
      'Gesture cancellation, velocity, bounds, snap points, and simultaneous/require-fail relationships are part of the animation contract.',
      'Reduced motion does not mean disabling gestures; it usually means simplifying travel, spring, or parallax behavior.',
    ], [
      'Test cancel, interruption, nested scroll, simultaneous gestures, and platform back/swipe gestures.',
      'Verify snap points and bounds with dynamic layout sizes.',
    ], [
      'Updating React state on every gesture frame.',
      'Hard-coding snap distances without measuring layout.',
    ], [sourceAnchors.reanimatedGettingStarted, sourceAnchors.reactNativeAnimations]),
  ],
  'native-rive': [
    supplemental('rive-file-caching-and-assets.md', 'Native Rive file caching and asset loading', 'Read when `.riv` files are bundled, cached, loaded remotely, reused across views, or include out-of-band assets.', [
      'A `.riv` file can be expensive to load and parse. Reuse/caching can be useful when the same file appears in multiple places, but ownership and invalidation must be explicit.',
      'Bundled native assets, remote assets, images, fonts, and audio have different build and runtime requirements.',
      'Remote Rive assets need allowlisting, cache policy, and failure fallback.',
    ], [
      'Test missing asset, slow load, route unmount, and repeated mount behavior.',
      'Verify development-build/native runtime requirements before closing.',
    ], [
      'Hard-coded asset names not verified against native bundling output.',
      'Remote `.riv` files without fallback or security review.',
    ], [sourceAnchors.riveNative, sourceAnchors.riveNativeStateMachines]),
    supplemental('state-machine-input-protocol.md', 'Native Rive state-machine input protocol', 'Read when app state drives boolean, number, or trigger inputs in a native Rive state machine.', [
      'State machine names and input names are designer/runtime contracts. Verify them with the asset before wiring app logic.',
      'Triggers are events, not durable state. Boolean/number inputs should represent durable state and be reset intentionally.',
      'Keep app state to Rive input mapping small and observable for debugging.',
    ], [
      'Exercise every state-machine input and reset/replay path.',
      'Check accessibility fallback for every meaningful visual state.',
    ], [
      'Guessing input names from code comments.',
      'Using animation state as the only business-state source.',
    ], [sourceAnchors.riveNativeStateMachines, sourceAnchors.riveNative]),
  ],
  'native-skia': [
    supplemental('skia-reanimated-interoperability.md', 'Skia and Reanimated interoperability', 'Read when Skia drawing state is animated by Reanimated shared values, gestures, or UI-thread updates.', [
      'Keep high-frequency drawing values off React render state. Reanimated shared values can drive Skia properties without forcing React reconciliation.',
      'Separate drawing primitives, animation values, and app state so each runtime owns the correct work.',
      'Skia effects that replace many native views should still expose accessible labels and actions through surrounding React Native UI.',
    ], [
      'Test gesture-driven updates, app background/foreground, resize, and reduced motion.',
      'Inspect whether derived drawing values allocate objects per frame.',
    ], [
      'Pushing Skia animation through React state on every frame.',
      'Canvas-only controls with no accessible native control layer.',
    ], [sourceAnchors.skiaAnimations, sourceAnchors.reanimatedGettingStarted]),
    supplemental('image-font-shader-resource-cache.md', 'Skia image, font, shader, and resource cache policy', 'Read when Skia code loads images, fonts, paths, shaders, color filters, or large resources.', [
      'Images, fonts, shaders, paths, and runtime effects are resources with load, cache, and invalidation behavior. Avoid rebuilding them per frame.',
      'Large canvas resources can become memory issues on lower-end devices even when animations are smooth.',
      'Web/CanvasKit paths can differ from native and need platform-specific validation.',
    ], [
      'Test missing resources, reload, theme changes, and memory-heavy surfaces.',
      'Verify cache keys and cleanup for screen unmount.',
    ], [
      'Creating shaders or parsed paths inside render/frame loops.',
      'Assuming Skia web behavior proves native behavior.',
    ], [sourceAnchors.skiaInstall, sourceAnchors.skiaAnimations]),
  ],
  'native-styling-boundaries': [
    supplemental('nativewind-metro-babel-pipeline.md', 'NativeWind Metro, Babel, and CSS pipeline', 'Read when NativeWind classes do not apply, hot reload fails, or setup crosses Expo/Metro/Tailwind config.', [
      'NativeWind/react-native-css setup is a toolchain pipeline: Babel, Metro, CSS entrypoint, Tailwind content scanning, and package versions must agree.',
      'Expo and native projects can differ in CSS asset support. Verify the actual app surface before copying setup from web Tailwind.',
      'Class extraction depends on static discoverability unless the local policy provides an explicit mapping.',
    ], [
      'Inspect package versions, Babel config, Metro config, CSS import, and content paths.',
      'Run a clean-start or cache-reset check when setup changes.',
    ], [
      'Debugging missing styles only at component code while setup is broken.',
      'Runtime class strings built from arbitrary user or CMS data.',
    ], [sourceAnchors.nativewindInstall, sourceAnchors.expoTailwind]),
    supplemental('cross-platform-style-differences.md', 'Cross-platform style and animation differences', 'Read when a class, transition, transform, color, or layout utility behaves differently on iOS, Android, and web.', [
      'React Native style semantics are not identical to browser CSS. Verify support for transforms, layout, pseudo states, media variants, and transitions per platform.',
      'Animation ownership should be singular: NativeWind classes can express static/pressed states, while Reanimated should own continuous interactive motion.',
      'Design tokens should map to platform-supported values rather than leaking browser-only CSS assumptions.',
    ], [
      'Check iOS, Android, and web where the app supports all three.',
      'Verify native accessibility states still map to visible states.',
    ], [
      'Assuming every Tailwind web utility has native parity.',
      'Combining NativeWind class transitions and Reanimated shared values for the same property without an owner.',
    ], [sourceAnchors.nativewindMigrate, sourceAnchors.expoTailwind]),
  ],
  'native-three-r3f': [
    supplemental('expo-gl-webgpu-decision-tree.md', 'Expo GL, WebGPU, and Three decision tree', 'Read when choosing expo-gl, WebGPU, Three.js, R3F native, or a fallback rendering surface.', [
      'Native Three/R3F work is a runtime decision, not just an import decision. Confirm Expo SDK, renderer support, development build requirements, and target platforms first.',
      'Use native 3D only when the product surface needs 3D. Use Skia for 2D canvas and Reanimated/native views for ordinary UI motion.',
      'Web examples often assume DOM, browser WebGL extension behavior, and asset loaders not present on native.',
    ], [
      'Check iOS/Android build and runtime proof requirements before coding deeply.',
      'Verify fallback behavior for unsupported devices or GPU errors.',
    ], [
      'Copying browser R3F Canvas examples into native without renderer/package review.',
      'Treating TypeScript success as GPU runtime proof.',
    ], [sourceAnchors.r3fNative, sourceAnchors.mdnWebGpu]),
    supplemental('native-asset-loader-recipes.md', 'Native 3D asset loader recipes', 'Read when loading GLTF/GLB, textures, HDR/environment maps, or decoder-backed assets in native Three/R3F.', [
      'Native asset loading needs bundler-safe imports, URI resolution, and platform proof. Public web URLs and decoder paths may not map to native.',
      'Large meshes/textures need memory budget and loading fallback. Compression helps only when the decoder path is available.',
      'Scene cleanup must include externally created materials, geometries, textures, controls, and renderer resources not owned automatically.',
    ], [
      'Test missing asset, reload, route unmount, memory pressure, and app background/foreground.',
      'Verify dimensions and DPR/quality on device.',
    ], [
      'Remote unpinned model URLs in production UI.',
      'Large uncompressed textures loaded before the screen needs them.',
    ], [sourceAnchors.r3fNative, sourceAnchors.threeCleanup, sourceAnchors.dreiGltf]),
  ],
  'native-validation': [
    supplemental('risk-tier-validation-ladder.md', 'Native motion risk-tier validation ladder', 'Read when deciding whether lint/typecheck, Expo Doctor, simulator/device, development build, or EAS proof is required.', [
      'Tier validation by blast radius: JS-only view motion can often use tests plus simulator proof; package/config/native module/GPU changes require device or development-build proof.',
      'Expo-compatible package versions can lag npm latest. Use Expo install/check paths before manual version changes.',
      'Document skipped device proof explicitly with the reason and residual risk.',
    ], [
      'Classify touched files into JS-only, package/config, native module, GPU/canvas, navigation, or release-risk.',
      'Run the smallest validation set that actually proves the changed runtime surface.',
    ], [
      'Using Expo Go as proof for a module requiring a development build.',
      'Treating `tsc` as validation for native runtime or GPU changes.',
    ], [sourceAnchors.expoDoctor, sourceAnchors.expoDevelopmentBuilds, sourceAnchors.easBuild]),
    supplemental('animation-test-harnesses.md', 'Animation test harness and fixture selection', 'Read when writing or choosing tests for Reanimated, Lottie, Rive, Skia, native navigation, or motion accessibility.', [
      'Unit tests can prove deterministic mapping, reduced-motion branching, and lifecycle guards; they cannot prove native rendering, GPU output, or gesture feel alone.',
      'Use small fixtures for animation-state transitions and save device proof for runtime surfaces that need it.',
      'Fake timers and Reanimated/Jest setup must match the project test harness before relying on timing assertions.',
    ], [
      'Check Jest/Reanimated setup and whether fake timers are already used.',
      'Pair tests with simulator/device proof for native modules, canvas/GPU, or navigation transitions.',
    ], [
      'Testing only snapshots for motion behavior.',
      'Writing sleeps instead of deterministic animation-state assertions.',
    ], [sourceAnchors.reanimatedGettingStarted, sourceAnchors.expoDoctor]),
  ],
};

function supplemental(file, title, whenToLoad, details, checks, antiPatterns, officialSources) {
  return {
    file,
    title,
    bullets: [whenToLoad],
    details,
    checks,
    antiPatterns,
    officialSources,
  };
}

function applySupplementalReferences() {
  for (const data of [...skills.web, ...skills.native]) {
    const seen = new Set(data.references.map((reference) => reference.file));
    for (const reference of supplementalReferences[data.name] ?? []) {
      if (!seen.has(reference.file)) {
        data.references.push(reference);
        seen.add(reference.file);
      }
    }
  }
}

function skill(input) {
  return {
    implicit: true,
    sourceDocs: [],
    ...input,
  };
}

function ref(file, title, bullets) {
  return { file, title, bullets };
}

function ensureDir(dir) {
  mkdirSync(dir, { recursive: true });
}

function write(file, text) {
  ensureDir(path.dirname(file));
  writeFileSync(file, text.endsWith('\n') ? text : `${text}\n`);
}

function read(file) {
  return readFileSync(file, 'utf8');
}

function listFiles(dir) {
  if (!existsSync(dir)) return [];
  const files = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...listFiles(full));
    else if (entry.isFile()) files.push(full);
  }
  return files;
}

function ghContent(ownerRepo, filePath) {
  const raw = execFileSync(
    'gh',
    ['api', `repos/${ownerRepo}/contents/${filePath}`, '--jq', '.content'],
    { encoding: 'utf8' },
  );
  return Buffer.from(raw.replace(/\s/g, ''), 'base64').toString('utf8');
}

function fetchGsapSkill(name) {
  return ghContent('greensock/gsap-skills', `skills/${name}/SKILL.md`).trimEnd();
}

function fetchGsapAsset(asset) {
  return ghContent('greensock/gsap-skills', `assets/${asset}`);
}

function yamlQuote(text) {
  return `"${text.replaceAll('\\', '\\\\').replaceAll('"', '\\"')}"`;
}

function flattenExistingReferences(skillDir, generatedReferences = new Map()) {
  const refsDir = path.join(skillDir, 'references');
  if (!existsSync(refsDir)) return [];
  const flattened = [];
  for (const file of listFiles(refsDir)) {
    const rel = path.relative(refsDir, file);
    if (!rel.includes(path.sep)) continue;
    const parts = rel.split(path.sep);
    const destName = `${parts.slice(0, -1).join('-')}-${parts.at(-1)}`.replace(/[^A-Za-z0-9_.-]/g, '-');
    const dest = path.join(refsDir, destName);
    const sourceText = read(file);
    if (generatedReferences.has(destName) && generatedReferences.get(destName) !== sourceText) {
      throw new Error(`Refusing to flatten ${file}: generated reference ${destName} would overwrite different content`);
    }
    if (existsSync(dest)) {
      const destText = read(dest);
      if (sourceText !== destText) {
        throw new Error(`Refusing to flatten ${file}: destination ${dest} already exists with different content`);
      }
    } else {
      copyFileSync(file, dest);
    }
    flattened.push(destName);
  }
  for (const entry of readdirSync(refsDir, { withFileTypes: true })) {
    if (entry.isDirectory()) rmSync(path.join(refsDir, entry.name), { recursive: true, force: true });
  }
  return flattened.sort();
}

function removeOldTemplateDir(skillDir) {
  rmSync(path.join(skillDir, 'templates'), { recursive: true, force: true });
}

function sourceDocsFor(data, skillDir, flattened) {
  const refsDir = path.join(skillDir, 'references');
  const existing = existsSync(refsDir)
    ? readdirSync(refsDir, { withFileTypes: true })
        .filter((entry) => entry.isFile() && entry.name.endsWith('.md'))
        .map((entry) => entry.name)
    : [];
  const set = new Set(data.sourceDocs);
  const tailored = new Set(data.references.map((reference) => reference.file));
  const reserved = new Set(['index.md', 'source-ledger.md']);
  const available = [...new Set([...flattened, ...existing])].sort();
  const explicit = available.filter((file) => {
    const simple = file.replace(/^docs-/, '').replace(/^software-mansion-animations-/, '');
    return set.has(file) || set.has(simple);
  });
  const extraPortableSourceDocs = available.filter((file) => (
    !explicit.includes(file)
    && !tailored.has(file)
    && !reserved.has(file)
  ));
  return [...explicit, ...extraPortableSourceDocs];
}

function renderSourceLedger(data, sourceDocs) {
  const sources = sourceList(data);
  return `# ${data.name} Source Ledger

Checked at: ${checkedAt}

This ledger is skill-local and portable. It names the upstream sources used for
the bundled guidance, source-backed notes, generated evals, examples, and
audit rules. It intentionally does not reference local scrape paths or machine
cache locations.

## Primary Sources

${sources.map((source) => `- ${source}`).join('\n')}

## Source-Backed Notes

${sourceDocs.length ? sourceDocs.map((file) => `- \`references/${file}\``).join('\n') : '- No additional source-backed note files are required for this skill beyond the official baseline and tailored references.'}

## Tailored Reference Files

${data.references.map((reference) => `- \`references/${reference.file}\` - ${reference.title}`).join('\n')}

## Local Additions

- \`references/provenance.json\` records source URLs, package facts, and copy policy.
- \`scripts/audit.mjs\` provides the repeatable static audit CLI for this skill.
- \`assets/templates/\` contains output templates and review checklists.
- \`assets/examples/\` contains small starter examples or fixtures.
- \`evals/\` contains trigger and task-quality evals.

Use official docs and installed package versions as API truth when they conflict
with older bundled notes.
`;
}

function sourceList(data) {
  if (data.family === 'GSAP') {
    return [
      'GreenSock official GSAP AI skills, MIT: https://github.com/greensock/gsap-skills',
      'GSAP documentation and package license source: https://gsap.com/docs/v3/ and https://gsap.com/standard-license/',
      'Agent Skills specification and best practices: https://agentskills.io/specification',
    ];
  }
  const byFamily = {
    CSS: [
      'MDN CSS Animations, Transitions, and prefers-reduced-motion documentation.',
      'CSSWG drafts for new CSS motion features when MDN notes support limits.',
      'Agent Skills specification and best practices.',
    ],
    WAAPI: [
      'MDN Web Animations API documentation.',
      'Browser compatibility and local support policy.',
      'Agent Skills specification and best practices.',
    ],
    'Tailwind CSS': [
      'Tailwind CSS v4 official documentation and npm package metadata.',
      'MDN prefers-reduced-motion documentation for motion media behavior.',
      'Agent Skills specification and best practices.',
    ],
    Lottie: [
      'lottie-web, lottie-react-native, dotLottie web/native package docs and source metadata.',
      'Expo SDK docs for native package compatibility when applicable.',
      'Agent Skills specification and best practices.',
    ],
    Rive: [
      'Rive web/native runtime package docs and source metadata.',
      'Rive state machine documentation.',
      'Agent Skills specification and best practices.',
    ],
    'Motion React': [
      'Motion for React official documentation.',
      'React and framework SSR/client-boundary docs where applicable.',
      'Agent Skills specification and best practices.',
    ],
    'Three.js/R3F': [
      'Three.js, React Three Fiber, and Drei documentation/source metadata.',
      'Expo/native GPU docs when the skill targets native runtime.',
      'Agent Skills specification and best practices.',
    ],
    WebGPU: [
      'TypeGPU documentation and MIT package source.',
      'MDN/WebGPU browser runtime documentation.',
      'Agent Skills specification and best practices.',
    ],
    Reanimated: [
      'Expo SDK docs, React Native Reanimated, React Native Worklets, and Software Mansion source notes.',
      'React Native accessibility and performance documentation.',
      'Agent Skills specification and best practices.',
    ],
    'React Native': [
      'React Native accessibility/performance docs and Expo docs.',
      'Software Mansion React Native skill notes where license-gated.',
      'Agent Skills specification and best practices.',
    ],
    'Expo UI': [
      'Expo Router, Expo UI, react-native-screens, and React Navigation docs.',
      'Expo SDK compatibility docs.',
      'Agent Skills specification and best practices.',
    ],
    Skia: [
      'React Native Skia package docs/source metadata and Expo SDK docs.',
      'Software Mansion animation/canvas skill notes where license-gated.',
      'Agent Skills specification and best practices.',
    ],
    NativeWind: [
      'NativeWind and react-native-css package docs/source metadata.',
      'Expo Tailwind setup documentation.',
      'Agent Skills specification and best practices.',
    ],
    'Expo validation': [
      'Expo Doctor, EAS Build, Expo development build, and SDK compatibility docs.',
      'React Native testing and Reanimated Jest docs.',
      'Agent Skills specification and best practices.',
    ],
  };
  return byFamily[data.family] ?? ['Official package documentation/source metadata.', 'Agent Skills specification and best practices.'];
}

function renderProvenance(data, sourceDocs) {
  return JSON.stringify({
    skill: data.name,
    checked_at: checkedAt,
    copy_policy: data.family === 'GSAP'
      ? 'GreenSock official GSAP skill text is copied as the primary SKILL.md baseline under the upstream MIT skill repository license; GSAP package/runtime docs remain subject to GSAP package terms and are summarized or cited unless separately license-gated.'
      : 'Bundled references are tailored notes with source links. Do not bundle full third-party prose unless the skill metadata and attribution explicitly cover that source license. Verify official docs before applying version-sensitive API examples.',
    upstream_sources: sourceList(data).map((label) => ({ label })),
    tailored_reference_files: data.references.map((reference) => ({
      file: `references/${reference.file}`,
      title: reference.title,
      sources: reference.officialSources ?? [],
    })),
    source_backed_note_files: sourceDocs.map((file) => `references/${file}`),
    local_files: [
      'SKILL.md',
      'scripts/audit.mjs',
      'assets/templates',
      'assets/examples',
      'evals/evals.json',
      'evals/trigger-queries.json',
    ],
  }, null, 2);
}

function renderReference(data, reference) {
  const sourceSection = reference.officialSources?.length
    ? `\n## Source Anchors\n\n${reference.officialSources.map((source) => `- ${source}`).join('\n')}\n`
    : '';
  const detailSection = reference.details?.length
    ? `\n## Reference Notes\n\n${reference.details.map((line) => `- ${line}`).join('\n')}\n`
    : '';
  const checkSection = reference.checks?.length
    ? `\n## Focused Checks\n\n${reference.checks.map((line) => `- ${line}`).join('\n')}\n`
    : '';
  const antiPatternSection = reference.antiPatterns?.length
    ? `\n## Failure Modes\n\n${reference.antiPatterns.map((line) => `- ${line}`).join('\n')}\n`
    : '';
  return `# ${reference.title}

Skill: ${data.name}
Checked at: ${checkedAt}

## When To Load

${reference.bullets.map((line) => `- ${line}`).join('\n')}
${sourceSection}${detailSection}${checkSection}${antiPatternSection}

## Operating Guidance

${data.focus}

### Decision Boundaries

${data.boundaries.map((line) => `- ${line}`).join('\n')}

### Workflow Details

${data.workflow.map((line, index) => `${index + 1}. ${line}`).join('\n')}

### Gotchas

${data.gotchas.map((line) => `- ${line}`).join('\n')}

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled \`scripts/audit.mjs doctor --root <repo> --format json\` command when setup is unclear.
- Use \`scripts/audit.mjs scan --root <repo> --format markdown\` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
`;
}

function renderNonGsapSkill(data, sourceDocs) {
  const resourceRows = [
    ...data.references.map((r) => `- \`references/${r.file}\` - ${r.title}. ${r.bullets[0]}`),
    ...sourceDocs.map((file) => `- \`references/${file}\` - Source-backed notes and links. Load when exact upstream API detail may have changed.`),
    '- `references/index.md` - Complete reference inventory and routing summary.',
    '- `references/source-ledger.md` - Source list, checked date, and copy policy.',
    '- `references/provenance.json` - Machine-readable source and local-resource metadata.',
    '- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.',
    `- \`assets/templates/${data.name}-audit-report.md\` - Audit response/report template.`,
    `- \`assets/templates/${data.name}-review-checklist.md\` - Manual review checklist.`,
    `- \`assets/examples/${data.name}-starter.${data.exampleExt}\` - Starter fixture/example for this skill.`,
    '- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.',
    '- `evals/evals.json` - Task-quality evals with assertions.',
  ];
  return `---
name: ${data.name}
description: >-
  Use this skill for ${data.focus} Trigger on ${data.triggers.join(', ')}. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# ${data.title}

${data.focus}

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

${data.boundaries.map((line) => `- ${line}`).join('\n')}

## Workflow

${data.workflow.map((line, index) => `${index + 1}. ${line}`).join('\n')}

## Gotchas

${data.gotchas.map((line) => `- ${line}`).join('\n')}

<!-- skill-resources:start -->
## Bundled Resources

${resourceRows.join('\n')}
<!-- skill-resources:end -->

## Audit CLI

\`\`\`bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output ${data.name}-audit.json
\`\`\`

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
`;
}

function renderGsapOverlay(data, sourceDocs) {
  const rows = [
    ...data.references.map((r) => `- \`references/${r.file}\` - ${r.title}. ${r.bullets[0]}`),
    ...sourceDocs.map((file) => `- \`references/${file}\` - Source-backed notes and links. Load when exact upstream API detail may have changed.`),
    '- `references/index.md` - Complete reference inventory and routing summary.',
    '- `references/source-ledger.md` - Portable source list and copy policy.',
    '- `references/provenance.json` - Machine-readable provenance and local-resource metadata.',
    '- `scripts/audit.mjs` - Self-contained Codex audit CLI with domain-specific GSAP rules.',
    `- \`assets/templates/${data.name}-audit-report.md\` - GSAP audit response template.`,
    `- \`assets/templates/${data.name}-review-checklist.md\` - GSAP manual review checklist.`,
    `- \`assets/examples/${data.name}-starter.${data.exampleExt}\` - Minimal starter fixture/example.`,
    '- `evals/trigger-queries.json` - Trigger/near-miss eval set.',
    '- `evals/evals.json` - Task-quality evals with assertions.',
  ];
  return `

---

## Codex Web Motion Overlay

The upstream GreenSock official skill content above is the primary GSAP
guidance. This local overlay adds Codex-specific progressive-disclosure
resources, static audit scripts, evals, and portable source metadata. Keep GSAP
API behavior aligned with GreenSock's official skill and docs; use this overlay
for validation, local boundaries, and report shape.

### Local Boundaries

${data.boundaries.map((line) => `- ${line}`).join('\n')}

### Local Workflow

${data.workflow.map((line, index) => `${index + 1}. ${line}`).join('\n')}

### Local Gotchas

${data.gotchas.map((line) => `- ${line}`).join('\n')}

<!-- skill-resources:start -->
### Bundled Resources

${rows.join('\n')}
<!-- skill-resources:end -->

### Audit CLI

\`\`\`bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output ${data.name}-audit.json
\`\`\`

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.
`;
}

function renderAuditReport(data) {
  return `# ${data.title} Audit Report

## Scope

- Target repo/root:
- Skill: ${data.name}
- Audit command:
- Date:
- Reviewer:
- Installed packages/versions:
- Runtime/platforms checked:

## Summary

- High:
- Medium:
- Low:
- Overall status:

## Findings

### [severity] [ruleId] Short Title

- File:
- Line:
- Confidence:
- Evidence:
- Why it matters for ${data.family}:
- Recommendation:
- Validation required:
- Status:

## Validation

- Commands run:
- Source/package versions checked:
- Runtime/browser/device proof:
- Accessibility/reduced-motion proof:
- Skipped checks and reason:

## Residual Risk

- Accepted tradeoffs:
- Follow-up owner:
`;
}

function renderChecklist(data) {
  return `# ${data.title} Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run \`node scripts/audit.mjs doctor --root <repo> --format json\` when setup is unclear.

## Manual Review

${data.workflow.map((line) => `- [ ] ${line}`).join('\n')}
${data.gotchas.map((line) => `- [ ] Check gotcha: ${line}`).join('\n')}
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run \`node scripts/audit.mjs scan --root <repo> --format markdown\` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
`;
}

function renderOpenAI(data) {
  return `interface:
  display_name: ${yamlQuote(data.title)}
  short_description: ${yamlQuote(data.short)}
  default_prompt: ${yamlQuote(`Use $${data.name} for ${data.focus}`)}
policy:
  allow_implicit_invocation: ${data.implicit ? 'true' : 'false'}
`;
}

function renderEval(data, index) {
  const prompts = [
    `In src/components/Hero.${data.exampleExt === 'tsx' ? 'tsx' : 'ts'}, implement ${data.triggers[0]} behavior for a product surface and explain the validation gates.`,
    `Review this branch for ${data.family} motion issues: package versions look current but users report jank and missing reduced-motion behavior.`,
    `Create a minimal example using ${data.triggers[1] ?? data.triggers[0]} that includes cleanup and accessibility notes.`,
    `Migrate an existing motion implementation to the right ${data.family} primitive without changing unrelated UI.`,
  ];
  return {
    id: `${data.name}-task-${index + 1}`,
    prompt: prompts[index],
    expected_output: `Uses ${data.name} routing, checks installed versions, applies domain-specific ${data.family} guidance, runs or recommends the bundled audit CLI where relevant, and reports validation evidence.`,
    assertions: [
      `Mentions the relevant installed package/runtime/version checks for ${data.family}.`,
      `Uses at least one ${data.name} bundled reference or explains why no extra reference was needed.`,
      `Includes reduced-motion/accessibility or explains why the effect is essential and already covered.`,
      `Includes cleanup/interruption/runtime validation appropriate to ${data.focus}.`,
      `Does not route the task to an unrelated near-miss skill.`,
    ],
  };
}

function renderEvals(data) {
  return JSON.stringify({ evals: [0, 1, 2, 3].map((i) => renderEval(data, i)) }, null, 2);
}

function renderTriggerQueries(data) {
  const positives = data.triggers.map((trigger, index) => ({
    query: `Can you help with ${trigger} in our app and make sure it has cleanup, reduced-motion behavior, and validation?`,
    should_trigger: true,
    reason: `${trigger} is directly in scope for ${data.name}.`,
  }));
  while (positives.length < 10) {
    const index = positives.length + 1;
    positives.push({
      query: `Review ${data.name} scenario ${index}: this ${data.family} implementation uses ${data.focus.split(',')[0].toLowerCase()} and needs an audit report.`,
      should_trigger: true,
      reason: `The task asks for ${data.family} motion expertise owned by ${data.name}.`,
    });
  }
  const boundaryNegatives = data.boundaries.map((boundary) => `I need ${boundary.replace(/^Use /, '').replace(/\.$/, '')} instead of ${data.name}`);
  const sharedSlots = Math.max(0, 10 - boundaryNegatives.length);
  const negatives = [
    ...boundaryNegatives,
    ...sharedNegativeQueries.slice(0, sharedSlots),
  ].slice(0, 10).map((query) => ({
    query,
    should_trigger: false,
    reason: `Near miss or unrelated task for ${data.name}.`,
  }));
  const unique = [];
  const seen = new Set();
  for (const query of [...positives.slice(0, 10), ...negatives]) {
    if (seen.has(query.query)) continue;
    seen.add(query.query);
    unique.push(query);
  }
  return JSON.stringify({ queries: unique }, null, 2);
}

function renderAuditWrapper(data) {
  return `#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';

const shared = path.resolve(import.meta.dirname, '..', '..', '..', 'scripts', 'motion-skillkit.mjs');
const result = spawnSync(process.execPath, [shared, ...process.argv.slice(2)], { stdio: 'inherit' });
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
`;
}

function renderReferenceIndex(data, sourceDocs) {
  return `# ${data.title} Reference Index

Read only the file that matches the task. Keep SKILL.md as the router.

## Tailored References

${data.references.map((r) => `- \`${r.file}\` - ${r.title}.`).join('\n')}

## Source-Backed Notes

${sourceDocs.length ? sourceDocs.map((file) => `- \`${file}\``).join('\n') : '- None required for this skill.'}
`;
}

function processSkill(pluginKey, data) {
  const pluginRoot = pluginRoots[pluginKey];
  const skillDir = path.join(pluginRoot, 'skills', data.name);
  ensureDir(skillDir);
  const generatedReferences = new Map(
    data.references.map((reference) => [reference.file, renderReference(data, reference)]),
  );
  const flattened = flattenExistingReferences(skillDir, generatedReferences);
  removeOldTemplateDir(skillDir);
  const sourceDocs = sourceDocsFor(data, skillDir, flattened);

  for (const reference of data.references) {
    write(path.join(skillDir, 'references', reference.file), generatedReferences.get(reference.file));
  }
  write(path.join(skillDir, 'references', 'index.md'), renderReferenceIndex(data, sourceDocs));
  write(path.join(skillDir, 'references', 'source-ledger.md'), renderSourceLedger(data, sourceDocs));
  write(path.join(skillDir, 'references', 'provenance.json'), renderProvenance(data, sourceDocs));

  write(path.join(skillDir, 'assets', 'templates', `${data.name}-audit-report.md`), renderAuditReport(data));
  write(path.join(skillDir, 'assets', 'templates', `${data.name}-review-checklist.md`), renderChecklist(data));
  write(path.join(skillDir, 'assets', 'examples', `${data.name}-starter.${data.exampleExt}`), data.example);

  write(path.join(skillDir, 'evals', 'evals.json'), renderEvals(data));
  write(path.join(skillDir, 'evals', 'trigger-queries.json'), renderTriggerQueries(data));
  write(path.join(skillDir, 'agents', 'openai.yaml'), renderOpenAI(data));
  const auditScript = path.join(skillDir, 'scripts', 'audit.mjs');
  if (!existsSync(auditScript)) write(auditScript, renderAuditWrapper(data));

  if (gsapSkills.has(data.name)) {
    const official = fetchGsapSkill(data.name);
    write(path.join(skillDir, 'SKILL.md'), `${official}${renderGsapOverlay(data, sourceDocs)}`);
  } else {
    write(path.join(skillDir, 'SKILL.md'), renderNonGsapSkill(data, sourceDocs));
  }
}

function updateGsapPluginAssets() {
  const dir = path.join(pluginRoots.web, 'assets', 'gsap');
  ensureDir(dir);
  for (const asset of ['gsap-green.svg', 'gsap-icon-inverted.svg', 'gsap-icon-square.svg', 'gsap-white.svg']) {
    write(path.join(dir, asset), fetchGsapAsset(asset));
  }
  write(
    path.join(pluginRoots.web, 'licenses', 'greensock-gsap-skills-MIT.txt'),
    ghContent('greensock/gsap-skills', 'LICENSE'),
  );
}

function main() {
  applySupplementalReferences();
  updateGsapPluginAssets();
  for (const data of skills.web) processSkill('web', data);
  for (const data of skills.native) processSkill('native', data);
}

main();
