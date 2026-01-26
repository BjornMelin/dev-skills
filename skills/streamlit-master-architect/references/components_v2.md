# Custom components v2 (contract + best practices)

## v2 vs v1 (why choose v2)

- v1 components run in an iframe and support a single callback.
- v2 components have better performance and can expose multiple callbacks (no iframe).

## Python side (mounting)

Use `st.components.v2.component(...)` to register a component and get a callable back.

Key parameters:
- `name=` stable component name
- `path=` directory with built frontend assets (recommended for production)
- `js=` inline JS (useful for tiny components / tests)
- `isolate_styles=` mount in Shadow DOM (recommended for CSS isolation)

Callback convention:
- frontend state/trigger keys map to Python callback params named `on_<key>_change`.

## Frontend side (JS/TS default export)

The component must export a **default function** that Streamlit calls with:
- `data` (payload from Python)
- `parentElement` (HTMLElement or ShadowRoot)
- `setStateValue(name, value)` (persistent)
- `setTriggerValue(name, value)` (one-rerun trigger)

Type-safe TS authoring:
- Use `@streamlit/component-v2-lib` for **types** (`Component`, `ComponentArgs`, `ComponentState`, theme types).
- Prefer `import type` so bundlers do not depend on runtime exports.

## Reliability checklist

- Create DOM nodes inside `parentElement`, do not replace its `innerHTML` blindly.
- Always return a cleanup function to remove event listeners/timers.
- Use `setTriggerValue` for events (click/submit), `setStateValue` for persistent selections.
- Keep payload small; use Arrow for large tabular data where appropriate.

