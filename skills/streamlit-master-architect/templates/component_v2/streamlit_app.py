from __future__ import annotations

from pathlib import Path
from typing import Any

import streamlit as st


def _load_js() -> str:
    return (Path(__file__).parent / "frontend" / "component.js").read_text(encoding="utf-8")


def main() -> None:
    st.set_page_config(page_title="SMA — Component v2", page_icon="🧩", layout="centered")
    st.title("Custom component v2 (minimal)")
    st.caption("Demonstrates the v2 contract: data → JS, state/trigger → Python callbacks.")

    if not hasattr(st, "components") or not hasattr(st.components, "v2"):
        st.error("This template requires Streamlit custom components v2 (`st.components.v2`).")
        st.stop()

    counter = st.components.v2.component(
        name="sma_counter",
        js=_load_js(),
        isolate_styles=True,
    )

    def on_clicked() -> None:
        st.toast("Clicked!", icon="✅")

    result: Any = counter(
        label="Click me",
        on_clicked_change=on_clicked,
        key="counter",
    )

    st.subheader("Component result")
    st.write(result)


if __name__ == "__main__":
    main()
