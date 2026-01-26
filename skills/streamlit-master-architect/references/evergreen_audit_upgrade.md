# Evergreen audit + upgrade playbook (keep this skill future-proof)

## 1) Always start with reality: audit the current project

Run:

```bash
python3 <skill_root>/scripts/audit_streamlit_project.py --root <project_root> --format md
```

This reports:
- installed Streamlit version (if present)
- dependency constraints found in `pyproject.toml` / `requirements.txt`
- latest Streamlit version (from PyPI, when enabled)
- risky/deprecated Streamlit APIs and security flags detected

## 2) Keep docs fresh (never rely on stale memory)

Pull the docs graph from `llms.txt`:

```bash
python3 <skill_root>/scripts/sync_streamlit_docs.py --out /tmp/streamlit-docs
```

For deeper offline browsing, fetch HTML pages too:

```bash
python3 <skill_root>/scripts/sync_streamlit_docs.py --out /tmp/streamlit-docs --fetch --max-pages 0
```

(`--max-pages 0` means “fetch all pages listed in llms.txt”.)

## 3) Decide upgrades safely

Upgrade strategy:
- Prefer **patch** upgrades (lowest risk).
- For **minor** upgrades, read the release notes and scan for breaking changes that affect your app.
- For any upgrade: run AppTest smoke + critical E2E smoke.

## 4) Verify with tests

Recommended minimum:
- AppTest smoke on entrypoint pages
- Playwright MCP smoke (load page + screenshot + console errors)

Tip: run tests using the project environment (e.g., `uv run pytest`, `poetry run pytest`, or an activated venv).

## 5) Security hardening checklist

- Do not render untrusted HTML.
- Avoid `unsafe_allow_html=True` and `unsafe_allow_javascript=True` unless absolutely required, then isolate and sanitize inputs.
- Ensure secrets are only sourced from env/secrets systems and never printed.
