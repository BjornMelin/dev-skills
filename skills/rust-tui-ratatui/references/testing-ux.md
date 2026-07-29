# Testing and UX

## Tests

Use state/update tests for most behavior:

- keybinding maps to action
- action mutates state
- selection/focus clamps after data changes
- modal transitions are legal
- task results update visible state

Use render tests for stable screens. Ratatui buffers can be compared directly or snapshot-tested with `insta` after normalizing dynamic content.

## Keybindings

- Keep global keys small and documented in-app.
- Avoid conflicting mode-specific keys.
- Provide an obvious quit path.
- Do not require mouse support for primary workflows.

## Terminal UX

- Support narrow terminals with fallback layouts.
- Avoid color-only state.
- Preserve scroll position when refreshing data if the selected item still exists.
- Keep loading, empty, and error states first-class.

## Performance

Large lists need virtualization or bounded rendering. Avoid reformatting thousands of rows every frame. Precompute expensive strings when data changes, not while drawing.
