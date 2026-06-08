[Motion](https://motion.dev/)

[Docs](https://motion.dev/docs) [Examples](https://motion.dev/examples) [Tutorials](https://motion.dev/tutorials) [AI Kit](https://motion.dev/docs/ai-kit) [Motion+](https://motion.dev/plus)

[Docs](https://motion.dev/docs)/ [React](https://motion.dev/docs/react)

# useScroll

Track scroll progress as motion values, for parallax and progress bars.

Copy pageCopy page

DocsuseScrollReactJavaScriptVueAIAI Kit

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

[›Usage](https://motion.dev/docs/react-use-scroll#usage) [›Performance](https://motion.dev/docs/react-use-scroll#performance) [›Options](https://motion.dev/docs/react-use-scroll#options) [›Related topics](https://motion.dev/docs/react-use-scroll#docs-related-title)

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

`useScroll` is used to create scroll-linked animations, like progress indicators and parallax effects.

```
const { scrollYProgress } = useScroll()

return <motion.div style={{ scaleX: scrollYProgress }} />
```

`useScroll` is able to run some animations with the browser's `ScrollTimeline` [API](https://developer.mozilla.org/en-US/docs/Web/API/ScrollTimeline) for optimal hardware-accelerated performance, removing scroll measurements, improving scroll synchronisation and ensuring animations remain smooth even under heavy CPI usage.

## [Usage](https://motion.dev/docs/react-use-scroll\#usage)

Import `useScroll` from Motion:

```
import { useScroll } from "motion/react"
```

`useScroll` returns four [motion values](https://motion.dev/docs/react-motion-value):

- `scrollX`/`Y`: The absolute scroll position, in pixels.

- `scrollXProgress`/`YProgress`: The scroll position between the defined offsets, as a value between `0` and `1`.


### [Page scroll](https://motion.dev/docs/react-use-scroll\#page-scroll)

By default, useScroll tracks the page scroll.

```
const { scrollY } = useScroll()

useMotionValueEvent(scrollY, "change", (latest) => {
  console.log("Page scroll: ", latest)
})
```

For example, we could show a page scroll indicator by passing `scrollYProgress` straight to the `scaleX` style of a progress bar.

```
const { scrollYProgress } = useScroll()

return <motion.div style={{ scaleX: scrollYProgress }} />
```

>Live example [Open](https://examples.motion.dev/react/scroll-linked)

Scroll-linked animations — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+motion%2C+useScroll+%7D+from+%22motion%2Freact%22%0A%0Aexport+default+function+ScrollLinked%28%29+%7B%0Aconst+%7B+scrollYProgress+%7D+%3D+useScroll%28%29%0A%0Areturn+%28%0A%3C%3E%0A%3Cmotion.div%0Aid%3D%22scroll-indicator%22%0Astyle%3D%7B%7B%0AscaleX%3A+scrollYProgress%2C%0Aposition%3A+%22fixed%22%2C%0Atop%3A+0%2C%0Aleft%3A+0%2C%0Aright%3A+0%2C%0Aheight%3A+10%2C%0AoriginX%3A+0%2C%0AbackgroundColor%3A+%22var%28--hue-1%29%22%2C%0A%7D%7D%0A%2F%3E%0A%3CContent+%2F%3E%0A%3C%2F%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Utils+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Afunction+Content%28%29+%7B%0Areturn+%28%0A%3C%3E%0A%3Carticle%0Astyle%3D%7B%7B%0AmaxWidth%3A+500%2C%0Apadding%3A+%22150px+20px%22%2C%0Adisplay%3A+%22flex%22%2C%0AflexDirection%3A+%22column%22%2C%0Agap%3A+20%2C%0A%7D%7D%0A%3E%0A%3Cp%3E%0ALorem+ipsum+dolor+sit+amet%2C+consectetur+adipiscing+elit.%0AAliquam+ac+rhoncus+quam.%0A%3C%2Fp%3E%0A%3Cp%3E%0AFringilla+quam+urna.+Cras+turpis+elit%2C+euismod+eget+ligula%0Aquis%2C+imperdiet+sagittis+justo.+In+viverra+fermentum+ex+ac%0Avestibulum.+Aliquam+eleifend+nunc+a+luctus+porta.+Mauris%0Alaoreet+augue+ut+felis+blandit%2C+at+iaculis+odio+ultrices.%0ANulla+facilisi.+Vestibulum+cursus+ipsum+tellus%2C+eu+tincidunt%0Aneque+tincidunt+a.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AIn+eget+sodales+arcu%2C+consectetur+efficitur+metus.+Duis%0Aefficitur+tincidunt+odio%2C+sit+amet+laoreet+massa+fringilla%0Aeu.%0A%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3Cp%3E%0ALorem+ipsum+dolor+sit+amet%2C+consectetur+adipiscing+elit.%0AAliquam+ac+rhoncus+quam.%0A%3C%2Fp%3E%0A%3Cp%3E%0AFringilla+quam+urna.+Cras+turpis+elit%2C+euismod+eget+ligula%0Aquis%2C+imperdiet+sagittis+justo.+In+viverra+fermentum+ex+ac%0Avestibulum.+Aliquam+eleifend+nunc+a+luctus+porta.+Mauris%0Alaoreet+augue+ut+felis+blandit%2C+at+iaculis+odio+ultrices.%0ANulla+facilisi.+Vestibulum+cursus+ipsum+tellus%2C+eu+tincidunt%0Aneque+tincidunt+a.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AIn+eget+sodales+arcu%2C+consectetur+efficitur+metus.+Duis%0Aefficitur+tincidunt+odio%2C+sit+amet+laoreet+massa+fringilla%0Aeu.%0A%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3C%2Farticle%3E%0A%3C%2F%3E%0A%29%0A%7D%0A%0A%60%60%60)

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam ac rhoncus quam.

Fringilla quam urna. Cras turpis elit, euismod eget ligula quis, imperdiet sagittis justo. In viverra fermentum ex ac vestibulum. Aliquam eleifend nunc a luctus porta. Mauris laoreet augue ut felis blandit, at iaculis odio ultrices. Nulla facilisi. Vestibulum cursus ipsum tellus, eu tincidunt neque tincidunt a.

## Sub-header

In eget sodales arcu, consectetur efficitur metus. Duis efficitur tincidunt odio, sit amet laoreet massa fringilla eu.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam ac rhoncus quam.

Fringilla quam urna. Cras turpis elit, euismod eget ligula quis, imperdiet sagittis justo. In viverra fermentum ex ac vestibulum. Aliquam eleifend nunc a luctus porta. Mauris laoreet augue ut felis blandit, at iaculis odio ultrices. Nulla facilisi. Vestibulum cursus ipsum tellus, eu tincidunt neque tincidunt a.

## Sub-header

In eget sodales arcu, consectetur efficitur metus. Duis efficitur tincidunt odio, sit amet laoreet massa fringilla eu.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

As `useScroll` returns motion values, we can compose this scroll info with other motion value hooks like `useTransform` and `useSpring`:

```
const { scrollYProgress } = useScroll()
const scaleX = useSpring(scrollYProgress)

return <motion.div style={{ scaleX }} />
```

>Live example [Open](https://examples.motion.dev/react/scroll-linked-with-spring)

Scroll-linked spring animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+motion%2C+useSpring%2C+useScroll+%7D+from+%22motion%2Freact%22%0A%0Aexport+default+function+ScrollLinked%28%29+%7B%0Aconst+%7B+scrollYProgress+%7D+%3D+useScroll%28%29%0Aconst+scaleX+%3D+useSpring%28scrollYProgress%2C+%7B%0Astiffness%3A+100%2C%0Adamping%3A+30%2C%0ArestDelta%3A+0.001%2C%0A%7D%29%0A%0Areturn+%28%0A%3C%3E%0A%3Cmotion.div%0Aid%3D%22scroll-indicator%22%0Astyle%3D%7B%7B%0AscaleX%2C%0Aposition%3A+%22fixed%22%2C%0Atop%3A+0%2C%0Aleft%3A+0%2C%0Aright%3A+0%2C%0Aheight%3A+10%2C%0AoriginX%3A+0%2C%0AbackgroundColor%3A+%22var%28--hue-1%29%22%2C%0A%7D%7D%0A%2F%3E%0A%3CContent+%2F%3E%0A%3C%2F%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Utils+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Afunction+Content%28%29+%7B%0Areturn+%28%0A%3C%3E%0A%3Carticle+style%3D%7B%7B+maxWidth%3A+500%2C+padding%3A+%22150px+20px%22+%7D%7D%3E%0A%3Cp%3E%0ALorem+ipsum+dolor+sit+amet%2C+consectetur+adipiscing+elit.%0AAliquam+ac+rhoncus+quam.%0A%3C%2Fp%3E%0A%3Cp%3E%0AFringilla+quam+urna.+Cras+turpis+elit%2C+euismod+eget+ligula%0Aquis%2C+imperdiet+sagittis+justo.+In+viverra+fermentum+ex+ac%0Avestibulum.+Aliquam+eleifend+nunc+a+luctus+porta.+Mauris%0Alaoreet+augue+ut+felis+blandit%2C+at+iaculis+odio+ultrices.%0ANulla+facilisi.+Vestibulum+cursus+ipsum+tellus%2C+eu+tincidunt%0Aneque+tincidunt+a.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AIn+eget+sodales+arcu%2C+consectetur+efficitur+metus.+Duis%0Aefficitur+tincidunt+odio%2C+sit+amet+laoreet+massa+fringilla%0Aeu.%0A%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3Cp%3E%0ALorem+ipsum+dolor+sit+amet%2C+consectetur+adipiscing+elit.%0AAliquam+ac+rhoncus+quam.%0A%3C%2Fp%3E%0A%3Cp%3E%0AFringilla+quam+urna.+Cras+turpis+elit%2C+euismod+eget+ligula%0Aquis%2C+imperdiet+sagittis+justo.+In+viverra+fermentum+ex+ac%0Avestibulum.+Aliquam+eleifend+nunc+a+luctus+porta.+Mauris%0Alaoreet+augue+ut+felis+blandit%2C+at+iaculis+odio+ultrices.%0ANulla+facilisi.+Vestibulum+cursus+ipsum+tellus%2C+eu+tincidunt%0Aneque+tincidunt+a.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AIn+eget+sodales+arcu%2C+consectetur+efficitur+metus.+Duis%0Aefficitur+tincidunt+odio%2C+sit+amet+laoreet+massa+fringilla%0Aeu.%0A%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3Cp%3E%0APellentesque+id+lacus+pulvinar+elit+pulvinar+pretium+ac+non%0Aurna.+Mauris+id+mauris+vel+arcu+commodo+venenatis.+Aliquam%0Aeu+risus+arcu.+Proin+sit+amet+lacus+mollis%2C+semper+massa+ut%2C%0Arutrum+mi.%0A%3C%2Fp%3E%0A%3Cp%3E%0ASed+sem+nisi%2C+luctus+consequat+ligula+in%2C+congue+sodales%0Anisl.%0A%3C%2Fp%3E%0A%3Cp%3E%0AVestibulum+bibendum+at+erat+sit+amet+pulvinar.+Pellentesque%0Apharetra+leo+vitae+tristique+rutrum.+Donec+ut+volutpat+ante%2C%0Aut+suscipit+leo.%0A%3C%2Fp%3E%0A%3Ch2%3ESub-header%3C%2Fh2%3E%0A%3Cp%3E%0AMaecenas+quis+elementum+nulla%2C+in+lacinia+nisl.+Ut+rutrum%0Afringilla+aliquet.+Pellentesque+auctor+vehicula+malesuada.%0AAliquam+id+feugiat+sem%2C+sit+amet+tempor+nulla.+Quisque%0Afermentum+felis+faucibus%2C+vehicula+metus+ac%2C+interdum+nibh.%0ACurabitur+vitae+convallis+ligula.+Integer+ac+enim+vel+felis%0Apharetra+laoreet.+Interdum+et+malesuada+fames+ac+ante+ipsum%0Aprimis+in+faucibus.+Pellentesque+hendrerit+ac+augue+quis%0Apretium.%0A%3C%2Fp%3E%0A%3Cp%3E%0AMorbi+ut+scelerisque+nibh.+Integer+auctor%2C+massa+non+dictum%0Atristique%2C+elit+metus+efficitur+elit%2C+ac+pretium+sapien+nisl%0Anec+ante.+In+et+ex+ultricies%2C+mollis+mi+in%2C+euismod+dolor.%0A%3C%2Fp%3E%0A%3Cp%3EQuisque+convallis+ligula+non+magna+efficitur+tincidunt.%3C%2Fp%3E%0A%3C%2Farticle%3E%0A%3C%2F%3E%0A%29%0A%7D%0A%0A%60%60%60)

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam ac rhoncus quam.

Fringilla quam urna. Cras turpis elit, euismod eget ligula quis, imperdiet sagittis justo. In viverra fermentum ex ac vestibulum. Aliquam eleifend nunc a luctus porta. Mauris laoreet augue ut felis blandit, at iaculis odio ultrices. Nulla facilisi. Vestibulum cursus ipsum tellus, eu tincidunt neque tincidunt a.

## Sub-header

In eget sodales arcu, consectetur efficitur metus. Duis efficitur tincidunt odio, sit amet laoreet massa fringilla eu.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aliquam ac rhoncus quam.

Fringilla quam urna. Cras turpis elit, euismod eget ligula quis, imperdiet sagittis justo. In viverra fermentum ex ac vestibulum. Aliquam eleifend nunc a luctus porta. Mauris laoreet augue ut felis blandit, at iaculis odio ultrices. Nulla facilisi. Vestibulum cursus ipsum tellus, eu tincidunt neque tincidunt a.

## Sub-header

In eget sodales arcu, consectetur efficitur metus. Duis efficitur tincidunt odio, sit amet laoreet massa fringilla eu.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

Pellentesque id lacus pulvinar elit pulvinar pretium ac non urna. Mauris id mauris vel arcu commodo venenatis. Aliquam eu risus arcu. Proin sit amet lacus mollis, semper massa ut, rutrum mi.

Sed sem nisi, luctus consequat ligula in, congue sodales nisl.

Vestibulum bibendum at erat sit amet pulvinar. Pellentesque pharetra leo vitae tristique rutrum. Donec ut volutpat ante, ut suscipit leo.

## Sub-header

Maecenas quis elementum nulla, in lacinia nisl. Ut rutrum fringilla aliquet. Pellentesque auctor vehicula malesuada. Aliquam id feugiat sem, sit amet tempor nulla. Quisque fermentum felis faucibus, vehicula metus ac, interdum nibh. Curabitur vitae convallis ligula. Integer ac enim vel felis pharetra laoreet. Interdum et malesuada fames ac ante ipsum primis in faucibus. Pellentesque hendrerit ac augue quis pretium.

Morbi ut scelerisque nibh. Integer auctor, massa non dictum tristique, elit metus efficitur elit, ac pretium sapien nisl nec ante. In et ex ultricies, mollis mi in, euismod dolor.

Quisque convallis ligula non magna efficitur tincidunt.

> Since `scrollY` is a `MotionValue`, there's a neat trick you can use to tell when the user's scroll direction changes:
>
> ```
> const { scrollY } = useScroll()
> const [scrollDirection, setScrollDirection] = useState("down")
>
> useMotionValueEvent(scrollY, "change", (current) => {
>   const diff = current - scrollY.getPrevious()
>   setScrollDirection(diff > 0 ? "down" : "up")
> })
> ```
>
> Perfect for triggering a sticky header animation!
>
> ~ Sam Selikoff, [Motion for React Recipes](https://buildui.com/courses/framer-motion-recipes)

### [Element scroll](https://motion.dev/docs/react-use-scroll\#element-scroll)

To track the scroll position of a scrollable element we can pass the element's `ref` to `useScroll`'s `container` option:

```
const carouselRef = useRef(null)
const { scrollX } = useScroll({
  container: carouselRef
})

return (
  <div ref={carouselRef} style={{ overflow: "scroll" }}>
    {children}
  </div>
)
```

>Live example [Open](https://examples.motion.dev/react/scroll-container)

Element scroll-linked animation — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B%0Aanimate%2C%0Amotion%2C%0AMotionValue%2C%0AuseMotionValue%2C%0AuseMotionValueEvent%2C%0AuseScroll%2C%0A%7D+from+%22motion%2Freact%22%0Aimport+%7B+useRef+%7D+from+%22react%22%0A%0Aexport+default+function+ScrollLinked%28%29+%7B%0Aconst+ref+%3D+useRef%28null%29%0Aconst+%7B+scrollXProgress+%7D+%3D+useScroll%28%7B+container%3A+ref+%7D%29%0Aconst+maskImage+%3D+useScrollOverflowMask%28scrollXProgress%29%0A%0Areturn+%28%0A%3Cdiv+id%3D%22example%22%3E%0A%3Csvg+id%3D%22progress%22+width%3D%2280%22+height%3D%2280%22+viewBox%3D%220+0+100+100%22%3E%0A%3Ccircle+cx%3D%2250%22+cy%3D%2250%22+r%3D%2230%22+pathLength%3D%221%22+className%3D%22bg%22+%2F%3E%0A%3Cmotion.circle%0Acx%3D%2250%22%0Acy%3D%2250%22%0Ar%3D%2230%22%0AclassName%3D%22indicator%22%0Astyle%3D%7B%7B+pathLength%3A+scrollXProgress+%7D%7D%0A%2F%3E%0A%3C%2Fsvg%3E%0A%3Cmotion.ul+ref%3D%7Bref%7D+style%3D%7B%7B+maskImage+%7D%7D%3E%0A%3Cli+style%3D%7B%7B+background%3A+%22var%28--hue-1%29%22+%7D%7D%3E%3C%2Fli%3E%0A%3Cli+style%3D%7B%7B+background%3A+%22var%28--hue-2%29%22+%7D%7D%3E%3C%2Fli%3E%0A%3Cli+style%3D%7B%7B+background%3A+%22var%28--hue-3%29%22+%7D%7D%3E%3C%2Fli%3E%0A%3Cli+style%3D%7B%7B+background%3A+%22var%28--hue-4%29%22+%7D%7D%3E%3C%2Fli%3E%0A%3Cli+style%3D%7B%7B+background%3A+%22var%28--hue-5%29%22+%7D%7D%3E%3C%2Fli%3E%0A%3Cli+style%3D%7B%7B+background%3A+%22var%28--hue-6%29%22+%7D%7D%3E%3C%2Fli%3E%0A%3C%2Fmotion.ul%3E%0A%3CStyleSheet+%2F%3E%0A%3C%2Fdiv%3E%0A%29%0A%7D%0A%0Aconst+left+%3D+%600%25%60%0Aconst+right+%3D+%60100%25%60%0Aconst+leftInset+%3D+%6020%25%60%0Aconst+rightInset+%3D+%6080%25%60%0Aconst+transparent+%3D+%60%230000%60%0Aconst+opaque+%3D+%60%23000%60%0Afunction+useScrollOverflowMask%28scrollXProgress%3A+MotionValue%3Cnumber%3E%29+%7B%0Aconst+maskImage+%3D+useMotionValue%28%0A%60linear-gradient%2890deg%2C+%24%7Bopaque%7D%2C+%24%7Bopaque%7D+%24%7Bleft%7D%2C+%24%7Bopaque%7D+%24%7BrightInset%7D%2C+%24%7Btransparent%7D%29%60%0A%29%0A%0AuseMotionValueEvent%28scrollXProgress%2C+%22change%22%2C+%28value%29+%3D%3E+%7B%0Aif+%28value+%3D%3D%3D+0%29+%7B%0Aanimate%28%0AmaskImage%2C%0A%60linear-gradient%2890deg%2C+%24%7Bopaque%7D%2C+%24%7Bopaque%7D+%24%7Bleft%7D%2C+%24%7Bopaque%7D+%24%7BrightInset%7D%2C+%24%7Btransparent%7D%29%60%0A%29%0A%7D+else+if+%28value+%3D%3D%3D+1%29+%7B%0Aanimate%28%0AmaskImage%2C%0A%60linear-gradient%2890deg%2C+%24%7Btransparent%7D%2C+%24%7Bopaque%7D+%24%7BleftInset%7D%2C+%24%7Bopaque%7D+%24%7Bright%7D%2C+%24%7Bopaque%7D%29%60%0A%29%0A%7D+else+if+%28%0AscrollXProgress.getPrevious%28%29+%3D%3D%3D+0+%7C%7C%0AscrollXProgress.getPrevious%28%29+%3D%3D%3D+1%0A%29+%7B%0Aanimate%28%0AmaskImage%2C%0A%60linear-gradient%2890deg%2C+%24%7Btransparent%7D%2C+%24%7Bopaque%7D+%24%7BleftInset%7D%2C+%24%7Bopaque%7D+%24%7BrightInset%7D%2C+%24%7Btransparent%7D%29%60%0A%29%0A%7D%0A%7D%29%0A%0Areturn+maskImage%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Afunction+StyleSheet%28%29+%7B%0Areturn+%28%0A%3Cstyle%3E%7B%60%0A%23example+%7B%0Awidth%3A+100vw%3B%0Amax-width%3A+400px%3B%0Aposition%3A+relative%3B%0A%7D%0A%0A%23example+%23progress+%7B%0Aposition%3A+absolute%3B%0Atop%3A+-65px%3B%0Aleft%3A+-15px%3B%0Atransform%3A+rotate%28-90deg%29%3B%0A%7D%0A%0A%23example+.bg+%7B%0Astroke%3A+var%28--layer%29%3B%0A%7D%0A%0A%23example+%23progress+circle+%7B%0Astroke-dashoffset%3A+0%3B%0Astroke-width%3A+10%25%3B%0Afill%3A+none%3B%0A%7D%0A%0A%23progress+.indicator+%7B%0Astroke%3A+var%28--accent%29%3B%0A%7D%0A%0A%23example+ul+%7B%0Adisplay%3A+flex%3B%0Alist-style%3A+none%3B%0Aheight%3A+220px%3B%0Aoverflow-x%3A+scroll%3B%0Apadding%3A+20px+0%3B%0Aflex%3A+0+0+600px%3B%0Amargin%3A+0+auto%3B%0Agap%3A+20px%3B%0A%7D%0A%0A%23example+%3A%3A-webkit-scrollbar+%7B%0Aheight%3A+5px%3B%0Awidth%3A+5px%3B%0Abackground%3A+%23fff3%3B%0A-webkit-border-radius%3A+1ex%3B%0A%7D%0A%0A%23example+%3A%3A-webkit-scrollbar-thumb+%7B%0Abackground%3A+var%28--accent%29%3B%0A-webkit-border-radius%3A+1ex%3B%0A%7D%0A%0A%23example+%3A%3A-webkit-scrollbar-corner+%7B%0Abackground%3A+%23fff3%3B%0A%7D%0A%0A%23example+li+%7B%0Aflex%3A+0+0+200px%3B%0Abackground%3A+var%28--accent%29%3B%0A%7D%0A%0A%60%7D%3C%2Fstyle%3E%0A%29%0A%7D%0A%0A%60%60%60)

### [Element position](https://motion.dev/docs/react-use-scroll\#element-position)

We can track the progress of an element as it moves within a container by passing its `ref` to the `target` option.

```
const ref = useRef(null)
const { scrollYProgress } = useScroll({
  target: ref,
  offset: ["start end", "end end"]
})

return <div ref={ref}>
```

In this example, each item has its own progress indicator.

>Live example [Open](https://examples.motion.dev/react/scroll-track-element-in-viewport)

Track element within viewport — Motion for React Example

View source [Open in Cursor](https://cursor.com/link/prompt?text=Add+the+following+to+this+project.+Adapt+according+to+project+styles%3A+e.g.+if+the+project+uses+Tailwind%2C+adapt+styles+to+use+Tailwind+etc.+Install+the+referenced+packages+if+not+already+installed.+If+code+imports+from+motion-plus%2C+install+via+https%3A%2F%2Fapi.motion.dev%2Fregistry%3Fpackage%3Dmotion-plus%26version%3D2.0.0-alpha.4%26token%3DYOUR_AUTH_TOKEN+after+prompting+user+for+YOUR_AUTH_TOKEN.%0A%0A%60%60%60jsx%0A%22use+client%22%0A%0Aimport+%7B+motion%2C+useScroll+%7D+from+%22motion%2Freact%22%0Aimport+%7B+useRef+%7D+from+%22react%22%0A%0Afunction+Item%28%29+%7B%0Aconst+ref+%3D+useRef%28null%29%0Aconst+%7B+scrollYProgress+%7D+%3D+useScroll%28%7B%0Atarget%3A+ref%2C%0Aoffset%3A+%5B%22end+end%22%2C+%22start+start%22%5D%2C%0A%7D%29%0A%0Areturn+%28%0A%3Csection+style%3D%7BitemContainer%7D%3E%0A%3Cdiv+ref%3D%7Bref%7D+style%3D%7Bitem%7D%3E%0A%3Cfigure+style%3D%7BprogressIconContainer%7D%3E%0A%3Csvg%0Astyle%3D%7BprogressIcon%7D%0Awidth%3D%2275%22%0Aheight%3D%2275%22%0AviewBox%3D%220+0+100+100%22%0A%3E%0A%3Ccircle%0Astyle%3D%7BprogressIconBg%7D%0Acx%3D%2250%22%0Acy%3D%2250%22%0Ar%3D%2230%22%0ApathLength%3D%221%22%0AclassName%3D%22bg%22%0A%2F%3E%0A%3Cmotion.circle%0Acx%3D%2250%22%0Acy%3D%2250%22%0Ar%3D%2230%22%0ApathLength%3D%221%22%0Astyle%3D%7B%7B%0A...progressIconIndicator%2C%0ApathLength%3A+scrollYProgress%2C%0A%7D%7D%0A%2F%3E%0A%3C%2Fsvg%3E%0A%3C%2Ffigure%3E%0A%3C%2Fdiv%3E%0A%3C%2Fsection%3E%0A%29%0A%7D%0A%0Aexport+default+function+TrackElementWithinViewport%28%29+%7B%0Areturn+%28%0A%3C%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3CItem+%2F%3E%0A%3C%2F%3E%0A%29%0A%7D%0A%0A%2F**%0A*+%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D+++Styles+++%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%3D%0A*%2F%0A%0Aconst+itemContainer%3A+React.CSSProperties+%3D+%7B%0Aheight%3A+%22100vh%22%2C%0AmaxHeight%3A+%22400px%22%2C%0Adisplay%3A+%22flex%22%2C%0AjustifyContent%3A+%22center%22%2C%0AalignItems%3A+%22center%22%2C%0A%7D%0A%0Aconst+progressIconContainer%3A+React.CSSProperties+%3D+%7B%0Aposition%3A+%22sticky%22%2C%0Atop%3A+0%2C%0Awidth%3A+80%2C%0Aheight%3A+80%2C%0Amargin%3A+0%2C%0Apadding%3A+0%2C%0A%7D%0A%0Aconst+processCircle%3A+React.CSSProperties+%3D+%7B%0AstrokeDashoffset%3A+0%2C%0AstrokeWidth%3A+5%2C%0Afill%3A+%22none%22%2C%0A%7D%0A%0Aconst+progressIcon%3A+React.CSSProperties+%3D+%7B%0A...processCircle%2C%0Atransform%3A+%22translateX%28-100px%29+rotate%28-90deg%29%22%2C%0Astroke%3A+%22var%28--hue-1%29%22%2C%0A%7D%0A%0Aconst+progressIconIndicator%3A+React.CSSProperties+%3D+%7B%0A...processCircle%2C%0AstrokeDashoffset%3A+0%2C%0AstrokeWidth%3A+5%2C%0Afill%3A+%22none%22%2C%0A%7D%0A%0Aconst+progressIconBg%3A+React.CSSProperties+%3D+%7B%0Aopacity%3A+0.2%2C%0A%7D%0A%0Aconst+item%3A+React.CSSProperties+%3D+%7B%0Awidth%3A+200%2C%0Aheight%3A+250%2C%0Aborder%3A+%222px+dotted+var%28--hue-1%29%22%2C%0Aposition%3A+%22relative%22%2C%0A%7D%0A%0A%60%60%60)

### [Scroll offsets](https://motion.dev/docs/react-use-scroll\#scroll-offsets)

With [the](https://motion.dev/docs/react-use-scroll#offset)`offset` [option](https://motion.dev/docs/react-use-scroll#offset) we can define which parts of the element we want to track with the viewport, for instance track elements as they enter in from the bottom, leave at the top, or travel throughout the whole viewport.

## [Performance](https://motion.dev/docs/react-use-scroll\#performance)

Browsers are capable of animating some values, like `opacity`, `transform`, `clipPath` and `filter`, entirely on the GPU. This improves scroll synchronisation and ensures animations remain smooth even when sites are performing heavy work.

`useScroll` is also capable of running animations via the GPU. By passing `scrollXProgress` or `scrollYProgress` either directly to an `opacity` style, or via `useTransform` to one of the above styles, it will create a hardware-accelerated animation.

```
const { scrollYProgress } = useScroll()
const filter = useTransform(scrollYProgress, [0, 1], ["blur(10px)", "blur(0px)"])

return <motion.div style={{ opacity: scrollYProgress, filter }} />
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

## [Options](https://motion.dev/docs/react-use-scroll\#options)

`useScroll` accepts the following options.

### [`container`](https://motion.dev/docs/react-use-scroll\#container)

**Default**: Viewport

The scrollable container to track the scroll position of. By default, this is the browser viewport. By passing a ref to a scrollable element, that element can be used instead.

```
const containerRef = useRef(null)
const { scrollYProgress } = useScroll({ container: containerRef })
```

### [`target`](https://motion.dev/docs/react-use-scroll\#target)

`useScroll` tracks the progress of the `target` within the `container`. By default, the `target` is the scrollable area of the `container`. It can additionally be set as another element, to track its progress within the `container`.

```
const targetRef = useRef(null)
const { scrollYProgress } = useScroll({ target: targetRef })
```

`target` is tracked by the element's layout position, so any CSS `transform` applied to it (or its ancestors) is ignored when measuring progress.

### [`axis`](https://motion.dev/docs/react-use-scroll\#axis)

**Default:**`"y"`

The tracked axis for the defined `offset`.

### [`offset`](https://motion.dev/docs/react-use-scroll\#offset)

**Default:**`["start start", "end end"]`

`offset` describes intersections, points where the `target` and `container` meet.

For example, the intersection `"start end"` means when the **start of the target** on the tracked axis meets the **end of the container.**

So if the target is an element, the container is the window, and we're tracking the vertical axis then `"start end"` is where the **top of the element** meets **the bottom of the viewport**.

#### [Accepted intersections](https://motion.dev/docs/react-use-scroll\#accepted-intersections)

Both target and container points can be defined as:

- **Number:** A value where `0` represents the start of the axis and `1` represents the end. So to define the top of the target with the middle of the container you could define `"0 0.5"`. Values outside this range are permitted.

- **Names:**`"start"`, `"center"` and `"end"` can be used as clear shortcuts for `0`, `0.5` and `1` respectively.

- **Pixels:** Pixel values like `"100px"`, `"-50px"` will be defined as that number of pixels from the start of the target/container.

- **Percent:** Same as raw numbers but expressed as `"0%"` to `"100%"`.

- **Viewport:**`"vh"` and `"vw"` units are accepted.


```
// Track an element as it enters from the bottom
const { scrollYProgress } = useScroll({
  target: targetRef,
  offset: ["start end", "end end"]
})

// Track an element as it moves out the top
const { scrollYProgress } = useScroll({
  target: targetRef,
  offset: ["start start", "end start"]
})
```

### [`trackContentSize`](https://motion.dev/docs/react-use-scroll\#trackcontentsize)

**Default:**`false`

When the size of a page or element's content changes, its scrollable area can change too. But, because browsers don't provide a callback for changes in content size, by default `useScroll()` will not update until the next `"scroll"` event.

`useScroll` can automatically track changes to content size by setting `trackContentSize` to `true`.

```
useScroll({ trackContentSize: true })
```

Content size tracking is disabled by default because most of the time, scrollable area remains stable, and tracking changes to it involves a small overhead.

>Newsletter

### Stay in the loop.

Deep dives on animation, performance, and building Motion. New issues land roughly once a month, no filler.

Subscribe

## Related topics

- [React scroll animation→\\
\\
Scroll-triggered and scroll-linked effects in React: parallax, progress, and more.](https://motion.dev/docs/react-scroll-animations)
- [Motion values overview→\\
\\
Composable, animatable values that update styles without re-rendering React.](https://motion.dev/docs/react-motion-value)
- [React animation→\\
\\
An overview of animating React with motion components, variants, gestures, and keyframes.](https://motion.dev/docs/react-animation)
- [Parallax→\\
\\
Scroll-linked parallax effect where background images move at a different speed to foreground content, creating a sense of depth.](https://motion.dev/tutorials/react-parallax)

[Motion+370+ examplesLifetime updates\\
\\
Motion+ **Level up your animations.** \\
\\
Unlock 370+ premium examples, premium APIs, private Discord and GitHub, and a transition editor for your IDE. One-time purchase, lifetime updates.\\
\\
PricingOne-time payment, lifetime updates\\
\\
Get Motion+](https://motion.dev/plus)

[PrevioususeMotionValueEvent](https://motion.dev/docs/react-use-motion-value-event) [NextuseSpring](https://motion.dev/docs/react-use-spring)

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