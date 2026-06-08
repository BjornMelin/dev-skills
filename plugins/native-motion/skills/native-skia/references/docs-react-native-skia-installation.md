[Skip to main content](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#__docusaurus_skipToContent_fallback)

[![React Native Skia](https://shopify.github.io/react-native-skia/img/logo.png)\\
**React Native Skia**](https://shopify.github.io/react-native-skia/) [Docs](https://shopify.github.io/react-native-skia/docs/getting-started/installation)

[GitHub](https://github.com/shopify/react-native-skia)

Search`Ctrl`  `K`

- [Getting started](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

  - [Installation](https://shopify.github.io/react-native-skia/docs/getting-started/installation)
  - [Hello World](https://shopify.github.io/react-native-skia/docs/getting-started/hello-world)
  - [Web](https://shopify.github.io/react-native-skia/docs/getting-started/web)
  - [Headless](https://shopify.github.io/react-native-skia/docs/getting-started/headless)
  - [Bundle Size](https://shopify.github.io/react-native-skia/docs/getting-started/bundle-size)
- [Canvas](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Painting](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Group](https://shopify.github.io/react-native-skia/docs/group)
- [Pictures](https://shopify.github.io/react-native-skia/docs/shapes/pictures)
- [Shapes](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Images](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Video](https://shopify.github.io/react-native-skia/docs/video)
- [Skottie](https://shopify.github.io/react-native-skia/docs/skottie)
- [Text](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Shaders](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Image Filters](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Backdrop Filters](https://shopify.github.io/react-native-skia/docs/backdrops-filters)
- [Mask Filters](https://shopify.github.io/react-native-skia/docs/mask-filters)
- [Color Filters](https://shopify.github.io/react-native-skia/docs/color-filters)
- [Mask](https://shopify.github.io/react-native-skia/docs/mask)
- [Path Effects](https://shopify.github.io/react-native-skia/docs/path-effects)
- [Animations](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#)

- [Tutorials](https://shopify.github.io/react-native-skia/docs/tutorials)

- [Home page](https://shopify.github.io/react-native-skia/)
- Getting started
- Installation

On this page

# Installation

React Native Skia brings the [Skia Graphics Library](https://skia.org/) to React Native.
Skia serves as the graphics engine for Google Chrome and Chrome OS, Android, Flutter, Mozilla Firefox, Firefox OS, and many other products.

Skill refresh note: Expo SDK 56 bundles `@shopify/react-native-skia@2.6.2`;
upstream latest checked on 2026-06-04 was `2.6.4`. Use the target repo's
package policy and Expo compatibility before copying install commands from this
upstream excerpt.

**Version compatibility:**`react-native@>=0.79` and `react@>=19` are required.

In addition you should make sure you're on at least `iOS 14` and `Android API level 21` or above.

To use React Native Skia with video support, `Android API level 26` or above is required.

For `react-native@<=0.78` and `react@<=18`, you need to use `@shopify/react-native-skia` version `1.12.4` or below.

tvOS, macOS, and macOS Catalyst are also supported platforms.

```sh
yarn add @shopify/react-native-skia
# or
npm install @shopify/react-native-skia
```

This package uses a `postinstall` script to copy Skia prebuilt binaries into the correct location for the native build systems. Some package managers require you to explicitly allow this script to run:

- **Bun**: Add `@shopify/react-native-skia` to `trustedDependencies` in your `package.json`:




```json
{
    "trustedDependencies": ["@shopify/react-native-skia"]
}
```

- **Yarn (Berry/v2+)**: Make sure `enableScripts` is not set to `false` in `.yarnrc.yml`.
- **npm/Yarn Classic**: The postinstall script runs automatically.

## Using Expo [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#using-expo "Direct link to Using Expo")

Expo provides a `with-skia` template, which you can use to create a new project.

```bash
yarn create expo-app my-app -e with-skia
# or
npx create-expo-app my-app -e with-skia
```

### Bundle Size [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#bundle-size "Direct link to Bundle Size")

Below is the app size increase to be expected when adding React Native Skia to your project ( [learn more](https://shopify.github.io/react-native-skia/docs/getting-started/bundle-size)).

| iOS | Android | Web |
| --- | --- | --- |
| 6 MB | 4 MB | 2.9 MB |

## iOS [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#ios "Direct link to iOS")

Run `pod install` on the `ios/` directory.

## Android [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#android "Direct link to Android")

Currently, you will need Android NDK to be installed.
If you have Android Studio installed, make sure `$ANDROID_NDK` is available.
`ANDROID_NDK=$ANDROID_HOME/ndk/<version>` for instance.

If the NDK is not installed, you can install it via Android Studio by going to the menu _File > Project Structure_

And then the _SDK Location_ section. It will show you the NDK path, or the option to download it if you don't have it installed.

### Proguard [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#proguard "Direct link to Proguard")

If you're using Proguard, make sure to add the following rule at `proguard-rules.pro`:

```text
-keep class com.shopify.reactnative.skia.** { *; }
```

### TroubleShooting [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#troubleshooting "Direct link to TroubleShooting")

For error **_CMake 'X.X.X' was not found in SDK, PATH, or by cmake.dir property._**

open _Tools > SDK Manager_, switch to the _SDK Tools_ tab.
Find `CMake` and click _Show Package Details_ and download compatiable version **'X.X.X'**, and apply to install.

## Web [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#web "Direct link to Web")

To use this library in the browser, see [these instructions](https://shopify.github.io/react-native-skia/docs/getting-started/web).

## TV [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#tv "Direct link to TV")

Starting from version [1.9.0](https://github.com/Shopify/react-native-skia/releases/tag/v1.9.0) React Native Skia supports running on TV devices using [React Native TVOS](https://github.com/react-native-tvos/react-native-tvos).
Currently both Android TV and Apple TV are supported.

info

Not all features have been tested yet, so please [report](https://github.com/Shopify/react-native-skia/issues) any issues you encounter when using React Native Skia on TV devices.

## Debugging [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#debugging "Direct link to Debugging")

We recommend using React Native DevTools to debug your JS code — see the [React Native docs](https://reactnative.dev/docs/debugging). Alternatively, you can debug both JS and platform code in VS Code and via native IDEs. If using VS Code, we recommend [Expo Tools](https://github.com/expo/vscode-expo), [Radon IDE](https://ide.swmansion.com/), or Microsoft's [React Native Tools](https://marketplace.visualstudio.com/items?itemName=msjsdiag.vscode-react-native#debugging-react-native-applications).

## Testing with Jest [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#testing-with-jest "Direct link to Testing with Jest")

React Native Skia test mocks use a web implementation that depends on loading CanvasKit.

The very first step is to make sure that your Skia files are not being transformed by jest, for instance, we can add it the `transformIgnorePatterns` directive:

```js
"transformIgnorePatterns": [\
  "node_modules/(?!(react-native|react-native.*|@react-native.*|@?react-navigation.*|@shopify/react-native-skia)/)"\
]
```

You also need to add the following to your `jest.config.js` file:

```js
// jest.config.js
module.exports = {
  // Other values
  testEnvironment: "@shopify/react-native-skia/jestEnv.js",
  setupFilesAfterEnv: [\
    "@shopify/react-native-skia/jestSetup.js",\
  ],
};
```

The `jestEnv.js` will load CanvasKit for you and `jestEnv.js` mocks React Native Skia.
You can also have a look at the [example app](https://github.com/Shopify/react-native-skia/tree/main/apps/example) to see how Jest tests are enabled there.

## Graphite (Experimental) [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#graphite-experimental "Direct link to Graphite (Experimental)")

Skia has two backends: Ganesh (default) and Graphite. An experimental preview of Graphite is available in the `@next` distribution channel:

```sh
yarn add @shopify/react-native-skia@next
```

warning

Graphite support is highly experimental. Skia Graphite requires Android API Level 26 or above.

## Playground [​](https://shopify.github.io/react-native-skia/docs/getting-started/installation/\#playground "Direct link to Playground")

We have example projects you can play with [here](https://github.com/Shopify/react-native-skia/tree/main/apps).
It would require you first to [build Skia locally](https://github.com/shopify/react-native-skia?tab=readme-ov-file#library-development) first.

[Edit this page](https://github.com/shopify/react-native-skia/edit/main/apps/docs/docs/getting-started/installation.md)

[Next\\
\\
Hello World](https://shopify.github.io/react-native-skia/docs/getting-started/hello-world)

- [Using Expo](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#using-expo)
  - [Bundle Size](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#bundle-size)
- [iOS](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#ios)
- [Android](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#android)
  - [Proguard](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#proguard)
  - [TroubleShooting](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#troubleshooting)
- [Web](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#web)
- [TV](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#tv)
- [Debugging](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#debugging)
- [Testing with Jest](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#testing-with-jest)
- [Graphite (Experimental)](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#graphite-experimental)
- [Playground](https://shopify.github.io/react-native-skia/docs/getting-started/installation/#playground)

Resources

- [Documentation](https://shopify.github.io/react-native-skia/docs/getting-started/installation)

More

- [GitHub](https://github.com/shopify/react-native-skia)

Copyright © 2026 Shopify, Inc. Built with Docusaurus.
