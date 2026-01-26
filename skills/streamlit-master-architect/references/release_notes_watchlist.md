# Release notes watchlist (keep API usage correct over time)

Use this file as a **version-aware checklist**, not a static “truth table”. It is intentionally short and process-oriented so it stays useful across future Streamlit versions.

## Always start with the project’s actual version

Prefer running inside the project environment:

```bash
uv run python -c "import streamlit as st; print(st.__version__)"
```

If Streamlit isn’t installed in the current interpreter, use lockfiles:
- `uv.lock` → locked `streamlit` version
- `poetry.lock` → locked `streamlit` version

You can also run the bundled audit:

```bash
python3 <skill_root>/scripts/audit_streamlit_project.py --root <project_root> --format md
```

## Docs vs patch releases (how to stay current)

- Streamlit docs release notes primarily emphasize **minor** releases (feature-level changes).
- Patch release details are easiest to track via **GitHub releases**.

Evergreen approach:
1) Refresh docs URLs and pages from `llms.txt`:
   ```bash
   python3 <skill_root>/scripts/sync_streamlit_docs.py --out /tmp/streamlit-docs --fetch --max-pages 0
   ```
2) Search the snapshot for relevant pages/keywords (use `rg`):
   ```bash
   rg -n \"Release notes|1\\.52|breaking|deprecated\" /tmp/streamlit-docs/pages | head
   ```

## Upgrade safety checklist (do this every time)

1) Run the audit script; fix findings (deprecated APIs, unsafe flags).
2) Run AppTest smoke tests.
3) Run Playwright MCP E2E smoke on critical flows.
4) Re-check widget keys and rerun hotspots (caching/fragments/forms).

## Example snapshot: Streamlit 1.52.x highlights (Dec 2025)

Treat this as a **memory jogger**. Always confirm against official docs/release notes for your target version.

Notable changes to use in new code:
- `st.datetime_input` for combined date+time selection.
- `st.download_button(data=callable)` for deferred generation (compute on click). Docs note the callable executes on click and runs on a separate thread from the resulting script rerun.
- `st.chat_input(..., accept_audio=True, audio_sample_rate=...)` for optional audio (guard with `try/except TypeError` if you need compatibility).
- `st.html(..., unsafe_allow_javascript=...)` explicitly gates JS execution (still high risk; never use with untrusted input).
- Widget identity change: `st.file_uploader` and `st.camera_input` use `key` as their primary identity (watch dynamic widget trees).

Breaking/migration note:
- Native Bokeh support removed (apps relying on Bokeh integration should migrate to Altair/Plotly/Vega-Lite).

