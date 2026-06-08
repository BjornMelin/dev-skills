[Motion](https://motion.dev/)

[Docs](https://motion.dev/docs) [Examples](https://motion.dev/examples) [Tutorials](https://motion.dev/tutorials) [AI Kit](https://motion.dev/docs/ai-kit) [Motion+](https://motion.dev/plus)

[Docs](https://motion.dev/docs)/ [React](https://motion.dev/docs/react)

# Get started with Motion for React

Install Motion for React and animate elements with springs.

Copy pageCopy page

DocsGet started with Motion for ReactReactJavaScriptVueAIAI Kit

[Get started](https://motion.dev/docs/react)

[›Why Motion for React?](https://motion.dev/docs/react#why-motion-for-react) [›Install](https://motion.dev/docs/react#install) [›Create your first animation](https://motion.dev/docs/react#create-your-first-animation) [›Enter animation](https://motion.dev/docs/react#enter-animation) [›Hover & tap animation](https://motion.dev/docs/react#hover--tap-animation) [›Scroll animation](https://motion.dev/docs/react#scroll-animation) [›Layout animation](https://motion.dev/docs/react#layout-animation) [›Exit animations](https://motion.dev/docs/react#exit-animations) [›SVG animations](https://motion.dev/docs/react#svg-animations) [›Development tools](https://motion.dev/docs/react#development-tools) [›Learn next](https://motion.dev/docs/react#learn-next) [›Related topics](https://motion.dev/docs/react#docs-related-title)

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

[>Presented by](https://clerk.com/?utm_campaign=motion-react-start-guide)

**Motion for React** (previously Framer Motion) is a React animation library for building smooth, production-grade UI animations. You can start with simple prop-based animations before growing to layout, gesture and scroll animations.

Motion's hybrid engine runs animations natively in the browser using the Web Animations API and ScrollTimeline for 120fps performance. When you need capabilities those APIs can't provide (like spring physics, interruptible keyframes, or gesture tracking) it seamlessly falls back to JavaScript.

Motion is trusted by companies like [Framer](https://framer.com/) and [Figma](https://figma.com/) to power animations for their millions of users, and has over 30 million downloads per month on [npm](https://www.npmjs.com/package/framer-motion).

In this guide, we'll learn **why** and **when** you should use Motion, how to **install** it, and give you an overview of its main features.

## [Why Motion for React?](https://motion.dev/docs/react\#why-motion-for-react)

React gives you the power to build dynamic user interfaces, but orchestrating complex, performant animations can be a challenge. Motion is a production-ready React animation library designed to solve this problem, making it simple to create everything from beautiful micro-interactions to complex, gesture-driven animations.

```
import { motion } from "motion/react"

function Component() {
  return <motion.button animate={{ opacity: 1 }} />
}
```

### [Key advantages](https://motion.dev/docs/react\#key-advantages)

Here’s when it’s the right choice for your project.

- **Built for React.** While other animation libraries like [GSAP](https://motion.dev/docs/gsap-vs-motion) are messy to integrate with React, Motion's declarative API is a natural fit. Animations can be linked directly to state and props.

- **Hardware-acceleration.** Motion leverages the same high-performance browser animations as CSS, ensuring your UIs stay smooth and snappy. 120fps animations with a much simpler and more expressive API.

- **Animate anything.** CSS has hard limits. Values you can't animate, keyframes you can't interrupt, staggers that must be hardcoded. Motion provides a single, consistent API that scales from simple to complex.

- **App-like gestures.** Standard CSS `:hover` events are unreliable on touch devices. Motion provides robust, cross-device gesture recognisers for tap, drag, and hover that feel native and intuitive on any device.

- **Production ready.** Built on TypeScript, surrounded by an extensive test suite, and fully tree-shakable so you only include what you import.


### [When is CSS a better choice?](https://motion.dev/docs/react\#when-is-css-a-better-choice)

For simple, self-contained effects (like a color change on hover) a standard CSS transition is a lightweight solution. The strength of Motion is that it can do these simple kinds of animations but also scale to anything you can imagine. All with the same easy to write and maintain API.

## [Install](https://motion.dev/docs/react\#install)

Motion is available via [npm](https://www.npmjs.com/package/motion):

```
npm install motion
```

Features can now be imported via `"motion/react"`:

```
import { motion } from "motion/react"
```

Prefer to install via CDN, or looking for framework-specific instructions? Check out our [full installation guide](https://motion.dev/docs/react-installation).

## [Create your first animation](https://motion.dev/docs/react\#create-your-first-animation)

The `<motion />` component is the foundation of Motion for React. Prefix any HTML or SVG tag with `motion.` to unlock animation props like `animate`, `whileHover`, and `exit`:

```
<motion.ul animate={{ rotate: 360 }} />
```

>Live example [Open](https://examples.motion.dev/react/rotate)

Rotate — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0Aimport+*+as+motion+from+%22motion%2Freact-client%22%0A%0Aexport+default+function+Rotate%28%29+%7B%0Areturn+%28%0A%3Cmotion.div%0Astyle%3D%7Bbox%7D%0Aanimate%3D%7B%7B+rotate%3A+360+%7D%7D%0Atransition%3D%7B%7B+duration%3A+1+%7D%7D%0A%2F%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+box+%3D+%7B%0Awidth%3A+100%2C%0Aheight%3A+100%2C%0AbackgroundColor%3A+%22var%28--hue-1%29%22%2C%0AborderRadius%3A+5%2C%0A%7D%0A%0A%60%60%60) [Tutorial](https://motion.dev/tutorials/react-rotate)

When values in `animate` change, Motion automatically transitions between them.

Physical properties like `x` and `scale` use spring physics by default; visual properties like `opacity` use tween easing. Override the animation type, duration, easing, or delay via [the](https://motion.dev/docs/react-transitions)`transition` [prop](https://motion.dev/docs/react-transitions):

```
<motion.div
  animate={{
    scale: 2,
    transition: { duration: 2 }
  }}
/>
```

[Learn more about React animation](https://motion.dev/docs/react-animation)

If you're the kind of developer who learns better by doing, check out our library of [Basics examples](https://motion.dev/examples#basics). Each comes complete with a live demo and copy/paste source code.

## [Enter animation](https://motion.dev/docs/react\#enter-animation)

When a component enters the page, it will automatically animate to the values defined in the `animate` prop.

You can provide values to animate from via the `initial` prop (otherwise these will be read from the DOM).

```
<motion.button initial={{ scale: 0 }} animate={{ scale: 1 }} />
```

>Live example [Open](https://examples.motion.dev/react/enter-animation)

Enter animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0Aimport+*+as+motion+from+%22motion%2Freact-client%22%0A%0Aexport+default+function+EnterAnimation%28%29+%7B%0Areturn+%28%0A%3Cmotion.div%0Ainitial%3D%7B%7B+opacity%3A+0%2C+scale%3A+0+%7D%7D%0Aanimate%3D%7B%7B+opacity%3A+1%2C+scale%3A+1+%7D%7D%0Atransition%3D%7B%7B%0Aduration%3A+0.4%2C%0Ascale%3A+%7B+type%3A+%22spring%22%2C+visualDuration%3A+0.4%2C+bounce%3A+0.5+%7D%2C%0A%7D%7D%0Astyle%3D%7Bball%7D%0A%2F%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+ball+%3D+%7B%0Awidth%3A+100%2C%0Aheight%3A+100%2C%0AbackgroundColor%3A+%22var%28--hue-2%29%22%2C%0AborderRadius%3A+%2250%25%22%2C%0A%7D%0A%0A%60%60%60) [Tutorial](https://motion.dev/tutorials/react-enter-animation)

Or disable this initial animation entirely by setting `initial` to `false`.

```
<motion.button initial={false} animate={{ scale: 1 }} />
```

## [Hover & tap animation](https://motion.dev/docs/react\#hover--tap-animation)

`<motion />` extends React's event system with powerful [gesture animations](https://motion.dev/docs/react-gestures). It currently supports hover, tap, focus, and drag.

```
<motion.button
  whileHover={{ scale: 1.1 }}
  whileTap={{ scale: 0.95 }}
  onHoverStart={() => console.log('hover started!')}
/>
```

>Live example [Open](https://examples.motion.dev/react/gestures)

Gestures — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0Aimport+*+as+motion+from+%22motion%2Freact-client%22%0A%0Aexport+default+function+Gestures%28%29+%7B%0Areturn+%28%0A%3Cmotion.div%0AwhileHover%3D%7B%7B+scale%3A+1.2+%7D%7D%0AwhileTap%3D%7B%7B+scale%3A+0.8+%7D%7D%0Astyle%3D%7Bbox%7D%0A%2F%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+box+%3D+%7B%0Awidth%3A+100%2C%0Aheight%3A+100%2C%0AbackgroundColor%3A+%22var%28--hue-3%29%22%2C%0AborderRadius%3A+5%2C%0A%7D%0A%0A%60%60%60) [Tutorial](https://motion.dev/tutorials/react-gestures)

Motion's gestures are designed to feel better than using CSS or JavaScript events alone.

## [Scroll animation](https://motion.dev/docs/react\#scroll-animation)

Motion supports both types of [scroll animations](https://motion.dev/docs/react-scroll-animations): **Scroll-triggered** and **scroll-linked**.

To trigger an animation on scroll, the `whileInView` prop defines a state to animate to/from when an element enters/leaves the viewport:

```
<motion.div
  initial={{ backgroundColor: "rgb(0, 255, 0)", opacity: 0 }}
  whileInView={{ backgroundColor: "rgb(255, 0, 0)", opacity: 1 }}
/>
```

>Live example [Open](https://examples.motion.dev/react/scroll-triggered)

Whereas to link a value directly to scroll position, it's possible to use `MotionValue`s via `useScroll`.

```
const { scrollYProgress } = useScroll()

return <motion.div style={{ scaleX: scrollYProgress }} />
```

>Live example [Open](https://examples.motion.dev/react/scroll-linked)

## [Layout animation](https://motion.dev/docs/react\#layout-animation)

Motion's [layout animation](https://motion.dev/docs/react-layout-animations) engine detects layout changes (size, position, reorder) and smoothly animates between states using transforms. Unlike basic "FLIP" implementations, it does so while correcting for scale-distortion.

It's as easy as applying the `layout` prop.

```
<motion.div layout />
```

>Live example [Open](https://examples.motion.dev/react/layout-animation)

Or to animate between completely different elements, a `layoutId`:

```
<motion.div layoutId="underline" />
```

>Live example [Open](https://examples.motion.dev/react/shared-layout-animation)

## [Exit animations](https://motion.dev/docs/react\#exit-animations)

By wrapping `motion` components with `<AnimatePresence>` we gain access to [exit animations](https://motion.dev/docs/react-animate-presence). This allows us to animate elements as they're removed from the DOM.

```
<AnimatePresence>
  {show ? <motion.div key="box" exit={{ opacity: 0 }} /> : null}
</AnimatePresence>
```

>Live example [Open](https://examples.motion.dev/react/exit-animation)

## [SVG animations](https://motion.dev/docs/react\#svg-animations)

Motion has full support for [SVG animations](https://motion.dev/docs/react-svg-animation), including support for animating `viewBox` and special values for simple path drawing effects.

```
<motion.circle animate={{ pathLength: 1 }} />
```

>Live example [Open](https://examples.motion.dev/react/use-transform)

## [Development tools](https://motion.dev/docs/react\#development-tools)

Enhance your animation workflow with a suite of Motion developer tools. [Motion+](https://motion.dev/plus) provides access to the `/motion` skill, which helps your agent access the latest Motion docs, build from over 370+ examples, find and fix animation performance issues in your codebase and much more.

>Motion+ · AI Kit

### One click install for Cursor

Add powerful animation skills and MCP to Cursor with one click. Motion AI Kit requires [Motion+](https://motion.dev/plus) to install.

[Add AI Kit](cursor://anysphere.cursor-deeplink/prompt?text=Install+the+Motion+AI+Kit+for+Cursor.%0A%0A1.+Ask+me+for+my+Motion%2B+API+key+if+MOTION_TOKEN+is+not+already+set+%28from+https%3A%2F%2Fmotion.dev%2Fdashboard%2Ftokens%29.%0A%0AConfigure+the+Motion+MCP+server+globally+for+Cursor%3A%0A%0AMOTION_TOKEN%3D%22%3Ctoken%3E%22+npx+-y+add-mcp+%5C%0A++%22npx+-y+https%3A%2F%2Fapi.motion.dev%2Fregistry.tgz%3Fpackage%3Dmotion-studio-mcp%26version%3Dlatest%22+%5C%0A++--name+motion+%5C%0A++--env+TOKEN%3D%22%24MOTION_TOKEN%22+%5C%0A++-y+-g+-a+cursor%0AThis+should+update+%7E%2F.cursor%2Fmcp.json.%0A%0AInstall+the+Motion+skills+into+%7E%2F.cursor%2Fskills%2F+by+fetching%3A+https%3A%2F%2Fapi.motion.dev%2Fregistry%2Fskills%2Fmotion-ai-kit%3Ftoken%3D%3Ctoken%3E+Parse+the+skill+files+from+the+bundle+and+write+them+to%3A%0A%0A%7E%2F.cursor%2Fskills%2Fmotion%2F%0AVerify%3A%0A%0A%7E%2F.cursor%2Fmcp.json+has+a+motion+server+with+env.TOKEN%0AThe+motion+skill+folder+exists+under+%7E%2F.cursor%2Fskills%2F%0ATell+me+to+fully+restart+Cursor.%0A%0ADo+not+touch+unrelated+skills+already+in+%7E%2F.cursor%2Fskills%2F.)

Motion AI Kit is also available for Claude Code, Codex, and other popular agents. [See full installation guide](https://motion.dev/docs/ai-kit-install).

## [Learn next](https://motion.dev/docs/react\#learn-next)

That covers the core building blocks. Here's where to go next based on what you want to build and your learning style.

The [React animation](https://motion.dev/docs/react-animation) guide will teach you more about the different types of animations you can build with this React animation library.

Or, you can learn by doing, diving straight into our collection of [examples](https://motion.dev/examples?platform=react&category=basics). Each comes complete with full source code that you can copy-paste into your project.

## Related topics

- [Installation guide for Motion for React→\\
\\
Install Motion in your React project.](https://motion.dev/docs/react-installation)
- [React animation→\\
\\
An overview of animating React with motion components, variants, gestures, and keyframes.](https://motion.dev/docs/react-animation)
- [React scroll animation→\\
\\
Scroll-triggered and scroll-linked effects in React: parallax, progress, and more.](https://motion.dev/docs/react-scroll-animations)
- [Layout animation→\\
\\
Smoothly animate layout changes and shared element transitions.](https://motion.dev/docs/react-layout-animations)
- [Rotate→\\
\\
An example of animation the rotation of an element with Motion for React](https://motion.dev/tutorials/react-rotate)

[Motion+370+ examplesLifetime updates\\
\\
Motion+ **Level up your animations.** \\
\\
Unlock 370+ premium examples, premium APIs, private Discord and GitHub, and a transition editor for your IDE. One-time purchase, lifetime updates.\\
\\
PricingOne-time payment, lifetime updates\\
\\
Get Motion+](https://motion.dev/plus)

[NextReact animation](https://motion.dev/docs/react-animation)

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