---
name: rust-tui-ratatui
description: Implicit Rust TUI and Ratatui skill. Use for terminal UIs, ratatui widgets and layouts, crossterm or termion event loops, terminal rendering architecture, async input, app state machines, keybindings, terminal UX, snapshot tests, and TUI performance.
license: MIT
metadata:
  category: rust
  domains:
    - tui
    - ratatui
    - terminal-ui
    - crossterm
---

# Rust TUI Ratatui

Build terminal applications with clear state ownership, predictable rendering, responsive input, and testable UI contracts.

## Operating Model

1. Identify whether the app is a true TUI, a CLI with occasional prompts, or a daemon dashboard. Use Ratatui only when persistent terminal state and interactive views are justified.
2. Keep rendering pure: widgets read immutable view models and write to frames. Event handlers mutate application state through explicit actions.
3. Use an app/event/action/update/render shape for nontrivial TUIs. Avoid spreading terminal I/O across widgets.
4. Treat terminal cleanup as reliability-critical. Restore raw mode and alternate screen on all normal and error exits.
5. Preserve accessibility of terminal UX: discoverable keybindings, no required color-only state, clear focus, and deterministic fallback for narrow terminals.

## Reference Map

- `references/ratatui-app-architecture.md` for app structure, state machines, view models, layout, widgets, and theme ownership.
- `references/events-async-terminal.md` for crossterm, async tasks, cancellation, terminal restoration, and resize handling.
- `references/testing-ux.md` for buffer assertions, insta snapshots, keybinding tests, and performance checks.

## Defaults

- Prefer `ratatui` with `crossterm` unless the repository has an established backend.
- Use typed actions/events instead of pushing raw key events through business logic.
- Keep long-running work in tasks; never block the render/input loop on network or disk-heavy operations.
- Use bounded channels and cancellation tokens for background work.
- Use `unicode-width` or Ratatui text primitives for display width. Do not assume byte length equals terminal width.

## Verification

Use focused tests for state transitions and rendering:

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

Add render snapshots for stable screens, action/update tests for key flows, and at least one terminal cleanup path test when introducing a new event loop abstraction.
