# Generate a CSS spring

Motion can generate a CSS spring as a duration plus `linear()` easing. Use the direct API when
it is available; keep the MCP route for agent-side generation.

## Direct API

```ts
import { spring } from "motion";

const modalTransition = spring(0.5, 0.2);
```

`spring(visualDuration, bounce)` returns a CSS timing string such as `800ms linear(...)`. The
shorthand's `visualDuration` is in seconds, as are Motion transition durations. The
options-object `duration` is milliseconds; use the returned CSS string directly rather than
adding another duration.

Keep a standard transition before the `linear()` version for browsers that do not support it:

```ts
const css = `
.modal { transition: transform 0.3s ease-out; }
@supports (transition-timing-function: linear(0, 1)) {
  .modal { transition: transform ${modalTransition}; }
}
`;
```

## MCP route

Call the `generate-css-spring` MCP tool with the user's spring parameters.

### Parameters

The tool accepts spring configuration:

-   **bounce** (number, 0 to 1): How bouncy the spring is. 0 = no bounce, higher = more overshoot. Default: 0.2.
-   **duration** (number, seconds): Perceptual duration of the spring. Default: 0.4.

Or raw physics parameters:

-   **stiffness** (number): Spring stiffness coefficient
-   **damping** (number): Damping coefficient. Must be greater than 0: a spring with `damping: 0` never settles, so the generator would sample forever.
-   **mass** (number): Mass of the spring

### Example

User: "Generate a bouncy spring for a modal entrance"

→ Call `generate-css-spring` with `{ "bounce": 0.3, "duration": 0.6 }`

The tool returns a CSS `linear()` easing function and duration that can be used in CSS `transition`, `transition-timing-function` or `animation-timing-function` etc.
