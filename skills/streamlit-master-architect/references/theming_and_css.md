# Theming + CSS (modern UI without hacks)

## Prefer config.toml first

Use `.streamlit/config.toml` for consistent theming:
- base (light/dark)
- colors
- fonts
- radius / border tokens (when available)

## Layout defaults

- Prefer `layout="wide"` for dashboards and data apps.
- Use `st.sidebar` for filters; keep primary actions in main body.
- Avoid deep nesting; use containers, tabs, expanders for progressive disclosure.

## CSS injection (last resort)

- Prefer CSS-only injection; do not execute JS.
- If you must inject CSS, keep it minimal and scoped (classes) and document why.

