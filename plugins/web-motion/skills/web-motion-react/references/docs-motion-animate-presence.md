[Motion](https://motion.dev/)

[Docs](https://motion.dev/docs) [Examples](https://motion.dev/examples) [Tutorials](https://motion.dev/tutorials) [AI Kit](https://motion.dev/docs/ai-kit) [Motion+](https://motion.dev/plus)

[Docs](https://motion.dev/docs)/ [React](https://motion.dev/docs/react)

# AnimatePresence

Run exit animations on React components when they're removed from the page.

Copy pageCopy page

DocsAnimatePresenceReactJavaScriptVueAIAI Kit

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

[›Usage](https://motion.dev/docs/react-animate-presence#usage) [›Props](https://motion.dev/docs/react-animate-presence#props) [›Troubleshooting](https://motion.dev/docs/react-animate-presence#troubleshooting) [›Related topics](https://motion.dev/docs/react-animate-presence#docs-related-title)

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

`AnimatePresence` makes exit animations easy. By wrapping one or more `motion` [components](https://motion.dev/docs/react-motion-component) with `AnimatePresence`, we gain access to the `exit` animation prop.

```
<AnimatePresence>
  {show && <motion.div key="modal" exit={{ opacity: 0 }} />}
</AnimatePresence>
```

>Live example [Open](https://examples.motion.dev/react/exit-animation)

Exit animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+AnimatePresence%2C+motion+%7D+from+%22motion%2Freact%22%0Aimport+%7B+useState+%7D+from+%22react%22%0A%0Aexport+default+function+ExitAnimation%28%29+%7B%0Aconst+%5BisVisible%2C+setIsVisible%5D+%3D+useState%28true%29%0A%0Areturn+%28%0A%3Cdiv+style%3D%7Bcontainer%7D%3E%0A%3CAnimatePresence+initial%3D%7Bfalse%7D%3E%0A%7BisVisible+%3F+%28%0A%3Cmotion.div%0Ainitial%3D%7B%7B+opacity%3A+0%2C+scale%3A+0+%7D%7D%0Aanimate%3D%7B%7B+opacity%3A+1%2C+scale%3A+1+%7D%7D%0Aexit%3D%7B%7B+opacity%3A+0%2C+scale%3A+0+%7D%7D%0Astyle%3D%7Bbox%7D%0Akey%3D%22box%22%0A%2F%3E%0A%29+%3A+null%7D%0A%3C%2FAnimatePresence%3E%0A%3Cmotion.button%0Astyle%3D%7Bbutton%7D%0AonClick%3D%7B%28%29+%3D%3E+setIsVisible%28%21isVisible%29%7D%0AwhileTap%3D%7B%7B+y%3A+1+%7D%7D%0A%3E%0A%7BisVisible+%3F+%22Hide%22+%3A+%22Show%22%7D%0A%3C%2Fmotion.button%3E%0A%3C%2Fdiv%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+container%3A+React.CSSProperties+%3D+%7B%0Adisplay%3A+%22flex%22%2C%0AflexDirection%3A+%22column%22%2C%0Awidth%3A+100%2C%0Aheight%3A+160%2C%0Aposition%3A+%22relative%22%2C%0A%7D%0A%0Aconst+box%3A+React.CSSProperties+%3D+%7B%0Awidth%3A+100%2C%0Aheight%3A+100%2C%0AbackgroundColor%3A+%22var%28--hue-5%29%22%2C%0AborderRadius%3A+%2210px%22%2C%0A%7D%0A%0Aconst+button%3A+React.CSSProperties+%3D+%7B%0AbackgroundColor%3A+%22var%28--hue-5%29%22%2C%0AborderRadius%3A+%2210px%22%2C%0Apadding%3A+%2210px+20px%22%2C%0Acolor%3A+%22var%28--black%29%22%2C%0Aposition%3A+%22absolute%22%2C%0Abottom%3A+0%2C%0Aleft%3A+0%2C%0Aright%3A+0%2C%0A%7D%0A%0A%60%60%60) [Tutorial](https://motion.dev/tutorials/react-exit-animation)

Hide

Beginner's guide to AnimatePresence in Motion - YouTube

Tap to unmute

## [Usage](https://motion.dev/docs/react-animate-presence\#usage)

### [Import](https://motion.dev/docs/react-animate-presence\#import)

```
import { AnimatePresence } from "motion/react"
```

### [Exit animations](https://motion.dev/docs/react-animate-presence\#exit-animations)

`AnimatePresence` works by detecting when its **direct children** are removed from the React tree.

This can be due to a component mounting/remounting:

```
<AnimatePresence>
  {show && <Modal key="modal" />}
</AnimatePresence>
```

Its `key` changing:

```
<AnimatePresence>
  <Slide key={activeItem.id} />
</AnimatePresence>
```

Or when children in a list are added/removed:

```
<AnimatePresence>
  {items.map(item => (
    <motion.li key={item.id} exit={{ opacity: 1 }} layout />
  ))}
</AnimatePresence>
```

Any `motion` components within the exiting component will fire animations defined on their `exit` props before the component is removed from the DOM.

```
function Slide({ img, description }) {
  return (
    <motion.div exit={{ opacity: 0 }}>
      <img src={img.src} />
      <motion.p exit={{ y: 10 }}>{description}</motion.p>
    </motion.div>
  )
}
```

Like `initial` and `animate`, `exit` can be defined either as an object of values, or as a variant label.

```
const modalVariants = {
  visible: { opacity: 1, transition: { when: "beforeChildren" } },
  hidden: { opacity: 0, transition: { when: "afterChildren" } }
}

function Modal({ children }) {
  return (
    <motion.div initial="hidden" animate="visible" exit="hidden">
      {children}
    </motion.div>
  )
}
```

Direct children must each have a unique `key` prop so `AnimatePresence` can track their presence in the tree.

### [Changing `key`](https://motion.dev/docs/react-animate-presence\#changing-key)

Changing a `key` prop makes React create an entirely new component. So by changing the `key` of a single child of `AnimatePresence`, we can easily make components like slideshows.

```
export const Slideshow = ({ image }) => (
  <AnimatePresence>
    <motion.img
      key={image.src}
      src={image.src}
      initial={{ x: 300, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: -300, opacity: 0 }}
    />
  </AnimatePresence>
)
```

### [Access presence state](https://motion.dev/docs/react-animate-presence\#access-presence-state)

Any child of `AnimatePresence` can access presence state with the `useIsPresence` hook.

```
import { useIsPresent } from "motion/react"

function Component() {
  const isPresent = useIsPresent()

  return isPresent ? "Here!" : "Exiting..."
}
```

This allows you to change content or styles when a component is no longer rendered.

### [Access presence data](https://motion.dev/docs/react-animate-presence\#access-presence-data)

When a component has been removed from the React tree, its props can no longer be updated. We can use `AnimatePresence`'s `custom` prop to pass new data down through the tree, even into exiting components.

```
<AnimatePresence custom={swipeDirection}>
  <Slide key={activeSlideId}>
```

Then later we can extract that using `usePresenceData`.

```
import { AnimatePresence, usePresenceData } from "motion/react"

function Slide() {
  const isPresent = useIsPresent()
  const direction = usePresenceData()

  return (
    <motion.div exit={{ opacity: 0 }}>
      {isPresent ? "Here!" : "Exiting " + direction}
    </motion.div>
  )
}
```

>Live example [Open](https://examples.motion.dev/react/use-presence-data)

usePresenceData — Motion for React Example

View source Open in Cursor

### [Manual usage](https://motion.dev/docs/react-animate-presence\#manual-usage)

It's also possible to manually tell `AnimatePresence` when a component is safe to remove with the `usePresence` hook.

This returns both `isPresent` state and a callback, `safeToRemove`, that should be called when you're ready to remove the component from the DOM (for instance after a manual animation or other timeout).

```
import { usePresence } from "motion/react"

function Component() {
  const [isPresent, safeToRemove] = usePresence()

  useEffect(() => {
    // Remove from DOM 1000ms after being removed from React
    !isPresent && setTimeout(safeToRemove, 1000)
  }, [isPresent])

  return <div />
}
```

### [Propagate exit animations](https://motion.dev/docs/react-animate-presence\#propagate-exit-animations)

By default, `AnimatePresence` controls the `exit` animations on all of its children, **until** another `AnimatePresence` component is rendered.

```
<AnimatePresence>
  {show ? (
    <motion.section exit={{ opacity: 0 }}>
      <AnimatePresence>
        {/*
          * When `show` becomes `false`, exit animations
          * on these children will not fire.
          */}
        {children}
      </AnimatePresence>
    </motion.section>
  ) : null}
</AnimatePresence>
```

By setting an `AnimatePresence` component's `propagate` prop to `true`, when it's removed from another `AnimatePresence` it will fire all of **its** children's exit animations.

```
<AnimatePresence>
  {show ? (
    <motion.section exit={{ opacity: 0 }}>
      <AnimatePresence propagate>
        {/*
          * When `show` becomes `false`, exit animations
          * on these children **will** fire.
          */}
        {children}
      </AnimatePresence>
    </motion.section>
  ) : null}
</AnimatePresence>
```

1. S
2. A
3. B
4. C
5. D
6. F

>Performance audit

### Find & fix animation performance issues with your agent.

The MotionScore performance audit skill grades every animation in your codebase S to F and hands your agent the fixes, before you ship. Part of the Motion+ AI Kit.

[Get the AI Kit](https://motion.dev/docs/ai-kit)

>Motion+ Examples

### 370+ production-ready examples.

JavaScript, React, and Vue. Copy and paste straight into your project, adapt with AI, or pipe the whole set into your agent with the Examples MCP.

[See all examples](https://motion.dev/examples?plus=true&platform=all)

Part of [Motion+](https://motion.dev/plus). One-time fee, lifetime access.

## [Props](https://motion.dev/docs/react-animate-presence\#props)

### [`initial`](https://motion.dev/docs/react-animate-presence\#initial)

By passing `initial={false}`, `AnimatePresence` will disable any initial animations on children that are present when the component is first rendered.

```
<AnimatePresence initial={false}>
  <Slide key={activeItem.id} />
</AnimatePresence>
```

### [`custom`](https://motion.dev/docs/react-animate-presence\#custom)

When a component is removed, there's no longer a chance to update its props (because it's no longer in the React tree). Therefore we can't update its exit animation with the same render that removed the component.

By passing a value through `AnimatePresence`'s `custom` prop, we can use dynamic variants to change the `exit` animation.

```
const variants = {
  hidden: (direction) => ({
    opacity: 0,
    x: direction === 1 ? -300 : 300
  }),
  visible: { opacity: 1, x: 0 }
}

export const Slideshow = ({ image, direction }) => (
  <AnimatePresence custom={direction}>
    <motion.img
      key={image.src}
      src={image.src}
      variants={variants}
      initial="hidden"
      animate="visible"
      exit="hidden"
    />
  </AnimatePresence>
)
```

This data can be accessed by children via `usePresenceData`.

### [`mode`](https://motion.dev/docs/react-animate-presence\#mode)

**Default:**`"sync"`

Decides how `AnimatePresence` handles entering and exiting children.

>Live example [Open](https://examples.motion.dev/react/animate-presence-modes)

AnimatePresence modes — Motion for React Example

View source Open in Cursor [Tutorial](https://motion.dev/tutorials/react-animate-presence-modes)

#### [`sync`](https://motion.dev/docs/react-animate-presence\#sync)

In `"sync"` mode, elements animate in and out as soon as they're added/removed.

This is the most basic (and default) mode - `AnimatePresence` takes no opinion on sequencing animations or layout. Therefore, if element layouts conflict (as in the above example), you can either implement your own solution (using `position: absolute` or similar), or try one of the other two `mode` options.

#### [`wait`](https://motion.dev/docs/react-animate-presence\#wait)

In `"wait"` mode, the entering element will **wait** until the exiting child has animated out, before it animates in.

This is great for sequential animations, presenting users with one piece of information or one UI element at a time.

`wait` mode only supports one child at a time.

Try setting `ease: "easeIn"` (or similar) on the exit animation, and `ease: "easeOut"` on the enter animation for an overall `easeInOut` easing effect.

#### [`popLayout`](https://motion.dev/docs/react-animate-presence\#poplayout)

Exiting elements will be "popped" out of the page layout, allowing surrounding elements to immediately reflow. Pairs especially well with the `layout` prop, so elements can animate to their new layout.

```
<AnimatePresence>
  {items.map(item => (
    <motion.li layout exit={{ opacity: 0 }} />
  )}
</AnimatePresence>
```

For a more detailed comparison, check out the [full AnimatePresence modes tutorial](https://motion.dev/tutorials/react-animate-presence-modes).

When using `popLayout` mode, any immediate child of AnimatePresence that's a custom component must be wrapped in React's `forwardRef` function, forwarding the provided `ref` to the DOM node you wish to pop out of the layout.

### [`onExitComplete`](https://motion.dev/docs/react-animate-presence\#onexitcomplete)

Fires when all exiting nodes have completed animating out.

### [`propagate`](https://motion.dev/docs/react-animate-presence\#propagate)

**Default:**`false`

If set to `true`, exit animations on children will also trigger when this `AnimatePresence` exits from a parent `AnimatePresence`.

```
<AnimatePresence>
  {show ? (
    <motion.section exit={{ opacity: 0 }}>
      <AnimatePresence propagate>
        {/* This exit prop will now fire when show is false */}
        <motion.div exit={{ x: -100 }} />
      </AnimatePresence>
    </motion.section>
  ) : null}
</AnimatePresence>
```

### [`root`](https://motion.dev/docs/react-animate-presence\#root)

Root element for injecting `popLayout` styles. Defaults to `document.head` but can be set to another `ShadowRoot`, for use within shadow DOM.

## [Troubleshooting](https://motion.dev/docs/react-animate-presence\#troubleshooting)

### [Exit animations aren't working](https://motion.dev/docs/react-animate-presence\#exit-animations-arent-working)

Ensure all **immediate** children get a unique `key` prop that **remains the same for that component every render**.

For instance, providing `index` as a `key` is **bad** because if the items reorder then the `index` will not be matched to the `item`:

```
<AnimatePresence>
  {items.map((item, index) => (
    <Component key={index} />
  ))}
</AnimatePresence>
```

It's preferred to pass something that's unique to that item, for instance an ID:

```
<AnimatePresence>
  {items.map((item) => (
    <Component key={item.id} />
  ))}
</AnimatePresence>
```

Also make sure `AnimatePresence` is **outside** of the code that unmounts the element. If `AnimatePresence` itself unmounts, then it can't control exit animations!

For example, this will **not work**:

```
isVisible && (
  <AnimatePresence>
    <Component />
  </AnimatePresence>
)
```

Instead, the conditional should be at the root of `AnimatePresence`:

```
<AnimatePresence>
  {isVisible && <Component />}
</AnimatePresence>
```

### [Layout animations not working with `mode="sync"`](https://motion.dev/docs/react-animate-presence\#layout-animations-not-working-with-modesync)

When mixing exit and [layout animations](https://motion.dev/docs/react-layout-animations), it might be necessary to wrap the group in `LayoutGroup` to ensure that components outside of `AnimatePresence` know when to perform a layout animation.

```
<LayoutGroup>
  <motion.ul layout>
    <AnimatePresence>
      {items.map(item => (
        <motion.li layout key={item.id} />
      ))}
    </AnimatePresence>
  </motion.ul>
</LayoutGroup>
```

### [Layout animations not working with `mode="popLayout"`](https://motion.dev/docs/react-animate-presence\#layout-animations-not-working-with-modepoplayout)

When any HTML element has an active `transform` it temporarily becomes the [offset parent](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/offsetParent) of its children. This can cause children with `position: "absolute"` not to appear where you expect.

`mode="popLayout"` works by using `position: "absolute"`. So to ensure consistent and expected positioning during a layout animation, ensure that the animating parent has a `position` other than `"static"`.

```
<motion.ul layout style={{ position: "relative" }}>
  <AnimatePresence mode="popLayout">
    {items.map(item => (
      <motion.li layout key={item.id} />
    ))}
  </AnimatePresence>
</motion.ul>
```

>Newsletter

### Stay in the loop.

Deep dives on animation, performance, and building Motion. New issues land roughly once a month, no filler.

Subscribe

## Related topics

- [Motion component→\\
\\
Animate elements with a declarative API. Supports variants, gestures, and layout animations.](https://motion.dev/docs/react-motion-component)
- [Cursor→\\
\\
Custom cursor and follow-along effects for React.](https://motion.dev/docs/cursor)
- [AnimateActivity→\\
\\
Add enter, exit, and layout animations to components inside React's Activity boundaries.](https://motion.dev/docs/react-animate-activity)
- [Exit animation→\\
\\
An example of animating an element when it's removed from the DOM using AnimatePresence in Motion for React.](https://motion.dev/tutorials/react-exit-animation)

[Motion+370+ examplesLifetime updates\\
\\
Motion+ **Level up your animations.** \\
\\
Unlock 370+ premium examples, premium APIs, private Discord and GitHub, and a transition editor for your IDE. One-time purchase, lifetime updates.\\
\\
PricingOne-time payment, lifetime updates\\
\\
Get Motion+](https://motion.dev/plus)

[PreviousAnimateActivity](https://motion.dev/docs/react-animate-activity) [NextAnimateView](https://motion.dev/docs/react-animate-view)

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