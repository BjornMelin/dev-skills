[Motion](https://motion.dev/)

[Docs](https://motion.dev/docs) [Examples](https://motion.dev/examples) [Tutorials](https://motion.dev/tutorials) [AI Kit](https://motion.dev/docs/ai-kit) [Motion+](https://motion.dev/plus)

[Docs](https://motion.dev/docs)/ [React](https://motion.dev/docs/react)

# Layout animation

Smoothly animate layout changes and shared element transitions.

Copy pageCopy page

DocsLayout animationReactJavaScriptVueAIAI Kit

[Get started](https://motion.dev/docs/react)

Animations

[Overview](https://motion.dev/docs/react-animation)

[Layout](https://motion.dev/docs/react-layout-animations)

[›How to animate layout changes](https://motion.dev/docs/react-layout-animations#how-to-animate-layout-changes) [›Advanced use-cases](https://motion.dev/docs/react-layout-animations#advanced-use-cases) [›Troubleshooting](https://motion.dev/docs/react-layout-animations#troubleshooting) [›Technical reading](https://motion.dev/docs/react-layout-animations#technical-reading) [›Motion's layout animations vs the View Transitions API](https://motion.dev/docs/react-layout-animations#motions-layout-animations-vs-the-view-transitions-api) [›FAQs](https://motion.dev/docs/react-layout-animations#faqs) [›Related topics](https://motion.dev/docs/react-layout-animations#docs-related-title)

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

Motion (previously Framer Motion) can automatically animate an element's size and position whenever a layout change occurs - with a single prop. Add `layout` to animate a single component, or use `layoutId` to animate shared elements across components, creating seamless transitions between different UI states.

Layout animations with Motion - YouTube

Tap to unmute

[Layout animations with Motion](https://www.youtube.com/watch?v=zhl3k6s-Cf4) [Motion](https://www.youtube.com/channel/UC-hTdPnhQyNjeuCIFfkGPXg)

![thumbnail-image](https://yt3.ggpht.com/yGCgyTHXxI7V9C7QDZX9M_glj4GeUyh9HYuwWxCG50CKSZTnf3cD8AUg_WT5tw6HlEFAjmiyBZ4=s68-c-k-c0x00ffffff-no-rj)

Motion351 subscribers

In this guide, we'll learn how to:

- **Animate layout changes** with a single prop.

- Create **shared element transitions** between components.

- Explore **advanced techniques**.

- **Troubleshoot** common layout animation issues.

- Understand the **differences** between Motion and the native View Transitions API.


Prefer to learn by doing? Check out our collection of official [React layout animation examples](https://motion.dev/examples?category=layout-animations&platform=react).

## [How to animate layout changes](https://motion.dev/docs/react-layout-animations\#how-to-animate-layout-changes)

To enable layout animations on a `motion` component, simply add the `layout` prop. Any layout change that happens as a result of a React render will now be automatically animated.

```
<motion.div layout />
```

Layout animation can animate previously unanimatable CSS values, like switching `justify-content` between `flex-start` and `flex-end`.

```
<motion.div
  layout
  style={{ justifyContent: isOn ? "flex-start" : "flex-end" }}
/>
```

>Live example [Open](https://examples.motion.dev/react/layout-animation)

Layout animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+*+as+motion+from+%22motion%2Freact-client%22%0Aimport+%7B+useState+%7D+from+%22react%22%0A%0Aexport+default+function+LayoutAnimation%28%29+%7B%0Aconst+%5BisOn%2C+setIsOn%5D+%3D+useState%28false%29%0A%0Aconst+toggleSwitch+%3D+%28%29+%3D%3E+setIsOn%28%21isOn%29%0A%0Areturn+%28%0A%3Cbutton%0AclassName%3D%22toggle-container%22%0Astyle%3D%7B%7B%0A...container%2C%0AjustifyContent%3A+%22flex-%22+%2B+%28isOn+%3F+%22start%22+%3A+%22end%22%29%2C%0A%7D%7D%0AonClick%3D%7BtoggleSwitch%7D%0A%3E%0A%3Cmotion.div%0AclassName%3D%22toggle-handle%22%0Astyle%3D%7Bhandle%7D%0Alayout%0Atransition%3D%7B%7B%0Atype%3A+%22spring%22%2C%0AvisualDuration%3A+0.2%2C%0Abounce%3A+0.2%2C%0A%7D%7D%0A%2F%3E%0A%3C%2Fbutton%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+container+%3D+%7B%0Awidth%3A+100%2C%0Aheight%3A+50%2C%0AbackgroundColor%3A+%22var%28--hue-3-transparent%29%22%2C%0AborderRadius%3A+50%2C%0Acursor%3A+%22pointer%22%2C%0Adisplay%3A+%22flex%22%2C%0Apadding%3A+10%2C%0A%7D%0A%0Aconst+handle+%3D+%7B%0Awidth%3A+50%2C%0Aheight%3A+50%2C%0AbackgroundColor%3A+%22var%28--hue-3%29%22%2C%0AborderRadius%3A+%2250%25%22%2C%0A%7D%0A%0A%60%60%60) [Tutorial](https://motion.dev/tutorials/react-layout-animation)

Or by using the `layoutId` prop, it's possible to match two elements and animate between them for some truly advanced animations.

```
<motion.li layoutId="item" />
```

It can handle anything from microinteractions to full page transitions.

>Live example [Open](https://examples.motion.dev/react/app-store)

iOS App Store — Motion for React Example

View source Open in Cursor [Tutorial](https://motion.dev/tutorials/react-app-store)

## Today

![Photo of Matt Perry](https://examples.motion.dev/authors/matt-perry.png)

- ![](https://examples.motion.dev/photos/app-store/a.jpg)



Travel

## 5 Inspiring Apps for Your Next Trip

- ![](https://examples.motion.dev/photos/app-store/c.jpg)



How to

## Contemplate the Meaning of Life Twice a Day

- ![](https://examples.motion.dev/photos/app-store/d.jpg)



Steps

## Urban Exploration Apps for the Vertically-Inclined

- ![](https://examples.motion.dev/photos/app-store/b.jpg)



Hats

## Take Control of Your Hat Life With This Stunning New App


When performing layout animations, changes to layout should be made via `style` or `className`, not via animation props like `animate` or `whileHover`, as `layout` will take care of the animation.

>Live example [Open](https://examples.motion.dev/react/reorder-items)

Reorder animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+Transition+%7D+from+%22motion%2Freact%22%0Aimport+*+as+motion+from+%22motion%2Freact-client%22%0Aimport+%7B+useEffect%2C+useState+%7D+from+%22react%22%0A%0Aexport+default+function+Reordering%28%29+%7B%0Aconst+%5Border%2C+setOrder%5D+%3D+useState%28initialOrder%29%0A%0AuseEffect%28%28%29+%3D%3E+%7B%0Aconst+timeout+%3D+setTimeout%28%28%29+%3D%3E+setOrder%28shuffle%28order%29%29%2C+1000%29%0Areturn+%28%29+%3D%3E+clearTimeout%28timeout%29%0A%7D%2C+%5Border%5D%29%0A%0Areturn+%28%0A%3Cul+style%3D%7Bcontainer%7D%3E%0A%7Border.map%28%28backgroundColor%29+%3D%3E+%28%0A%3Cmotion.li%0Akey%3D%7BbackgroundColor%7D%0Alayout%0Atransition%3D%7Bspring%7D%0Astyle%3D%7B%7B+...item%2C+backgroundColor+%7D%7D%0A%2F%3E%0A%29%29%7D%0A%3C%2Ful%3E%0A%29%0A%7D%0A%0Aconst+initialOrder+%3D+%5B%0A%22var%28--hue-1%29%22%2C%0A%22var%28--hue-2%29%22%2C%0A%22var%28--hue-3%29%22%2C%0A%22var%28--hue-4%29%22%2C%0A%5D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Utils+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0Afunction+shuffle%28%5B...array%5D%3A+string%5B%5D%29+%7B%0Areturn+array.sort%28%28%29+%3D%3E+Math.random%28%29+-+0.5%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+spring%3A+Transition+%3D+%7B%0Atype%3A+%22spring%22%2C%0Adamping%3A+20%2C%0Astiffness%3A+300%2C%0A%7D%0A%0Aconst+container%3A+React.CSSProperties+%3D+%7B%0AlistStyle%3A+%22none%22%2C%0Apadding%3A+0%2C%0Amargin%3A+0%2C%0Aposition%3A+%22relative%22%2C%0Adisplay%3A+%22flex%22%2C%0AflexWrap%3A+%22wrap%22%2C%0Agap%3A+10%2C%0Awidth%3A+300%2C%0AflexDirection%3A+%22row%22%2C%0AjustifyContent%3A+%22center%22%2C%0AalignItems%3A+%22center%22%2C%0A%7D%0A%0Aconst+item%3A+React.CSSProperties+%3D+%7B%0Awidth%3A+100%2C%0Aheight%3A+100%2C%0AborderRadius%3A+%2210px%22%2C%0A%7D%0A%0A%60%60%60)

Layout changes can be anything, changing `width`/`height`, number of grid columns, reordering a list, or adding/removing new items:

### [Performance](https://motion.dev/docs/react-layout-animations\#performance)

Animating layout is traditionally slow, but Motion performs all layout animations using the CSS `transform` property for the highest possible performance.

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

### [Shared layout animations](https://motion.dev/docs/react-layout-animations\#shared-layout-animations)

For more advanced shared layout animations, `layoutId` allows you to connect two different elements.

When a new component is added with a `layoutId` prop matching an existing component, it will automatically animate out from the old component.

```
isSelected && <motion.div layoutId="underline" />
```

>Live example [Open](https://examples.motion.dev/react/shared-layout-animation)

Shared layout animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+AnimatePresence+%7D+from+%22motion%2Freact%22%0Aimport+*+as+motion+from+%22motion%2Freact-client%22%0Aimport+%7B+useState+%7D+from+%22react%22%0A%0Aexport+default+function+SharedLayoutAnimation%28%29+%7B%0Aconst+%5BselectedTab%2C+setSelectedTab%5D+%3D+useState%28tabs%5B0%5D%29%0A%0Areturn+%28%0A%3Cdiv+style%3D%7Bcontainer%7D%3E%0A%3Cnav+style%3D%7Bnav%7D%3E%0A%3Cul+style%3D%7BtabsContainer%7D%3E%0A%7Btabs.map%28%28item%29+%3D%3E+%28%0A%3Cmotion.li%0Akey%3D%7Bitem.label%7D%0Ainitial%3D%7Bfalse%7D%0Aanimate%3D%7B%7B%0AbackgroundColor%3A%0Aitem+%3D%3D%3D+selectedTab+%3F+%22%23eee%22+%3A+%22%23eee0%22%2C%0A%7D%7D%0Astyle%3D%7Btab%7D%0AonClick%3D%7B%28%29+%3D%3E+setSelectedTab%28item%29%7D%0A%3E%0A%7B%60%24%7Bitem.icon%7D+%24%7Bitem.label%7D%60%7D%0A%7Bitem+%3D%3D%3D+selectedTab+%3F+%28%0A%3Cmotion.div%0Astyle%3D%7Bunderline%7D%0AlayoutId%3D%22underline%22%0Aid%3D%22underline%22%0A%2F%3E%0A%29+%3A+null%7D%0A%3C%2Fmotion.li%3E%0A%29%29%7D%0A%3C%2Ful%3E%0A%3C%2Fnav%3E%0A%3Cmain+style%3D%7BiconContainer%7D%3E%0A%3CAnimatePresence+mode%3D%22wait%22%3E%0A%3Cmotion.div%0Akey%3D%7BselectedTab+%3F+selectedTab.label+%3A+%22empty%22%7D%0Ainitial%3D%7B%7B+y%3A+10%2C+opacity%3A+0+%7D%7D%0Aanimate%3D%7B%7B+y%3A+0%2C+opacity%3A+1+%7D%7D%0Aexit%3D%7B%7B+y%3A+-10%2C+opacity%3A+0+%7D%7D%0Atransition%3D%7B%7B+duration%3A+0.2+%7D%7D%0Astyle%3D%7Bicon%7D%0A%3E%0A%7BselectedTab+%3F+selectedTab.icon+%3A+%22%F0%9F%98%8B%22%7D%0A%3C%2Fmotion.div%3E%0A%3C%2FAnimatePresence%3E%0A%3C%2Fmain%3E%0A%3C%2Fdiv%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+container%3A+React.CSSProperties+%3D+%7B%0Awidth%3A+480%2C%0Aheight%3A+360%2C%0AmaxWidth%3A+%22calc%28100%25+-+40px%29%22%2C%0AmaxHeight%3A+%22calc%28100%25+-+40px%29%22%2C%0AborderRadius%3A+10%2C%0Abackground%3A+%22white%22%2C%0Aoverflow%3A+%22hidden%22%2C%0AboxShadow%3A%0A%220+1px+1px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+2px+2px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+4px+4px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+8px+8px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+16px+16px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+2px+2px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+4px+4px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+8px+8px+hsl%280deg+0%25+0%25+%2F+0.075%29%2C+0+16px+16px+hsl%280deg+0%25+0%25+%2F+0.075%29%22%2C%0Adisplay%3A+%22flex%22%2C%0AflexDirection%3A+%22column%22%2C%0A%7D%0A%0Aconst+nav%3A+React.CSSProperties+%3D+%7B%0Abackground%3A+%22%23fdfdfd%22%2C%0Apadding%3A+%225px+5px+0%22%2C%0AborderRadius%3A+%2210px%22%2C%0AborderBottomLeftRadius%3A+0%2C%0AborderBottomRightRadius%3A+0%2C%0AborderBottom%3A+%221px+solid+%23eeeeee%22%2C%0Aheight%3A+44%2C%0A%7D%0A%0Aconst+tabsStyles%3A+React.CSSProperties+%3D+%7B%0AlistStyle%3A+%22none%22%2C%0Apadding%3A+0%2C%0Amargin%3A+0%2C%0AfontWeight%3A+500%2C%0AfontSize%3A+14%2C%0A%7D%0A%0Aconst+tabsContainer%3A+React.CSSProperties+%3D+%7B%0A...tabsStyles%2C%0Adisplay%3A+%22flex%22%2C%0Awidth%3A+%22100%25%22%2C%0A%7D%0A%0Aconst+tab%3A+React.CSSProperties+%3D+%7B%0A...tabsStyles%2C%0AborderRadius%3A+5%2C%0AborderBottomLeftRadius%3A+0%2C%0AborderBottomRightRadius%3A+0%2C%0Awidth%3A+%22100%25%22%2C%0Apadding%3A+%2210px+15px%22%2C%0Aposition%3A+%22relative%22%2C%0Abackground%3A+%22white%22%2C%0Acursor%3A+%22pointer%22%2C%0Aheight%3A+24%2C%0Adisplay%3A+%22flex%22%2C%0AjustifyContent%3A+%22space-between%22%2C%0AalignItems%3A+%22center%22%2C%0Aflex%3A+1%2C%0AminWidth%3A+0%2C%0AuserSelect%3A+%22none%22%2C%0Acolor%3A+%22var%28--black%29%22%2C%0A%7D%0A%0Aconst+underline%3A+React.CSSProperties+%3D+%7B%0Aposition%3A+%22absolute%22%2C%0Abottom%3A+-2%2C%0Aleft%3A+0%2C%0Aright%3A+0%2C%0Aheight%3A+2%2C%0Abackground%3A+%22var%28--accent%29%22%2C%0A%7D%0A%0Aconst+iconContainer%3A+React.CSSProperties+%3D+%7B%0Adisplay%3A+%22flex%22%2C%0AjustifyContent%3A+%22center%22%2C%0AalignItems%3A+%22center%22%2C%0Aflex%3A+1%2C%0A%7D%0A%0Aconst+icon%3A+React.CSSProperties+%3D+%7B%0AfontSize%3A+128%2C%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Data+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+allIngredients+%3D+%5B%0A%7B+icon%3A+%22%F0%9F%8D%85%22%2C+label%3A+%22Tomato%22+%7D%2C%0A%7B+icon%3A+%22%F0%9F%A5%AC%22%2C+label%3A+%22Lettuce%22+%7D%2C%0A%7B+icon%3A+%22%F0%9F%A7%80%22%2C+label%3A+%22Cheese%22+%7D%2C%0A%7B+icon%3A+%22%F0%9F%A5%95%22%2C+label%3A+%22Carrot%22+%7D%2C%0A%7B+icon%3A+%22%F0%9F%8D%8C%22%2C+label%3A+%22Banana%22+%7D%2C%0A%7B+icon%3A+%22%F0%9F%AB%90%22%2C+label%3A+%22Blueberries%22+%7D%2C%0A%7B+icon%3A+%22%F0%9F%A5%82%22%2C+label%3A+%22Champers%3F%22+%7D%2C%0A%5D%0A%0Aconst+%5Btomato%2C+lettuce%2C+cheese%5D+%3D+allIngredients%0Aconst+tabs+%3D+%5Btomato%2C+lettuce%2C+cheese%5D%0A%0A%60%60%60)

- 🍅 Tomato

- 🥬 Lettuce
- 🧀 Cheese

🍅

If the original component is still on the page when the new one enters, they will automatically crossfade.

To animate an element back to its origin, you can use the `AnimatePresence` component to keep it in the DOM until its exit animation has finished.

```
<AnimatePresence>
  {isOpen && <motion.div layoutId="modal" />}
</AnimatePresence>
```

### [Customise a layout animation](https://motion.dev/docs/react-layout-animations\#customise-a-layout-animation)

Layout animations can be customised using [the](https://motion.dev/docs/react-transitions)`transition` [prop](https://motion.dev/docs/react-transitions).

```
<motion.div layout transition={{ duration: 0.3 }} />
```

If you need to set a transition specifically for the layout animation while having a different transition for other properties (like `opacity`), you can define a dedicated `layout` transition.

```
<motion.div
  layout
  animate={{ opacity: 0.5 }}
  transition={{
    ease: "linear",
    layout: { duration: 0.3 }
  }}
/>
```

When performing a shared layout animation, the transition defined for element we're animating **to** will be used.

```
<>
  <motion.button
    layoutId="modal"
    onClick={() => setIsOpen(true)}
    // This transition will be used when the modal closes
    transition={{ type: "spring" }}
  >
    Open
  </motion.button>
  <AnimatePresence>
    {isOn && (
      <motion.dialog
        layoutId="modal"
        // This transition will be used when the modal opens
        transition={{ duration: 0.3 }}
      />
    )}
  </AnimatePresence>
</>
```

[card.css/motion-app\\
\\
card.cssCard.tsx\\
\\
```\\
1.card {2  transition: scale 200ms linear(3    0, 0.009, 0.036, 0.084, 0.157, 0.255, 0.378,4    0.522, 0.679, 0.832, 0.954, 1.029, 1.052, 1.038,5    1.011, 0.99, 0.984, 0.991, 1.001, 1.005, 16  );7}89.card:hover {10  scale: 1.2;11}\\
```\\
\\
MOTION\\
\\
EaseSpring\\
\\
Duration0.3\\
\\
Delay0\\
\\
›Saved transitions12\\
\\
**Visual editing for IDEs.** \\
\\
Edit and preview Motion and CSS transitions live in your code. Tune ease curves, springs, and durations without leaving your editor.\\
\\
Part of Motion+. One-time fee, lifetime access.](https://motion.dev/plus)

## [Advanced use-cases](https://motion.dev/docs/react-layout-animations\#advanced-use-cases)

### [Layout animations inside scrollable containers](https://motion.dev/docs/react-layout-animations\#layout-animations-inside-scrollable-containers)

To correctly animate layout within a scrollable container, you must add the `layoutScroll` prop to the scrollable element. This allows Motion to account for the element's scroll offset.

```
<motion.div layoutScroll style={{ overflow: "scroll" }} />
```

### [Animating within fixed containers](https://motion.dev/docs/react-layout-animations\#animating-within-fixed-containers)

To correctly animate layout within fixed elements, we need to provide them the `layoutRoot` prop.

```
<motion.div layoutRoot style={{ position: "fixed" }} />
```

This lets Motion account for the page's scroll offset when measuring children.

### [Group layout animations](https://motion.dev/docs/react-layout-animations\#group-layout-animations)

Layout animations are triggered when a component re-renders and its layout has changed.

```
function Accordion() {
  const [isOpen, setOpen] = useState(false)

  return (
    <motion.div
      layout
      style={{ height: isOpen ? "100px" : "500px" }}
      onClick={() => setOpen(!isOpen)}
    />
  )
}
```

But what happens when we have two or more components that don't re-render at the same time, but **do** affect each other's layout?

```
function List() {
  return (
    <>
      <Accordion />
      <Accordion />
    </>
  )
}
```

When one re-renders, for performance reasons the other won't be able to detect changes to its layout.

We can synchronise layout changes across multiple components by wrapping them in the `LayoutGroup component`.

```
import { LayoutGroup } from "motion/react"

function List() {
  return (
    <LayoutGroup>
      <Accordion />
      <Accordion />
    </LayoutGroup>
  )
}
```

When layout changes are detected in any grouped `motion` component, layout animations will trigger across all of them.

### [Relative animation](https://motion.dev/docs/react-layout-animations\#relative-animation)

Motion's layout animations use **parent-relative** calculations instead of **viewport or page-relative**.

What this means is if you have a parent and child performing a layout animation with different transitions, unlike the browser's View Transition API, the child will never get "left behind" by its parent.

By default, these calculations use the top left of the child, but you can change this with the `layoutAnchor` prop. This accepts `0`-`1` progress values for `x` and `y` where `0` is top/left and 1 is bottom/right.

```
// Pin element to center
<motion.ul layout>
  <motion.li
    layout
    layoutAnchor={{ x: 0.5, y: 0.5 }}
    transition={{ delay: 1 }}
  />
</motion.ul>
```

>Live example [Open](https://examples.motion.dev/react/layout-anchor)

Layout Anchor — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+useState+%7D+from+%22react%22%0Aimport+%7B+motion+%7D+from+%22motion%2Freact%22%0A%0Afunction+LayoutAnchor%28%7B%0AanchorX+%3D+0.5%2C%0AanchorY+%3D+0.5%2C%0A%7D%29+%7B%0Aconst+%5Bexpanded%2C+setExpanded%5D+%3D+useState%28false%29%0A%0Areturn+%28%0A%3Cdiv+style%3D%7Bcontainer%7D%3E%0A%3Cmotion.div%0Alayout%0AlayoutDependency%3D%7Bexpanded%7D%0Astyle%3D%7B%7B%0A...parent%2C%0Awidth%3A+expanded+%3F+300+%3A+150%2C%0Aheight%3A+expanded+%3F+300+%3A+150%2C%0A%7D%7D%0Atransition%3D%7B%7B+duration%3A+0.8%2C+ease%3A+%22easeInOut%22+%7D%7D%0AonClick%3D%7B%28%29+%3D%3E+setExpanded%28%21expanded%29%7D%0A%3E%0A%3Cmotion.div%0Alayout%0AlayoutDependency%3D%7Bexpanded%7D%0AlayoutAnchor%3D%7B%7B+x%3A+anchorX%2C+y%3A+anchorY+%7D%7D%0Astyle%3D%7B%7B%0A...child%2C%0Awidth%3A+expanded+%3F+100+%3A+70%2C%0Aheight%3A+expanded+%3F+100+%3A+70%2C%0A%7D%7D%0Atransition%3D%7B%7B%0Aduration%3A+0.8%2C%0Aease%3A+%22easeInOut%22%2C%0Adelay%3A+0.8%2C%0A%7D%7D%0A%3E%0A%3Cmotion.div%0Alayout%0AlayoutDependency%3D%7Bexpanded%7D%0Astyle%3D%7B%7B%0A...crosshair%2C%0Aleft%3A+%60%24%7BanchorX+*+100%7D%25%60%2C%0Atop%3A+%60%24%7BanchorY+*+100%7D%25%60%2C%0A%7D%7D%0Atransition%3D%7B%7B%0Aduration%3A+0.8%2C%0Aease%3A+%22easeInOut%22%2C%0Adelay%3A+0.8%2C%0A%7D%7D%0A%3E%0A%3Cdiv+style%3D%7BcrosshairH%7D+%2F%3E%0A%3Cdiv+style%3D%7BcrosshairV%7D+%2F%3E%0A%3C%2Fmotion.div%3E%0A%3C%2Fmotion.div%3E%0A%3C%2Fmotion.div%3E%0A%3C%2Fdiv%3E%0A%29%0A%7D%0A%0Aexport+default+LayoutAnchor%0A%0A%2F**+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+*%2F%0A%0Aconst+container%3A+React.CSSProperties+%3D+%7B%0Adisplay%3A+%22flex%22%2C%0AjustifyContent%3A+%22center%22%2C%0AalignItems%3A+%22center%22%2C%0Awidth%3A+%22100%25%22%2C%0Aheight%3A+%22100%25%22%2C%0A%7D%0A%0Aconst+parent%3A+React.CSSProperties+%3D+%7B%0Adisplay%3A+%22flex%22%2C%0AalignItems%3A+%22center%22%2C%0AjustifyContent%3A+%22center%22%2C%0AbackgroundColor%3A+%22var%28--layer%29%22%2C%0Aborder%3A+%221px+solid+var%28--border%29%22%2C%0AborderRadius%3A+20%2C%0Acursor%3A+%22pointer%22%2C%0A%7D%0A%0Aconst+child%3A+React.CSSProperties+%3D+%7B%0Aposition%3A+%22relative%22%2C%0Awidth%3A+70%2C%0Aheight%3A+70%2C%0AborderRadius%3A+14%2C%0AbackgroundColor%3A+%22hsl%28240%2C+50%25%2C+55%25%29%22%2C%0Aoverflow%3A+%22visible%22%2C%0A%7D%0A%0Aconst+crosshairSize+%3D+20%0A%0Aconst+crosshair%3A+React.CSSProperties+%3D+%7B%0Aposition%3A+%22absolute%22%2C%0Awidth%3A+crosshairSize%2C%0Aheight%3A+crosshairSize%2C%0AmarginLeft%3A+-crosshairSize+%2F+2%2C%0AmarginTop%3A+-crosshairSize+%2F+2%2C%0ApointerEvents%3A+%22none%22%2C%0A%7D%0A%0Aconst+crosshairH%3A+React.CSSProperties+%3D+%7B%0Aposition%3A+%22absolute%22%2C%0Awidth%3A+%22100%25%22%2C%0Aheight%3A+0%2C%0Aleft%3A+0%2C%0Atop%3A+%2250%25%22%2C%0AborderTop%3A+%221px+dashed+rgba%28255%2C+255%2C+255%2C+0.7%29%22%2C%0A%7D%0A%0Aconst+crosshairV%3A+React.CSSProperties+%3D+%7B%0Aposition%3A+%22absolute%22%2C%0Awidth%3A+0%2C%0Aheight%3A+%22100%25%22%2C%0Aleft%3A+%2250%25%22%2C%0Atop%3A+0%2C%0AborderLeft%3A+%221px+dashed+rgba%28255%2C+255%2C+255%2C+0.7%29%22%2C%0A%7D%0A%0A%60%60%60)

Tweak–

anchorX0.50anchorY0.50

### [Fixing child distortion during layout animations](https://motion.dev/docs/react-layout-animations\#fixing-child-distortion-during-layout-animations)

Because `layout` animations use `transform: scale()`, they can sometimes visually distort children or certain CSS properties.

- **Child elements:** To fix distortion on direct children, these can also be given the `layout` prop.

- **Border radius and box shadow:** Motion automatically corrects distortion on these properties, but they must be set via the `style`, `animate` or other animation prop.


```
<motion.div layout style={{ borderRadius: 20 }} />
```

## [Troubleshooting](https://motion.dev/docs/react-layout-animations\#troubleshooting)

### [The component isn't animating](https://motion.dev/docs/react-layout-animations\#the-component-isnt-animating)

Ensure the component is **not** set to `display: inline`, as browsers don't apply `transform` to these elements.

Ensure the component is re-rendering when you expect the layout animation to start.

### [Animations don't work during window resize](https://motion.dev/docs/react-layout-animations\#animations-dont-work-during-window-resize)

Layout animations are blocked during horizontal window resize to improve performance and to prevent unnecessary animations.

### [SVG layout animations are broken](https://motion.dev/docs/react-layout-animations\#svg-layout-animations-are-broken)

SVG components aren't currently supported with layout animations. SVGs don't have layout systems so it's recommended to directly animate their attributes like `cx` etc.

### [Content is animating when the scrollbar appears](https://motion.dev/docs/react-layout-animations\#content-is-animating-when-the-scrollbar-appears)

Layout changes can affect whether or not a scrollbar is visible. Scrollbars take up visible space, which means layouts are then subsequently affected by the scrollbar. Layout animations will apply to any layout change.

If you're finding that this is leading to unwanted layout animations, you can ensure the scrollbar space is reserved, even when no scrollbar is visible, with the `scrollbar-gutter` CSS rule.

```
body {
  overflow-y: auto;
  scrollbar-gutter: stable;
}
```

### [The content stretches undesirably](https://motion.dev/docs/react-layout-animations\#the-content-stretches-undesirably)

This is a natural side-effect of animating `width` and `height` with `scale`.

Often, this can be fixed by providing these elements a `layout` animation and they'll be scale-corrected.

```
<motion.section layout>
  <motion.img layout />
</motion.section>
```

Some elements, like images or text that are changing between different aspect ratios, might be better animated with `layout="position"`.

### [Border radius or box shadows are behaving strangely](https://motion.dev/docs/react-layout-animations\#border-radius-or-box-shadows-are-behaving-strangely)

Animating `scale` is performant but can distort some styles like `border-radius` and `box-shadow`.

Motion automatically corrects for scale distortion on these properties, but they must be set on the element via `style`.

```
<motion.div layout style={{ borderRadius: 20 }} />
```

### [Border looks stretched during animation](https://motion.dev/docs/react-layout-animations\#border-looks-stretched-during-animation)

Elements with a `border` may look stretched during the animation. This is for two reasons:

1. Because changing `border` triggers layout recalculations, it defeats the performance benefits of animating via `transform`. You might as well animate `width` and `height` classically.

2. `border` can't render smaller than `1px`, which limits the degree of scale correction that Motion can perform on this style.


A work around is to replace `border` with a parent element with padding that acts as a `border`.

```
<motion.div layout style={{ borderRadius: 10, padding: 5 }}>
  <motion.div layout style={{ borderRadius: 5 }} />
</motion.div>
```

## [Technical reading](https://motion.dev/docs/react-layout-animations\#technical-reading)

Interested in the technical details behind layout animations? Nanda does an incredible job of [explaining the challenges](https://www.nan.fyi/magic-motion) of animating layout with transforms using interactive examples. Matt, creator of Motion, did a [talk at Vercel conference](https://www.youtube.com/watch?v=5-JIu0u42Jc&ab_channel=Vercel) about the implementation details that is largely up to date.

>Newsletter

### Stay in the loop.

Deep dives on animation, performance, and building Motion. New issues land roughly once a month, no filler.

Subscribe

## [Motion's layout animations vs the View Transitions API](https://motion.dev/docs/react-layout-animations\#motions-layout-animations-vs-the-view-transitions-api)

More browsers are starting to support the [View Transitions API](https://developer.mozilla.org/en-US/docs/Web/API/View_Transitions_API), which is similar to Motion's layout animations.

### [Benefits of View Transitions API](https://motion.dev/docs/react-layout-animations\#benefits-of-view-transitions-api)

The main two benefits of View Transitions is that **it's included in browsers** and **features a unique rendering system**.

#### [Filesize](https://motion.dev/docs/react-layout-animations\#filesize)

Because the View Transitions API is already included in browsers, it's cheap to implement very simple crossfade animations.

However, the CSS complexity can scale quite quickly. Motion's layout animations are around 12kb but from there it's very cheap to change transitions, add springs, mark matching

#### [Rendering](https://motion.dev/docs/react-layout-animations\#rendering)

Whereas Motion animates the elements as they exist on the page, View Transitions API does something quite unique in that it takes an image snapshot of the previous page state, and crossfades it with a live view of the new page state.

For shared elements, it does the same thing, taking little image snapshots and then crossfading those with a live view of the element's new state.

This can be leveraged to create interesting effects like full-screen wipes that aren't really in the scope of layout animations. [Framer's Page Effects](https://www.framer.com/academy/lessons/page-effects) were built with the View Transitions API and it also extensively uses layout animations. The right tool for the right job.

### [Drawbacks to View Transitions API](https://motion.dev/docs/react-layout-animations\#drawbacks-to-view-transitions-api)

There are quite a few drawbacks to the API vs layout animations:

- **Not interruptible**: Interrupting an animation mid-way will snap the animation to the end before starting the next one. This feels very janky.

- **Blocks interaction**: The animating elements overlay the "real" page underneath and block pointer events. Makes things feel quite sticky.

- **Difficult to manage IDs**: Layout animations allow more than one element with a `layoutId` whereas View Transitions will break if the previous element isn't removed.

- **Less performant:** View Transitions take an actual screenshot and animate via `width`/`height` vs layout animation's `transform`. This is measurably less performant when animating many elements.

- **Doesn't account for scroll**: If the page scroll changes during a view transition, elements will incorrectly animate this delta.

- **No relative animations:** If a nested element has a `delay` it will get "left behind" when its parent animates away, whereas Motion handles this kind of relative animation.

- **One animation at a time**: View Transitions animate the whole screen, which means combining it with other animations is difficult and other view animations impossible.


All-in-all, each system offers something different and each might be a better fit for your needs. In the future it might be that Motion also offers an API based on View Transitions API.

## [FAQs](https://motion.dev/docs/react-layout-animations\#faqs)

What is a layout animation?

A layout animation automatically animates an element's size and position when the layout changes, like reordering a list, toggling an accordion, or switching grid columns. Instead of calculating start and end values yourself, add `layout` to a `<motion />` component and Motion handles it automatically using transforms.

How are layout animations performant if they animate size?

Motion measures the layout change, then animates using CSS `transform` (translate + scale) instead of actually animating width and height. Animating transforms can entirely avoid triggering paint.

Why does my content look stretched during a layout animation?

When Motion uses `scale` to animate a size change, child elements can get visually distorted. Fix this by adding `layout` to the children too and Motion will calculate counter-scales them so they appear undistorted. For elements that change aspect ratio (like images), use `layout="position"` to only animate the position and let the size snap.

What's the difference between Motion's layout animations and the View Transitions API?

Both animate elements between layout states, but they work differently. Motion animates the actual elements using transforms: it's interruptible, doesn't block pointer events, and handles multiple simultaneous animations. View Transitions takes a screenshot of the old state and crossfades to the new one. It's built into browsers but can't be interrupted, blocks interaction during the transition, and is less performant when animating many elements.

## Related topics

- [LayoutGroup→\\
\\
Coordinate React layout animations between Motion components.](https://motion.dev/docs/react-layout-group)
- [AnimateNumber→\\
\\
Number ticker and countdown animations for React.](https://motion.dev/docs/react-animate-number)
- [Motion component→\\
\\
Animate elements with a declarative API. Supports variants, gestures, and layout animations.](https://motion.dev/docs/react-motion-component)
- [iOS App Store→\\
\\
An example of animating cards inspired by the iOS App Store using Motion for React's layout animations.](https://motion.dev/tutorials/react-app-store)

[Motion+370+ examplesLifetime updates\\
\\
Motion+ **Level up your animations.** \\
\\
Unlock 370+ premium examples, premium APIs, private Discord and GitHub, and a transition editor for your IDE. One-time purchase, lifetime updates.\\
\\
PricingOne-time payment, lifetime updates\\
\\
Get Motion+](https://motion.dev/plus)

[PreviousReact animation](https://motion.dev/docs/react-animation) [NextReact scroll animation](https://motion.dev/docs/react-scroll-animations)

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