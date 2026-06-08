[Skip to main content](https://rive.app/docs/editor/state-machine/state-machine#content-area)

[Rive home page![light logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_black.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=e09b8659935986a837947ae7c3b104e0)![dark logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_white.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=d2ccc2c4dec459d584f578475de89248)](https://rive.app/?utm_source=docs&utm_medium=header_nav)

Search...

Ctrl K

- [Learn](https://rive.app/docs/tutorials/learn-rive)
- [Support](https://rive.app/docs/community/support)
- [Get Rive](https://rive.app/downloads?utm_source=docs&utm_medium=header_nav)
- [Get Rive](https://rive.app/downloads?utm_source=docs&utm_medium=header_nav)

Search...

Navigation

State Machines

State Machine Overview

[Home](https://rive.app/docs/getting-started/introduction) [Editor](https://rive.app/docs/editor/get-rive) [Scripting](https://rive.app/docs/scripting/getting-started) [Runtimes](https://rive.app/docs/runtimes/getting-started)

- [Get Rive](https://rive.app/docs/editor/get-rive)
- Interface Overview


### Fundamentals

- Design

- Assets

- Animation

- Interaction



  - State Machines



    - [Overview](https://rive.app/docs/editor/state-machine/state-machine)
    - [States](https://rive.app/docs/editor/state-machine/states)
    - [Transitions](https://rive.app/docs/editor/state-machine/transitions)
    - [Layers](https://rive.app/docs/editor/state-machine/layers)
    - [Animation Mixing](https://rive.app/docs/editor/animate-mode/animation-mixing)
  - [Listeners](https://rive.app/docs/editor/state-machine/listeners)
  - Events
- Data

- Publishing


### Advanced Topics

- Rigging & Control

- [Revision History](https://rive.app/docs/editor/fundamentals/revision-history)
- Accessibility

- [AI Agent](https://rive.app/docs/editor/ai-agent/ai-agent)
- Legacy Features


## On this page

- [Overview](https://rive.app/docs/editor/state-machine/state-machine#overview)
  - [Anatomy of a State Machine](https://rive.app/docs/editor/state-machine/state-machine#anatomy-of-a-state-machine)

State Machines

# State Machine Overview

Copy page

Add intelligence to your animations.

Copy page

> ## Documentation Index
>
> Fetch the complete documentation index at: [https://uat.rive.app/docs/llms.txt](https://uat.rive.app/docs/llms.txt)
>
> Use this file to discover all available pages before exploring further.

## [​](https://rive.app/docs/editor/state-machine/state-machine\#overview)  Overview

State Machines are a visual way to connect animations together and define the logic that drives the transitions. They allow you to build interactive motion graphics that are ready to be implemented in your product, app, game, or website.State machines create a new level of collaboration between designers and developers, allowing both teams to iterate deep in the development process without the need for a complicated handoff.

Rive 101 - State Machine Overview - YouTube

Tap to unmute

[Rive 101 - State Machine Overview](https://www.youtube.com/watch?v=0Hb7SlEW6MI) [Rive](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg)

Rive54.4K subscribers

[Watch on](https://www.youtube.com/watch?v=0Hb7SlEW6MI)

Using the State Machine requires designers and animators to think more like a developer but in a straightforward, visual way.Every artboard has at least one State Machine by default, but you can create as many as you’d like. To create a new state machine, hit the plus button in the Animations List and select the State Machine option.

### [​](https://rive.app/docs/editor/state-machine/state-machine\#anatomy-of-a-state-machine)  Anatomy of a State Machine

A basic state machine will consist of a Graph, [States](https://rive.app/docs/editor/state-machine/states), [Transitions](https://rive.app/docs/editor/state-machine/transitions), [Inputs](https://rive.app/docs/editor/state-machine/inputs) and [Layers](https://rive.app/docs/editor/state-machine/layers). We’ll explore each of these pieces and more throughout this section.The Graph is the space in which you’ll be adding States and connecting Transitions. It appears in place of the Timeline when a state machine is selected in the animations list.![State Machine Graph](https://mintcdn.com/rive/IMrXM-oXoMvrTV9Y/images/editor/state-machine/307461c0-2006-4fdf-bdc3-61875d40f422.webp?fit=max&auto=format&n=IMrXM-oXoMvrTV9Y&q=85&s=cea85534542814a5aa2e48c2e51fb478)States are simply timeline animations that can play in your state machine. Typically, these will represent some state that your animated content is in. For example, a button will typically have an Idle state (the button is stationary), a Hovered state (what the button looks like when it is hovered), and a Clicked state (what the button looks like when it’s been clicked).![Preview of States](https://ucarecdn.com/ca93f148-a38c-4eac-a166-8399065315c2/)Once we have defined the States of our content, we can tie them together with transitions to create a logical path that our State Machine can take through these different timelines. We’re creating a map that our State Machine can use to get from one animation to the next.![Creating Transitions](https://ucarecdn.com/cf0f53e3-abc9-43a9-b43a-e18483fe2613/)

**DEPRECATION NOTICE:** This section is about the legacy Inputs system.

**For new projects:** Use [Data Binding](https://rive.app/docs/editor/data-binding) instead.

**For existing projects:** Plan to migrate from Inputs to Data Binding as soon as possible.

**This content is provided for legacy support only.**

Inputs are a legacy tool to control transitions in our state machine. While Inputs can still be used to control transitions, Data Binding is considered best practice since View Models are both more powerful and easier to control at runtime.The best use for Inputs is quick, prototype interactions that you don’t plan to migrate to runtime.Inputs are the contract between designers and developers. As designers, we use them as rules for our transitions to occur. For example, we could have a boolean called isHovered. That boolean controls the transition between our idle and hovered state. When the boolean is true, the state machine is in the hovered state, and when it is false, the state machine is in the Idle state. Developers tie into these inputs at runtime and define actions that control the state machines inputs I.E. defining hit areas that can change the isHovered boolean.![Adding Inputs and Conditions](https://mintcdn.com/rive/06FYAcz4MWxIGBaF/images/editor/state-machine/state-machine-overview-inputs.gif?s=70644948ee4092861583c8af84dc416f)Lastly, all state machines will have at least one Layer. Because only a single animation can play on a given layer, we have the ability to add multiple layers if we want to mix different animations, or add additional interactions. For example, this state machine has multiple layers, each one with the logic to control one of the buttons in this menu.![Image](https://ucarecdn.com/9b454ffc-1e08-495c-a4b7-b6ba71a7cbd2/)

Was this page helpful?

YesNo

[Suggest edits](https://github.com/rive-app/rive-docs/edit/main/editor/state-machine/state-machine.mdx) [Raise issue](https://github.com/rive-app/rive-docs/issues/new?title=Issue%20on%20docs&body=Path:%20/editor/state-machine/state-machine)

[Interpolation (Easing)](https://rive.app/docs/editor/animate-mode/interpolation-easing) [States](https://rive.app/docs/editor/state-machine/states)

Ctrl+I

[Rive home page![light logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_black.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=e09b8659935986a837947ae7c3b104e0)![dark logo](https://mintcdn.com/rive/tBktbqSaEr9ZwywZ/logo/rive_top_logo_white.svg?fit=max&auto=format&n=tBktbqSaEr9ZwywZ&q=85&s=d2ccc2c4dec459d584f578475de89248)](https://rive.app/?utm_source=docs&utm_medium=header_nav)

[youtube](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg) [x](https://twitter.com/rive_app) [instagram](https://www.instagram.com/rive.app) [github](https://github.com/rive-app) [discord](https://discord.com/invite/FGjmaTr)

Resources

[Community](https://community.rive.app/?utm_source=docs&utm_medium=footer_nav) [Blog](https://rive.app/blog?utm_source=docs&utm_medium=footer_nav) [Case Studies](https://rive.app/blog/case-studies?utm_source=docs&utm_medium=footer_nav) [Marketplace](https://rive.app/marketplace?utm_source=docs&utm_medium=footer_nav) [Feature Requests](https://community.rive.app/c/feature-requests)

Company

[Careers](https://rive.app/careers?utm_source=docs&utm_medium=footer_nav) [Terms of Service](https://rive.app/docs/legal/terms-of-service) [Acceptable Use Policy](https://rive.app/docs/legal/acceptable-use-policy) [Privacy Policy](https://rive.app/docs/legal/privacy-policy)

[youtube](https://www.youtube.com/channel/UCPal2R1FxwRTPylhP_7ofEg) [x](https://twitter.com/rive_app) [instagram](https://www.instagram.com/rive.app) [github](https://github.com/rive-app) [discord](https://discord.com/invite/FGjmaTr)

![State Machine Graph](https://mintcdn.com/rive/IMrXM-oXoMvrTV9Y/images/editor/state-machine/307461c0-2006-4fdf-bdc3-61875d40f422.webp?w=1100&fit=max&auto=format&n=IMrXM-oXoMvrTV9Y&q=85&s=693eda9907e0481641b058ebd7b50c5b)

![Preview of States](https://ucarecdn.com/ca93f148-a38c-4eac-a166-8399065315c2/)

![Creating Transitions](https://ucarecdn.com/cf0f53e3-abc9-43a9-b43a-e18483fe2613/)

![Adding Inputs and Conditions](https://mintcdn.com/rive/06FYAcz4MWxIGBaF/images/editor/state-machine/state-machine-overview-inputs.gif?s=70644948ee4092861583c8af84dc416f)

![Image](https://ucarecdn.com/9b454ffc-1e08-495c-a4b7-b6ba71a7cbd2/)