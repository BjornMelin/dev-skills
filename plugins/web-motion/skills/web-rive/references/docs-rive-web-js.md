[Skip to main content](https://rive.app/docs/runtimes/web/web-js#content-area)

[Rive home page![light logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_black.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=e09b8659935986a837947ae7c3b104e0)![dark logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_white.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=d2ccc2c4dec459d584f578475de89248)](https://rive.app/?utm_source=docs&utm_medium=header_nav)

Search...

Ctrl K

- [Learn](https://rive.app/docs/tutorials/learn-rive)
- [Support](https://rive.app/docs/community/support)
- [Get Rive](https://rive.app/downloads?utm_source=docs&utm_medium=header_nav)
- [Get Rive](https://rive.app/downloads?utm_source=docs&utm_medium=header_nav)

Search...

Navigation

Web (JS)

Getting Started

[Home](https://rive.app/docs/getting-started/introduction) [Editor](https://rive.app/docs/editor/get-rive) [Scripting](https://rive.app/docs/scripting/getting-started) [Runtimes](https://rive.app/docs/runtimes/getting-started)

- [Getting Started](https://rive.app/docs/runtimes/getting-started)
- [Demos](https://rive.app/docs/runtimes/demos)
- [Feature Support](https://rive.app/docs/feature-support)
- [Runtime Sizes](https://rive.app/docs/runtimes/runtime-sizes)
- Choose a Renderer

- [.riv File Format](https://rive.app/docs/runtimes/advanced-topic/format)

### Runtimes

- Web (JS)



  - [Getting Started](https://rive.app/docs/runtimes/web/web-js)
  - [Canvas vs WebGL2](https://rive.app/docs/runtimes/web/canvas-vs-webgl)
  - [Rive Parameters](https://rive.app/docs/runtimes/web/rive-parameters)
  - [Artboards](https://rive.app/docs/runtimes/web/artboards)
  - [Layout](https://rive.app/docs/runtimes/web/layouts)
  - [State Machine Playback](https://rive.app/docs/runtimes/web/state-machines)
  - [Data Binding](https://rive.app/docs/runtimes/web/data-binding)
  - [Loading Assets](https://rive.app/docs/runtimes/web/loading-assets)
  - [Fonts](https://rive.app/docs/runtimes/web/fonts)
  - [Preloading WASM](https://rive.app/docs/runtimes/web/preloading-wasm)
  - [Caching a Rive File](https://rive.app/docs/runtimes/web/caching-a-rive-file)
  - [Playing Audio](https://rive.app/docs/runtimes/web/playing-audio)
  - [Low-level API Usage](https://rive.app/docs/runtimes/web/low-level-api-usage)
  - Legacy Features

  - [Migration Guides](https://rive.app/docs/runtimes/web/migration-guides)
  - [FAQ](https://rive.app/docs/runtimes/web/faq)
- React

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

- [Overview](https://rive.app/docs/runtimes/web/web-js#overview)
- [Getting started](https://rive.app/docs/runtimes/web/web-js#getting-started)
  - [Complete example](https://rive.app/docs/runtimes/web/web-js#complete-example)
  - [Loading Rive files](https://rive.app/docs/runtimes/web/web-js#loading-rive-files)
- [Clean up Rive](https://rive.app/docs/runtimes/web/web-js#clean-up-rive)
- [Additional Rive web resources](https://rive.app/docs/runtimes/web/web-js#additional-rive-web-resources)
- [Examples](https://rive.app/docs/runtimes/web/web-js#examples)

Web (JS)

# Getting Started

Copy page

JavaScript/WASM runtime for Rive.

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

[Web](https://codesandbox.io/p/sandbox/rive-quick-start-js-xmwcm6?file=%2Fsrc%2Findex.ts)

## [​](https://rive.app/docs/runtimes/web/web-js\#overview)  Overview

This guide documents how to get started using the Rive web runtime library. The runtime is open source and available in this [GitHub repository](https://github.com/rive-app/rive-wasm). This library has a high-level JavaScript API (with TypeScript support) and a low-level API to load in Web Assembly (WASM) and control the rendering loop yourself. This runtime allows you to:

- Quickly integrate Rive into all web applications (Webflow, WordPress, etc.)
- Provides a base API to build other web-based Rive runtime wrappers (React, Svelte, etc.)
- Support advanced use cases by controlling the render loop (web-based game engines)

## [​](https://rive.app/docs/runtimes/web/web-js\#getting-started)  Getting started

Follow the steps below to integrate Rive into your web app.

The following instructions describe using the `@rive-app/webgl2` package. Rive provides web-based packages like WebGL2, Canvas, and Lite versions.See [Canvas vs WebGL2](https://rive.app/docs/runtimes/web/canvas-vs-webgl) for guidance on which package is the correct choice for your use case.

1

[Navigate to header](https://rive.app/docs/runtimes/web/web-js#)

Install the dependency

We recommend always using the [latest version](https://www.npmjs.com/package/@rive-app/webgl2). The versions listed below and in the examples may differ from the latest.

- Script Tag

- Package Manager


```
// Add the following script tag to your web page to get the latest version:

<script src="https://unpkg.com/@rive-app/webgl2"></script>

// You can also pin to a specific version (See [here](https://www.npmjs.com/package/@rive-app/webgl2) for the latest):

<script src="https://unpkg.com/@rive-app/webgl2@2.36.0"></script>

// This will make a global `rive` object available, allowing you to access the Rive API via the `rive` entry point:

new rive.Rive({});
```

npm

```
npm install @rive-app/webgl2
```

pnpm

```
pnpm add @rive-app/webgl2
```

yarn

```
yarn add @rive-app/webgl2
```

bun

```
bun add @rive-app/webgl2
```

Importing

```
// Import the entire module under the global identifier `rive`
import * as rive from "@rive-app/webgl2";

// Alternatively, import only the specific parts you need
import { Rive } from "@rive-app/webgl2";
```

Not using [Rive Text](https://rive.app/docs/editor/text), [Rive Layouts](https://rive.app/docs/editor/layouts), [Rive Scripting](https://rive.app/docs/scripting), or [Rive Audio](https://rive.app/docs/editor/events/audio-events)? Consider using [@rive-app/canvas-lite](https://rive.app/docs/runtimes/web/canvas-vs-webgl#rive-app-webgl2-lite) which is a smaller package variant of our canvas runtime.

2

[Navigate to header](https://rive.app/docs/runtimes/web/web-js#)

Create a Canvas

Add a canvas element to your HTML where you want the Rive graphic to be displayed:

```
<canvas id="canvas" width="500" height="500"></canvas>
```

3

[Navigate to header](https://rive.app/docs/runtimes/web/web-js#)

Create a Rive instance

To create a new instance of a Rive object, provide the following properties:

- `src`: A string representing the URL of the hosted `.riv` file (as shown in the example below) or the path to the public asset `.riv` file. For more details, refer to [Rive Parameters](https://rive.app/docs/runtimes/web/rive-parameters) on how to properly use this property.
- `artboard` \- (Optional) A string representing the artboard you want to display. If not supplied, the default artboard from the `.riv` file is selected.
- `stateMachines` \- A string representing the name of the state machine you wish to play. This must be supplied, or the Rive instance may only play the first linear animation it finds. In the next major version, the default behavior will be to play the default state machine of the artboard.
- `canvas` \- The canvas element where the animation will be rendered.
- `autoplay` \- A boolean indicating whether the animation should play automatically.
- `autoBind` \- A boolean indicating whether to automatically data-bind the default `ViewModelInstance` if one is found.

```
<script>
    const r = new rive.Rive({
        src: "https://cdn.rive.app/animations/vehicles.riv",
        // OR the path to a discoverable and public Rive asset
        // src: '/public/example.riv',
        canvas: document.getElementById("canvas"),
        autoplay: true,
        autoBind: true,
        // artboard: "Artboard", // Optional. If not supplied the default is selected
        stateMachines: "bumpy",
        onLoad: () => {
          r.resizeDrawingSurfaceToCanvas();
        },
    });
</script>
```

The `resizeDrawingSurfaceToCanvas` method ensures that the Rive animation is correctly scaled to fit the dimensions of the specified canvas element. By default, the canvas rendering surface might not match the exact size of the `<canvas>` element defined in your HTML, which can lead to blurry or incorrectly scaled graphics, especially on high-DPI or retina displays.Calling this method adjusts the internal drawing surface so that the animation is rendered with crisp detail, matching the pixel density of the canvas. This is particularly important when:

- The size of the canvas changes dynamically (e.g., if it is resized due to responsive layouts).
- You want to ensure the animation remains sharp, regardless of device or screen resolution.

**Best practices:**

- **Call after load**: It’s recommended to call `resizeDrawingSurfaceToCanvas` inside the `onLoad` callback to ensure that the Rive asset has been fully loaded before adjusting the drawing surface. This prevents any rendering issues.
- **Handling window resize**: If your canvas size changes during the user’s interaction (such as when resizing the browser window), you should also listen for window resize events and call `resizeDrawingSurfaceToCanvas` to re-adjust the rendering surface:

```
window.addEventListener("resize", () => {
    r.resizeDrawingSurfaceToCanvas();
});
```

This way, the Rive animation will continue to look sharp and correctly scaled as the canvas size changes.

### [​](https://rive.app/docs/runtimes/web/web-js\#complete-example)  Complete example

Bringing it all together, here’s how to load a Rive graphic in a single HTML file.

```
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Rive Hello World</title>
  </head>
  <body>
    <canvas id="canvas" width="500" height="500"></canvas>

    <script src="https://unpkg.com/@rive-app/webgl2"></script>
    <script>
      const r = new rive.Rive({
        src: "https://cdn.rive.app/animations/vehicles.riv",
        canvas: document.getElementById("canvas"),
        autoplay: true,
        // artboard: "Arboard", // Optional. If not supplied the default is selected
        stateMachines: "bumpy",
        onLoad: () => {
          // Ensure the drawing surface matches the canvas size and device pixel ratio
          r.resizeDrawingSurfaceToCanvas();
        },
      });
    </script>
  </body>
</html>
```

### [​](https://rive.app/docs/runtimes/web/web-js\#loading-rive-files)  Loading Rive files

[See this example](https://codesandbox.io/p/sandbox/rive-quick-start-js-xmwcm6?file=%2Fsrc%2Findex.ts) for the different ways to load in a .riv file, the options are:

1. **Hosted URL**: Use a string representing the URL where the `.riv` file is hosted. Set this as the `src` attribute when creating a new Rive instance.
2. **Static Assets in the bundle**: Provide a string with the path to a publicly accessible `.riv` file within your web project. Handle `.riv` files just like any other static asset (e.g., images or fonts) in your project.
3. **Fetching a file**: Instead of using the `src` attribute, use the `buffer` attribute to load an `ArrayBuffer` when fetching a file. This is useful when reusing the same `.riv` file across multiple Rive instances, allowing you to load it only once.
4. **Reusing a Loaded File**: Use the `rivFile` parameter to reuse a previously loaded Rive runtime file object, avoiding the need to fetch it again via the `src` URL or reload it from the `buffer`. This can significantly improve performance by eliminating redundant network requests and loading times, especially when creating multiple Rive instances from the same source. Unlike the `src` and `buffer` parameters, which require parsing under the hood to create a runtime file object, the `riveFile` parameter uses an already parsed object, including any loaded assets. See [Caching a Rive File](https://rive.app/docs/runtimes/web/caching-a-rive-file).

For more details, refer to the [Rive Parameters](https://rive.app/docs/runtimes/web/rive-parameters) section on the `src` property.

## [​](https://rive.app/docs/runtimes/web/web-js\#clean-up-rive)  Clean up Rive

When working with a Rive instance, it’s important to properly clean it up when it’s no longer needed. This is especially necessary in scenarios where:

- The UI containing Rive animations is no longer needed (e.g., when a modal with Rive graphics is closed).
- The animation or state machine has completed and will not be shown or run again.

Under the hood, Rive creates various low-level objects (such as artboard instances, animation instances, and state machine instances) in C++, which need to be manually deleted to prevent memory leaks. If not cleaned up, these objects can consume unnecessary resources, potentially impacting your application’s performance.Fortunately, the high-level JavaScript API simplifies this process. You don’t need to track every object created during the Rive instance lifecycle. Instead, you can clean up all associated objects with a single method call.To clean up a Rive instance and free up resources, simply call the following method on your Rive instance:

```
const riveInstance = new Rive({...));
...
// When ready to cleanup
riveInstance.cleanup();
```

# [​](https://rive.app/docs/runtimes/web/web-js\#additional-rive-web-resources)  Additional Rive web resources

More in-depth Rive web documentation and advanced use cases.

[**Rive Parameters** \\
\\
API docs for the Rive instance.](https://rive.app/docs/runtimes/web/rive-parameters)

[**Canvas vs WebGL2** \\
\\
A guide to the different Rive web packages](https://rive.app/docs/runtimes/web/canvas-vs-webgl)

[**FAQ** \\
\\
Frequently asked questions](https://rive.app/docs/runtimes/web/faq)

[**Preloading WASM** \\
\\
Instructions on how to preload and self-host the rive WASM library.](https://rive.app/docs/runtimes/web/preloading-wasm)

[**Low-level API Usage** \\
\\
Control the Rive render loop and layout, and draw multiple artboards to the same canvas.](https://rive.app/docs/runtimes/web/low-level-api-usage)

# [​](https://rive.app/docs/runtimes/web/web-js\#examples)  Examples

- [Basic gallery app](https://github.com/rive-app/rive-wasm/tree/master/js/examples/_frameworks/parcel_example_canvas)
- [Tracking mouse cursor](https://codesandbox.io/p/sandbox/tracking-mouse-cursor-n38gdd?file=%2Fsrc%2Findex.ts)
- [Connecting to page scroll](https://codesandbox.io/p/sandbox/rive-page-scroll-h4msqw?file=%2Fsrc%2Findex.ts%3A27%2C45)
- [Playing state machine only when scrolled into the user’s viewport](https://codesandbox.io/p/sandbox/rive-wait-for-scroll-into-view-y9wg8d?file=%2Fsrc%2Findex.ts)

Was this page helpful?

YesNo

[Suggest edits](https://github.com/rive-app/rive-docs/edit/main/runtimes/web/web-js.mdx) [Raise issue](https://github.com/rive-app/rive-docs/issues/new?title=Issue%20on%20docs&body=Path:%20/runtimes/web/web-js)

[.riv File Format](https://rive.app/docs/runtimes/advanced-topic/format) [Canvas vs WebGL2](https://rive.app/docs/runtimes/web/canvas-vs-webgl)

Ctrl+I

[Rive home page![light logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_black.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=e09b8659935986a837947ae7c3b104e0)![dark logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_white.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=d2ccc2c4dec459d584f578475de89248)](https://rive.app/?utm_source=docs&utm_medium=header_nav)

[youtube](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg) [x](https://twitter.com/rive_app) [instagram](https://www.instagram.com/rive.app) [github](https://github.com/rive-app) [discord](https://discord.com/invite/FGjmaTr)

Resources

[Community](https://community.rive.app/?utm_source=docs&utm_medium=footer_nav) [Blog](https://rive.app/blog?utm_source=docs&utm_medium=footer_nav) [Case Studies](https://rive.app/blog/case-studies?utm_source=docs&utm_medium=footer_nav) [Marketplace](https://rive.app/marketplace?utm_source=docs&utm_medium=footer_nav) [Feature Requests](https://community.rive.app/c/feature-requests)

Company

[Careers](https://rive.app/careers?utm_source=docs&utm_medium=footer_nav) [Terms of Service](https://rive.app/docs/legal/terms-of-service) [Acceptable Use Policy](https://rive.app/docs/legal/acceptable-use-policy) [Privacy Policy](https://rive.app/docs/legal/privacy-policy)

[youtube](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg) [x](https://twitter.com/rive_app) [instagram](https://www.instagram.com/rive.app) [github](https://github.com/rive-app) [discord](https://discord.com/invite/FGjmaTr)

![Data Binding Quick Start](https://rive.app/docs/images/runtimes/quick-start.gif)