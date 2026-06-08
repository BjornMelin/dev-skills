[Skip to main content](https://rive.app/docs/runtimes/react/react#content-area)

[Rive home page![light logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_black.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=e09b8659935986a837947ae7c3b104e0)![dark logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_white.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=d2ccc2c4dec459d584f578475de89248)](https://rive.app/?utm_source=docs&utm_medium=header_nav)

Search...

Ctrl K

- [Learn](https://rive.app/docs/tutorials/learn-rive)
- [Support](https://rive.app/docs/community/support)
- [Get Rive](https://rive.app/downloads?utm_source=docs&utm_medium=header_nav)
- [Get Rive](https://rive.app/downloads?utm_source=docs&utm_medium=header_nav)

Search...

Navigation

React

React

[Home](https://rive.app/docs/getting-started/introduction) [Editor](https://rive.app/docs/editor/get-rive) [Scripting](https://rive.app/docs/scripting/getting-started) [Runtimes](https://rive.app/docs/runtimes/getting-started)

- [Getting Started](https://rive.app/docs/runtimes/getting-started)
- [Demos](https://rive.app/docs/runtimes/demos)
- [Feature Support](https://rive.app/docs/feature-support)
- [Runtime Sizes](https://rive.app/docs/runtimes/runtime-sizes)
- Choose a Renderer

- [.riv File Format](https://rive.app/docs/runtimes/advanced-topic/format)

### Runtimes

- Web (JS)

- React



  - [React](https://rive.app/docs/runtimes/react/react)
  - [Parameters and Return Values](https://rive.app/docs/runtimes/react/parameters-and-return-values)
  - [Artboards](https://rive.app/docs/runtimes/react/artboards)
  - [Layout](https://rive.app/docs/runtimes/react/layouts)
  - [State Machine Playback](https://rive.app/docs/runtimes/react/state-machines)
  - [Data Binding](https://rive.app/docs/runtimes/react/data-binding)
  - [Loading Assets](https://rive.app/docs/runtimes/react/loading-assets)
  - [Fonts](https://rive.app/docs/runtimes/react/fonts)
  - [Preloading WASM](https://rive.app/docs/runtimes/react/preloading-wasm)
  - [Caching a Rive File](https://rive.app/docs/runtimes/react/caching-a-rive-file)
  - [Playing Audio](https://rive.app/docs/runtimes/react/playing-audio)
  - [Rendering to a Bitmap](https://rive.app/docs/runtimes/react/rendering-to-a-bitmap)
  - Legacy Features

  - [Migration Guides](https://rive.app/docs/runtimes/react/migration-guides)
- React Native

- Flutter

- Apple

- Android

- Unity

- Unreal

- C++

- [Defold](https://rive.app/docs/game-runtimes/defold)
- Community Runtimes


### Integrations

- [3rd Party Integrations](https://rive.app/docs/integrations/overview)
- [HTML Embed](https://rive.app/docs/integrations/html-embed)

## On this page

- [Overview](https://rive.app/docs/runtimes/react/react#overview)
- [Getting Started](https://rive.app/docs/runtimes/react/react#getting-started)
- [Rendering Considerations with useRive](https://rive.app/docs/runtimes/react/react#rendering-considerations-with-userive)
- [Resources](https://rive.app/docs/runtimes/react/react#resources)

React

# React

Copy page

React runtime for Rive.

Copy page

> ## Documentation Index
>
> Fetch the complete documentation index at: [https://uat.rive.app/docs/llms.txt](https://uat.rive.app/docs/llms.txt)
>
> Use this file to discover all available pages before exploring further.

Note that certain Rive features may not be supported yet for a particular runtime, or may require using the Rive Renderer.For more details, refer to the [feature support](https://rive.app/docs/feature-support) and [choosing a renderer](https://rive.app/docs/runtimes/choose-a-renderer) pages.

![Data Binding Quick Start](https://rive.app/docs/images/runtimes/quick-start.gif)

## Data Binding Quick Start

Load and control your Rive (.riv) file.Open the [Rive file](https://rive.app/marketplace/24637-46037-health-bar-data-binding-quick-start/).

[React](https://codesandbox.io/p/sandbox/rive-react-quick-start-4xy76h?file=%2Fsrc%2FApp.tsx%3A77%2C14) [React (Imperative)](https://codesandbox.io/p/devbox/rive-react-vanilla-js-quick-start-kz66t4?file=%2Fsrc%2FApp.tsx%3A53%2C7)

## [​](https://rive.app/docs/runtimes/react/react\#overview)  Overview

This guide documents how to get started using the React runtime library. Rive runtime libraries are open-source. The source is available in its [GitHub repository](https://github.com/rive-app/rive-react).This library contains a React component, as well as custom hooks to help integrate Rive into your web application (types included). Under the hood, this runtime is a React-friendly wrapper around the `@rive-app/webgl2` runtime, exposing types, and Rive instance functionality.

## [​](https://rive.app/docs/runtimes/react/react\#getting-started)  Getting Started

Follow the steps below for a quick start on integrating Rive into your React app.

1

[Navigate to header](https://rive.app/docs/runtimes/react/react#)

Install the dependency

The Rive React runtime allows for two main options based on which backing renderer you need.

- **(Recommended)**`@rive-app/react-webgl2` \- Wraps the `@rive-app/webgl2` dependency, which uses the Rive Renderer.
- `@rive-app/react-canvas` \- Wraps the `@rive-app/canvas` dependency. This does not utilize the Rive Renderer and doesn’t support advanced features, like Vector Feathering.
- `@rive-app/react-canvas-lite` \- Similar to `@rive-app/react-canvas`, but [smaller](https://rive.app/docs/runtimes/web/canvas-vs-webgl). This is not recommended if the Rive file uses [Rive Text](https://rive.app/docs/editor/text) or other advanced features.

To take advantage of the full performance benefits of the Rive Renderer with `react-webgl2`, [enable the draft](https://www.wikihow.tech/Enable-WebGL-Draft-Extensions-in-Google-Chrome)`WEBGL_shader_pixel_local_storage` Chrome Extension (by adding WebGL Draft Extensions).If the draft extension is disabled on a user’s device, Rive will fall back to an MSAA solution (also with WebGL2) on browsers without the extension support.Current work is underway with major browsers to support this extension by default in consumer’s browsers.

```
npm i --save @rive-app/react-webgl2
```

2

[Navigate to header](https://rive.app/docs/runtimes/react/react#)

Render the Rive component

Rive React provides a basic component as its default import for displaying simple animations with a few props you can set such as artboard and layout. Include the code below in your React project to test out an example Rive animation.

```
import Rive from '@rive-app/react-webgl2';

export const Simple = () => (
  <Rive
    src="https://cdn.rive.app/animations/vehicles.riv"
    stateMachines="bumpy"
  />
);
```

See [Parameters and Return Values](https://rive.app/docs/runtimes/react/parameters-and-return-values) for more on the parameters and return values of the `<Rive />` component.

3

[Navigate to header](https://rive.app/docs/runtimes/react/react#)

Using the useRive hook

In many cases, you may not only need the React component to render your animation but also the `rive` object instance that controls it as well. The Rive object instance allows you to tap into APIs for:

- Setting Rive Text values dynamically
- Subscribing to Rive Events with your own callbacks
- Controlling animation playback (i.e. pause and play)
- … and [much more](https://github.com/rive-app/rive-wasm)

The `useRive` hook returns both this `rive` instance, as well as the React component that mounts the underlying `<canvas>` element that Rive will draw onto.

```
import { useRive } from '@rive-app/react-webgl2';

export default function Simple() {
  const { rive, RiveComponent } = useRive({
    src: 'https://cdn.rive.app/animations/vehicles.riv',
    stateMachines: "bumpy",
    autoplay: false,
  });

  return (
    <RiveComponent
      onMouseEnter={() => rive && rive.play()}
      onMouseLeave={() => rive && rive.pause()}
    />
  );
}
```

**Note:** Rive will not instantiate until the `<RiveCopmonent />` is rendered out, as the underlying `<canvas>` element needs to be present in the DOM.

Also, keep in mind that the canvas size depends on the container it’s placed within. Initially, this is 0x0. Either pass a `className` to `RiveComponent` or wrap `RiveComponent` with an appropriately sized container.See [here](https://rive.app/docs/runtimes/react/parameters-and-return-values) for more on the parameters and return values of `useRive`.Additionally, explore subsequent runtime pages to learn how to control animation playback, state machines, and more.

## [​](https://rive.app/docs/runtimes/react/react\#rendering-considerations-with-userive)  Rendering Considerations with useRive

At this time, we highly recommend isolating your usage of `useRive` to its own wrapper component if you plan on conditionally rendering the `<RiveComponent />` returned from the `useRive` hook. This is due to Rive being instanced when the component is mounted and the rendering context associated with a specific underlying `<canvas>` element. When React tries to unmount/re-render, you may end up with the animation restarting or not displaying when a new `<canvas>` is mounted.By isolating `useRive` to its own wrapper component, Rive will have a chance to properly clean up, and restart the animation with a new canvas. In a parent component, you can then conditionally render the wrapper component based on any state or prop-based logic.Check out [this example CodeSandbox](https://codesandbox.io/p/sandbox/rive-react-swapping-skins-with-solos-ctcnlx?file=%2Fsrc%2FApp.tsx) to see this pattern in use.

## [​](https://rive.app/docs/runtimes/react/react\#resources)  Resources

**GitHub**: [https://github.com/rive-app/rive-react](https://github.com/rive-app/rive-react)**Types**: [https://github.com/rive-app/rive-react/blob/main/src/types.ts](https://github.com/rive-app/rive-react/blob/main/src/types.ts)**Examples**

- Simple skinning example: [https://codesandbox.io/p/sandbox/rive-react-swapping-skins-with-solos-ctcnlx](https://codesandbox.io/p/sandbox/rive-react-swapping-skins-with-solos-ctcnlx?file=%2Fsrc%2FApp.tsx)
- Storybook demo: [https://rive-app.github.io/rive-react/](https://rive-app.github.io/rive-react/)
- Animated Login Form:
  - Demo: [https://rive-app.github.io/rive-use-cases/?path=/story/example-loginformcomponent—primary](https://rive-app.github.io/rive-use-cases/?path=/story/example-loginformcomponent--primary)

Was this page helpful?

YesNo

[Suggest edits](https://github.com/rive-app/rive-docs/edit/main/runtimes/react/react.mdx) [Raise issue](https://github.com/rive-app/rive-docs/issues/new?title=Issue%20on%20docs&body=Path:%20/runtimes/react/react)

[FAQ](https://rive.app/docs/runtimes/web/faq) [Parameters and Return Values](https://rive.app/docs/runtimes/react/parameters-and-return-values)

Ctrl+I

[Rive home page![light logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_black.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=e09b8659935986a837947ae7c3b104e0)![dark logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_white.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=d2ccc2c4dec459d584f578475de89248)](https://rive.app/?utm_source=docs&utm_medium=header_nav)

[youtube](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg) [x](https://twitter.com/rive_app) [instagram](https://www.instagram.com/rive.app) [github](https://github.com/rive-app) [discord](https://discord.com/invite/FGjmaTr)

Resources

[Community](https://community.rive.app/?utm_source=docs&utm_medium=footer_nav) [Blog](https://rive.app/blog?utm_source=docs&utm_medium=footer_nav) [Case Studies](https://rive.app/blog/case-studies?utm_source=docs&utm_medium=footer_nav) [Marketplace](https://rive.app/marketplace?utm_source=docs&utm_medium=footer_nav) [Feature Requests](https://community.rive.app/c/feature-requests)

Company

[Careers](https://rive.app/careers?utm_source=docs&utm_medium=footer_nav) [Terms of Service](https://rive.app/docs/legal/terms-of-service) [Acceptable Use Policy](https://rive.app/docs/legal/acceptable-use-policy) [Privacy Policy](https://rive.app/docs/legal/privacy-policy)

[youtube](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg) [x](https://twitter.com/rive_app) [instagram](https://www.instagram.com/rive.app) [github](https://github.com/rive-app) [discord](https://discord.com/invite/FGjmaTr)

![Data Binding Quick Start](https://rive.app/docs/images/runtimes/quick-start.gif)