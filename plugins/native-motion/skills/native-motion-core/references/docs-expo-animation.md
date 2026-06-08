[Docs](https://docs.expo.dev/)

[Blog](https://expo.dev/blog) [Changelog](https://expo.dev/changelog) [Star Us on GitHub](https://github.com/expo/expo)  [GitHub](https://github.com/expo/expo)

Auto

Hide navigation

Search or Ask AI

`⌘`  `K`

[Home](https://docs.expo.dev/) [Guides](https://docs.expo.dev/guides/overview) [EAS](https://docs.expo.dev/eas) [Reference](https://docs.expo.dev/versions/latest) [Learn](https://docs.expo.dev/tutorial/overview)

Get started

[Create a project](https://docs.expo.dev/get-started/create-a-project) [Set up your environment](https://docs.expo.dev/get-started/set-up-your-environment) [Start developing](https://docs.expo.dev/get-started/start-developing) [Next steps](https://docs.expo.dev/get-started/next-steps)

AI

[Expo Skills](https://docs.expo.dev/skills) [AI agents](https://docs.expo.dev/llms)

Develop

[Tools for development](https://docs.expo.dev/develop/tools) [Navigation](https://docs.expo.dev/develop/app-navigation)

User interface

[Splash screen and app icon](https://docs.expo.dev/develop/user-interface/splash-screen-and-app-icon) [Safe areas](https://docs.expo.dev/develop/user-interface/safe-areas) [System bars](https://docs.expo.dev/develop/user-interface/system-bars) [Fonts](https://docs.expo.dev/develop/user-interface/fonts) [Assets](https://docs.expo.dev/develop/user-interface/assets) [Color themes](https://docs.expo.dev/develop/user-interface/color-themes) [Animation](https://docs.expo.dev/develop/user-interface/animation) [Store data](https://docs.expo.dev/develop/user-interface/store-data) [Next steps](https://docs.expo.dev/develop/user-interface/next-steps)

Development builds

Config plugins

Debugging
[Database](https://docs.expo.dev/develop/database) [Authentication](https://docs.expo.dev/develop/authentication) [Unit testing](https://docs.expo.dev/develop/unit-testing)

Review

[Distributing apps for review](https://docs.expo.dev/review/overview) [Share previews with your team](https://docs.expo.dev/review/share-previews-with-your-team) [Open updates with Orbit](https://docs.expo.dev/review/with-orbit)

Deploy

[Build project for app stores](https://docs.expo.dev/deploy/build-project) [Submit to app stores](https://docs.expo.dev/deploy/submit-to-app-stores) [App stores metadata](https://docs.expo.dev/deploy/app-stores-metadata) [Send over-the-air updates](https://docs.expo.dev/deploy/send-over-the-air-updates) [Deploy web apps](https://docs.expo.dev/deploy/web)

Monitor

[Monitoring services](https://docs.expo.dev/monitoring/services)

More

[Core concepts](https://docs.expo.dev/core-concepts) [FAQ](https://docs.expo.dev/faq)

[Archive](https://docs.expo.dev/archive) [Expo Snack](https://snack.expo.dev/) [Discord and Forums](https://chat.expo.dev/) [Newsletter](https://expo.dev/mailing-list/signup)

# Animation

[Edit page](https://github.com/expo/expo/edit/main/docs/pages/develop/user-interface/animation.mdx)

Copy page

Learn how to integrate React Native animations and use it in your Expo project.

[Edit page](https://github.com/expo/expo/edit/main/docs/pages/develop/user-interface/animation.mdx)

Copy page

* * *

> For the complete documentation index, see [llms.txt](https://docs.expo.dev/llms.txt). Use this file to discover all available pages.

Animations are a great way to enhance and provide a better user experience. In your Expo projects, you can use the [Animated API](https://reactnative.dev/docs/next/animations) from React Native. However, if you want to use more advanced animations with better performance, you can use the [`react-native-reanimated`](https://docs.swmansion.com/react-native-reanimated/) library. It provides an API that simplifies the process of creating smooth, powerful, and maintainable animations.

## Installation

You can skip installing `react-native-reanimated` if you have created a project using [the default template](https://docs.expo.dev/get-started/create-a-project). This library is already installed. Otherwise, install it by running the following command:

**Terminal**

```bash
npx expo install react-native-reanimated react-native-worklets
```

## Usage

### Minimal example

The following example shows how to use the `react-native-reanimated` library to create a simple animation. For more information on the API and advanced usage, see [`react-native-reanimated` documentation](https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/your-first-animation).

Using react-native-reanimated

Copy

Open in Snack

```tsx
import Animated, {
  useSharedValue,
  withTiming,
  useAnimatedStyle,
  Easing,
} from 'react-native-reanimated';
import { View, Button, StyleSheet } from 'react-native';

export default function AnimatedStyleUpdateExample() {
  const randomWidth = useSharedValue(10);

  const config = {
    duration: 500,
    easing: Easing.bezier(0.5, 0.01, 0, 1),
  };

  const style = useAnimatedStyle(() => {
    return {
      width: withTiming(randomWidth.value, config),
    };
  });

  return (
    <View style={styles.container}>
      <Animated.View style={[styles.box, style]} />
      <Button
        title="toggle"
        onPress={() => {
          randomWidth.value = Math.random() * 350;
        }}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  box: {
    width: 100,
    height: 80,
    backgroundColor: 'black',
    margin: 30,
  },
});

Show More
```

## Other animation libraries

You can use other animation packages such as [Moti](https://moti.fyi/) in your Expo project. It works on Android, iOS, and web.

[Previous (Develop - User interface)\\
\\
Color themes](https://docs.expo.dev/develop/user-interface/color-themes) [Next (Develop - User interface)\\
\\
Store data](https://docs.expo.dev/develop/user-interface/store-data)

Was this doc helpful?

- Share your feedback

- [Ask a question on the forums](https://chat.expo.dev/)
- [Edit this page](https://github.com/expo/expo/edit/main/docs/pages/develop/user-interface/animation.mdx)
- View [llms.txt](https://docs.expo.dev/llms.txt) and [llms-full.txt](https://docs.expo.dev/llms-full.txt)
- Last updated on June 03, 2026

Sign up for the Expo Newsletter

Sign Up

Unsubscribe at any time. Read our [privacy policy](https://expo.dev/privacy).

Your Privacy Choices

On this page

[Installation](https://docs.expo.dev/develop/user-interface/animation/#installation)

[Usage](https://docs.expo.dev/develop/user-interface/animation/#usage)

[Minimal example](https://docs.expo.dev/develop/user-interface/animation/#minimal-example)

[Other animation libraries](https://docs.expo.dev/develop/user-interface/animation/#other-animation-libraries)

We value your privacy

We use cookies to collect data and improve our services. [Learn more](https://expo.dev/privacy/cookies)

DeclineAccept

Customize

reCAPTCHA
