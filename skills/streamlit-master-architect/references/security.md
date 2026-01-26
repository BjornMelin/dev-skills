# Security (practical hardening)

## `st.html` and untrusted input

- HTML is sanitized; JavaScript is ignored by default.
- JS execution requires explicit opt-in (`unsafe_allow_javascript=True`) and is high risk.
- Never pass untrusted user input into HTML/JS sinks.

## Secrets

- Never hardcode secrets in code or templates.
- Use `.streamlit/secrets.toml` locally; use platform secret stores in deployment.
- Avoid printing secrets to logs.

## Auth boundaries (multipage apps)

- Put auth checks in the router/entrypoint for consistent gating.
- Keep public vs private pages explicit.
- For E2E tests, use test accounts and environment-injected credentials.

