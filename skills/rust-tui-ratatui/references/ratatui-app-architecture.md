# Ratatui App Architecture

## Structure

For nontrivial TUIs, use explicit layers:

- `App`: owns durable state and mode.
- `Event`: external input such as keys, resize, ticks, task results.
- `Action` or `Message`: semantic intent.
- `update`: transforms state from actions.
- `view model`: computed display data.
- `render`: pure frame drawing from state/view model.

This keeps business behavior testable without a terminal and keeps widgets free of hidden side effects.

## State and Modes

Represent major screens and modal flows with enums. Avoid boolean clusters such as `is_searching`, `is_editing`, `is_help_open` when only one mode can be active.

Keep focus explicit. For tables/lists, centralize selection state and clamp it after data changes.

## Layout

- Use named layout helpers for reused regions.
- Define minimum supported dimensions and fallback views.
- Prefer predictable split constraints over chains of magic percentages.
- Keep theme/style ownership in one module or struct.

## Widgets

Custom widgets should be small and deterministic:

- Inputs: immutable state/view model.
- Outputs: draw calls only.
- No direct I/O.
- No spawning tasks.
- No mutation outside documented state objects.

When a widget becomes complex, split data preparation from drawing rather than introducing mutable global UI state.

## Domain Integration

Keep domain operations behind services or command objects. The TUI should request work and receive results, not own database clients, HTTP retry policy, or file scanners directly inside render/update functions.
