# Architecture + state (Streamlit-first)

## Mental model

- Streamlit runs top-to-bottom; **any interaction causes a rerun**.
- Widgets keep their values across reruns; `st.session_state` holds additional per-session state.
- Prefer small, pure helper functions for business logic; keep Streamlit code as wiring/rendering.

## Multipage apps (preferred API)

- Use `st.Page` + `st.navigation` in the entrypoint/router.
- Put shared setup (theme, auth checks, shared sidebar, global resources) in the router.
- Put page logic in page files or page functions.

## Session State

Rules:
- Initialize keys once (guard `if key not in st.session_state:`).
- Store stable “base state” only; derive everything else from base state deterministically.
- Avoid putting large dataframes/models in session_state; prefer cache.

## Query parameters (`st.query_params`)

Use query params for shareable state:
- selected item IDs
- filters
- active tab/section

Pattern:
1) Read query params early
2) Validate/coerce types
3) Write back only on meaningful state changes (then `st.rerun()`)

## Forms

Use `st.form` to batch multiple inputs and reduce reruns. Prefer a single “Apply” submit to commit changes.

