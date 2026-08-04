# Transition preview

Generate an image based on a Motion or CSS spring/easing curve. This can help the user and the agent visualise the transition.

## Support

Transition previews require either that the agent supports displaying generated images in the chat interface, or saving the image as a file to a temporary location.

If neither of these are true, inform the user that this capability is not supported in the current UI.

## Usage

Based on the user's input, call the appropriate MCP tool.

### Spring visualisation

For spring-based easing, call `visualise-spring`:

-   **bounce** (number, -1 to 1): How bouncy the spring is
-   **duration** (number, seconds): Duration of the spring animation

Or raw physics parameters:

-   **stiffness**, **damping**, **mass**

### Cubic Bezier visualisation

For cubic-bezier easing, call `visualise-cubic-bezier`:

-   **x1**, **y1**, **x2**, **y2** (numbers): The four control points

### Detecting intent

-   "spring", "bounce", "stiffness", "damping" → `visualise-spring`
-   "cubic-bezier", "bezier", "ease", "easeIn", "easeOut", "easeInOut" → `visualise-cubic-bezier`
-   Named easings map to cubic-bezier: `ease` = `(0.25, 0.1, 0.25, 1)`, `easeIn` = `(0.42, 0, 1, 1)`, `easeOut` = `(0, 0, 0.58, 1)`, `easeInOut` = `(0.42, 0, 0.58, 1)`

### Prompt examples

User: "Show me a bouncy spring"
→ Call `visualise-spring` with `{ "bounce": 0.25, "duration": 0.8 }`

User: "Visualise cubic-bezier(0.4, 0, 0.2, 1)"
→ Call `visualise-cubic-bezier` with `{ "x1": 0.4, "y1": 0, "x2": 0.2, "y2": 1 }`

User: "See easeOut"
→ Call `visualise-cubic-bezier` with `{ "x1": 0, "y1": 0, "x2": 0.58, "y2": 1 }`
