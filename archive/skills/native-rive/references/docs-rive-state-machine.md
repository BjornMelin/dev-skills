# Rive State Machines And Data Binding

Use this file when implementing or reviewing Rive interactivity contracts.

## Source Snapshot

- Runtime state machines:
  https://rive.app/docs/runtimes/react-native/state-machines
- Runtime data binding:
  https://rive.app/docs/runtimes/react-native/data-binding
- Rive ref methods:
  https://rive.app/docs/runtimes/react-native/rive-ref-methods
- Editor state-machine overview:
  https://rive.app/docs/editor/state-machine
- Package metadata inspected: `@rive-app/react-native` 0.4.10.

## Mental Model

State machines should stay designer-owned. Runtime code should not duplicate or
reach around the graph unless a migration requires it. The preferred contract is:

1. The Rive file defines artboards, state machines, view models, instances, and
   bindings in the editor.
2. App code loads a `RiveFile` and creates or selects a `ViewModelInstance`.
3. App code reads/writes typed properties on that instance.
4. `RiveView` receives the instance through `dataBind`.
5. The state machine or artboard advances and applies bound values.

Changing a property value does not always immediately render when the state
machine is settled. Call `playIfNeeded()` when a settled state machine must
advance after a data-binding update.

## Data Binding First

Use these new-runtime APIs for product state:

- `useViewModelInstance(riveFile, options)`
- `useRiveNumber(path, instance)`
- `useRiveBoolean(path, instance)`
- `useRiveString(path, instance)`
- `useRiveColor(path, instance)`
- `useRiveEnum(path, instance)`
- `useRiveList(path, instance)`
- `useRiveTrigger(path, instance, { onTrigger })`

Use `onInit` for values that must exist before first render/advance:

```tsx
const { instance } = useViewModelInstance(riveFile, {
  artboardName: RIVE_CONTRACT.artboard,
  viewModelName: RIVE_CONTRACT.viewModel,
  instanceName: RIVE_CONTRACT.instance,
  onInit: (vmi) => {
    vmi.numberProperty(RIVE_CONTRACT.properties.progress)?.set(0);
  },
});
```

Bind explicitly when defaults are a product risk:

```tsx
<RiveView
  file={riveFile}
  artboardName={RIVE_CONTRACT.artboard}
  stateMachineName={RIVE_CONTRACT.stateMachine}
  dataBind={instance}
  onError={handleRiveError}
/>
```

`DataBindMode.Auto` can be useful for prototypes, but it depends on the editor's
default view model and default instance. Prefer explicit artboard/view-model/
instance selection for production flows with domain state.

## Property Hooks

Property paths are runtime contracts. Keep them centralized:

```tsx
const RIVE_CONTRACT = {
  artboard: 'Checkout',
  stateMachine: 'Checkout Machine',
  viewModel: 'Checkout VM',
  instance: 'Default',
  properties: {
    progress: 'checkout/progress',
    status: 'checkout/status',
    submit: 'checkout/submit',
  },
} as const;
```

Hook usage should check hook errors and preserve null/undefined states:

```tsx
const progress = useRiveNumber(RIVE_CONTRACT.properties.progress, instance);
const status = useRiveEnum(RIVE_CONTRACT.properties.status, instance);
const submit = useRiveTrigger(RIVE_CONTRACT.properties.submit, instance);
```

Nested property paths use slash-delimited strings. Chain notation is not the
stable cross-runtime contract for React Native.

## Dynamic Images, Lists, Artboards, Enums

- Image properties and referenced image assets need loading/fallback behavior,
  cache and privacy decisions, and `playIfNeeded()` when a settled state machine
  must update immediately.
- Lists and nested view models should be treated as typed product contracts.
  Validate item count, order, default/empty states, and designer expectations.
- Artboard properties can turn Rive files into nested runtime compositions.
  Confirm renderer/platform support and memory cost.
- Enums are string contracts from the editor. Validate unknown values before
  setting them from app/domain state.

## Legacy Inputs

The new package source/docs mark these ref methods as deprecated in favor of
data binding:

- `setNumberInputValue`
- `getNumberInputValue`
- `setBooleanInputValue`
- `getBooleanInputValue`
- `triggerInput`
- `setTextRunValue`
- `getTextRunValue`
- `onEventListener`
- `removeEventListeners`

Use them only for legacy migration or when the `.riv` file has not yet moved to
view-model binding. If used, centralize names, add listener cleanup, and record
the migration decision.

## Contract Checklist

Record these before implementation:

- Rive package and version.
- `.riv` file path and loading mode: Metro `require`, native resource, URL,
  `{ uri }`, or `ArrayBuffer`.
- Metro `.riv` support or native resource bundle proof.
- Artboard name and whether the default artboard is acceptable.
- State-machine name and whether the default state machine is acceptable.
- View-model name, instance name, and whether defaults are acceptable.
- Bound property paths with type: number, boolean, string, color, enum, list,
  image, artboard, trigger, or nested view model.
- Referenced asset keys and source policy.
- Expected first-render values and where they are initialized.
- Reduced-motion fallback: static artboard, poster, paused state, or alternate
  native UI.
- Error/loading fallback and user-facing copy.
- Platform proof: iOS, Android, route unmount, and native rebuild when native
  setup changed.

## Reduced Motion

Decide whether the Rive file is decorative or state-bearing:

- Decorative/ambient loops: skip playback and render a static poster/final
  state when reduced motion is enabled.
- State-bearing UI: preserve information with less motion. Set bound values,
  avoid nonessential triggers, and pause once the informative state is visible.
- Audio or haptics tied to Rive events require the same accessibility review as
  visual motion.

## Testing Notes

- Unit tests can validate wrapper state mapping, contract constants, and
  fallback branches.
- Device/simulator tests must prove real rendering, native package linkage,
  data-bound property changes, trigger behavior, and route cleanup.
- Wrong names often surface through `onError`, not TypeScript. Test missing or
  renamed artboard, state-machine, view-model, instance, property, and asset
  paths when the wrapper owns fallback behavior.
- Add a design-export review step when `.riv` files change. TypeScript cannot
  detect a designer-renamed property or state-machine input.
