[Motion](https://motion.dev/)

[Docs](https://motion.dev/docs) [Examples](https://motion.dev/examples) [Tutorials](https://motion.dev/tutorials) [AI Kit](https://motion.dev/docs/ai-kit) [Motion+](https://motion.dev/plus)

[Docs](https://motion.dev/docs)/ [React](https://motion.dev/docs/react)

# useReducedMotion

Adapt or disable animations based on the device's Reduced Motion setting.

Copy pageCopy page

DocsuseReducedMotionReactJavaScriptVueAIAI Kit

[Get started](https://motion.dev/docs/react)

Animations

[Overview](https://motion.dev/docs/react-animation)

[Layout](https://motion.dev/docs/react-layout-animations)

[Scroll](https://motion.dev/docs/react-scroll-animations)

[SVG](https://motion.dev/docs/react-svg-animation)

[Transitions](https://motion.dev/docs/react-transitions)

Gestures

[Overview](https://motion.dev/docs/react-gestures)

[Drag](https://motion.dev/docs/react-drag)

[Hover](https://motion.dev/docs/react-hover-animation)

Components

[`<motion>`](https://motion.dev/docs/react-motion-component)

[`<AnimateActivity>`](https://motion.dev/docs/react-animate-activity)

[`<AnimatePresence>`](https://motion.dev/docs/react-animate-presence)

[`<AnimateView>`](https://motion.dev/docs/react-animate-view)

[`<LayoutGroup>`](https://motion.dev/docs/react-layout-group)

[`<LazyMotion>`](https://motion.dev/docs/react-lazy-motion)

[`<MotionConfig>`](https://motion.dev/docs/react-motion-config)

[`<Reorder>`](https://motion.dev/docs/react-reorder)

Motion+

[`<AnimateNumber>`](https://motion.dev/docs/react-animate-number)

[`<Carousel>`](https://motion.dev/docs/react-carousel)

[`<Cursor>`](https://motion.dev/docs/cursor)

[`<ScrambleText>`](https://motion.dev/docs/react-scramble-text)

[`<Ticker>`](https://motion.dev/docs/react-ticker)

[`<Typewriter>`](https://motion.dev/docs/react-typewriter)

Motion Values

[Overview](https://motion.dev/docs/react-motion-value)

[useMotionTemplate](https://motion.dev/docs/react-use-motion-template)

[useMotionValueEvent](https://motion.dev/docs/react-use-motion-value-event)

[useScroll](https://motion.dev/docs/react-use-scroll)

[useSpring](https://motion.dev/docs/react-use-spring)

[useTime](https://motion.dev/docs/react-use-time)

[useTransform](https://motion.dev/docs/react-use-transform)

[useVelocity](https://motion.dev/docs/react-use-velocity)

Hooks

[useAnimate](https://motion.dev/docs/react-use-animate)

[useAnimationFrame](https://motion.dev/docs/react-use-animation-frame)

[useDragControls](https://motion.dev/docs/react-use-drag-controls)

[useInView](https://motion.dev/docs/react-use-in-view)

[usePageInView](https://motion.dev/docs/react-use-page-in-view)

[useReducedMotion](https://motion.dev/docs/react-use-reduced-motion)

[›Usage](https://motion.dev/docs/react-use-reduced-motion#usage) [›Related topics](https://motion.dev/docs/react-use-reduced-motion#docs-related-title)

Integrations

[Framer](https://motion.dev/docs/framer)

[Figma](https://motion.dev/docs/figma)

[Tailwind CSS](https://motion.dev/docs/react-tailwind)

[Base UI](https://motion.dev/docs/base-ui)

[Radix](https://motion.dev/docs/radix)

Guides

[Installation](https://motion.dev/docs/react-installation)

[Accessibility](https://motion.dev/docs/react-accessibility)

[Reduce bundle size](https://motion.dev/docs/react-reduce-bundle-size)

[Upgrade guide](https://motion.dev/docs/react-upgrade-guide)

A hook that returns `true` if the current device has Reduced Motion setting enabled.

```
const shouldReduceMotion = useReducedMotion()
```

This can be used to implement changes to your UI based on Reduced Motion. For instance, replacing potentially motion-sickness inducing `x`/`y` animations with `opacity`, disabling the autoplay of background videos, or turning off parallax motion.

It will actively respond to changes and re-render your components with the latest setting.

```
export function Sidebar({ isOpen }) {
  const shouldReduceMotion = useReducedMotion()
  const closedX = shouldReduceMotion ? 0 : "-100%"

  return (
    <motion.div animate={{
      opacity: isOpen ? 1 : 0,
      x: isOpen ? 0 : closedX
    }} />
  )
}
```

## [Usage](https://motion.dev/docs/react-use-reduced-motion\#usage)

Import `useReducedMotion` from Motion:

```
import { useReducedMotion } from "motion/react"
```

In any component, call `useReducedMotion` to check whether the device's Reduced Motion setting is enabled.

```
const prefersReducedMotion = useReducedMotion()
```

You can then use this `true`/`false` value to change your application logic.

## Related topics

- [React animation→\\
\\
An overview of animating React with motion components, variants, gestures, and keyframes.](https://motion.dev/docs/react-animation)
- [Motion component→\\
\\
Animate elements with a declarative API. Supports variants, gestures, and layout animations.](https://motion.dev/docs/react-motion-component)
- [Accessibility→\\
\\
Respect users' Reduced Motion preferences with the reducedMotion option and useReducedMotion hook.](https://motion.dev/docs/react-accessibility)

[Motion+370+ examplesLifetime updates\\
\\
Motion+ **Level up your animations.** \\
\\
Unlock 370+ premium examples, premium APIs, private Discord and GitHub, and a transition editor for your IDE. One-time purchase, lifetime updates.\\
\\
PricingOne-time payment, lifetime updates\\
\\
Get Motion+](https://motion.dev/plus)

[PrevioususePageInView](https://motion.dev/docs/react-use-page-in-view) [NextMotion x Framer integration guide](https://motion.dev/docs/framer)

### Sponsors

Motion is supported by the best in the industry.

[Become a sponsor](https://motion.dev/sponsor)

[Framer (opens in new window)](https://framer.link/6ogjBZd)[Cursor (opens in new window)](https://cursor.com/)[Linear (opens in new window)](https://linear.app/)[Clerk (opens in new window)](https://clerk.com/?utm_campaign=motion)[Figma (opens in new window)](https://figma.com/)[Sanity (opens in new window)](https://www.sanity.io/)

### Subscribe

Updates on new features and releases.

Subscribe

###### Site

- [About](https://motion.dev/about)
- [Changelog](https://motion.dev/changelog)
- [Docs](https://motion.dev/docs)
- [Examples](https://motion.dev/examples)
- [Magazine](https://motion.dev/magazine)
- [Sponsor](https://motion.dev/sponsor)
- [Troubleshooting](https://motion.dev/troubleshooting)
- [Tutorials](https://motion.dev/tutorials)

###### Products

- [AI Kit](https://motion.dev/docs/ai-kit)
- [CSS Studio](https://cssstudio.ai/)
- [Motion](https://motion.dev/)
- [Motion+](https://motion.dev/plus)
- [MotionScore](https://score.motion.dev/)

###### Most Popular

- [React animation](https://motion.dev/docs/react-animation)
- [Layout animation](https://motion.dev/docs/react-layout-animations)
- [SVG animation](https://motion.dev/docs/react-svg-animation)
- [Motion component](https://motion.dev/docs/react-motion-component)
- [GSAP vs Motion](https://motion.dev/docs/gsap-vs-motion)

###### Docs

- [JavaScript](https://motion.dev/docs/quick-start)
- [React](https://motion.dev/docs/react)
- [Vue](https://motion.dev/docs/vue)
- [AI Kit](https://motion.dev/docs/ai-kit)

###### Social

- [Discord](https://motion.dev/plus)
- [GitHub](https://github.com/motiondivision/motion)
- [X/Twitter](https://x.com/motiondotdev)
- [YouTube](https://www.youtube.com/@motiondotdev)

© 2026 Motion

+ [Login](https://motion.dev/login) [Purchase](https://motion.dev/plus)

[![MotionScore](https://api.motion.dev/score/badge?url=motion.dev)](https://score.motion.dev/site/motion.dev)