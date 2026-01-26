from __future__ import annotations

import streamlit as st


def _common_frame() -> None:
    st.set_page_config(page_title="SMA — Multipage", page_icon="🧭", layout="wide")
    with st.sidebar:
        st.caption("Router frame (executes every rerun).")


def main() -> None:
    _common_frame()

    if not hasattr(st, "Page") or not hasattr(st, "navigation"):
        st.error("This template requires Streamlit with `st.Page` and `st.navigation` (modern multipage API).")
        st.stop()

    pages = [
        st.Page("pages/01_home.py", title="Home", icon="🏠", default=True),
        st.Page("pages/02_reports.py", title="Reports", icon="📈"),
        st.Page("pages/03_settings.py", title="Settings", icon="⚙️"),
    ]

    pg = st.navigation(pages, position="sidebar")
    pg.run()


if __name__ == "__main__":
    main()
