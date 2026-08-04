# Generate a CSS spring

This skill uses Motion to generate a CSS spring via the `linear()` easing curve.

## Usage

Call the `generate-css-spring` MCP tool with the user's spring parameters.

### Parameters

The tool accepts spring configuration:

-   **bounce** (number, -1 to 1): How bouncy the spring is. 0 = no bounce, positive = overshoot, negative = underdamp. Default: 0.
-   **duration** (number, seconds): Duration of the spring animation. Default: 0.8.

Or raw physics parameters:

-   **stiffness** (number): Spring stiffness coefficient
-   **damping** (number): Damping coefficient
-   **mass** (number): Mass of the spring

### Example

User: "Generate a bouncy spring for a modal entrance"

→ Call `generate-css-spring` with `{ "bounce": 0.3, "duration": 0.6 }`

The tool returns a CSS `linear()` easing function and duration that can be used in CSS `transition`, `transition-timing-function` or `animation-timing-function` etc.
