# Testing: AppTest (fast, deterministic)

## When to use AppTest

Use AppTest for:
- app loads (smoke tests)
- widget defaults and state transitions
- simple flows that do not need a real browser

## Core API patterns

```python
from streamlit.testing.v1 import AppTest

at = AppTest.from_file("streamlit_app.py").run()
at.text_input[0].input("hello").run()
at.button[0].click().run()
assert "hello" in at.markdown[0].value
```

Notes:
- Prefer `.from_file(...)` for real apps, `.from_string(...)` for tiny focused tests.
- Keep tests offline; mock network/LLM calls.

## Advanced controls (beyond basics)

AppTest can set:
- secrets
- session_state
- query_params

Use this for auth gating, URL-driven pages, and stateful flows.

