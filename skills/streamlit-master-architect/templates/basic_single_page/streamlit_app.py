from __future__ import annotations

import datetime as dt
import io

import pandas as pd
import streamlit as st


APP_TITLE = "SMA Demo — Single Page"


def _set_page() -> None:
    st.set_page_config(page_title=APP_TITLE, page_icon="🧱", layout="wide")


@st.cache_data(show_spinner="Loading demo dataset…")
def load_demo_data(n: int = 2_000) -> pd.DataFrame:
    now = dt.datetime.now()
    df = pd.DataFrame(
        {
            "ts": pd.date_range(now - dt.timedelta(days=30), periods=n, freq="h"),
            "value": pd.Series(range(n)).astype(float).rolling(24, min_periods=1).mean(),
            "group": (pd.Series(range(n)) % 5).map(lambda x: f"g{x}"),
        }
    )
    return df


def _make_csv_bytes(df: pd.DataFrame) -> bytes:
    buf = io.StringIO()
    df.to_csv(buf, index=False)
    return buf.getvalue().encode("utf-8")


def main() -> None:
    _set_page()

    st.title("Streamlit patterns demo")
    st.caption(
        "Demonstrates caching, datetime input, deferred download generation, chat input (audio optional), and safe HTML."
    )

    df = load_demo_data()

    with st.sidebar:
        st.header("Controls")
        group = st.selectbox("Group", sorted(df["group"].unique()), key="group")

        def dt_input(label: str, *, key_prefix: str) -> dt.datetime:
            now = dt.datetime.now()
            if hasattr(st, "datetime_input"):
                v: dt.datetime | str = "now"
                out = st.datetime_input(label, value=v, key=key_prefix)
                return out if isinstance(out, dt.datetime) else now
            d = st.date_input(f"{label} (date)", value=now.date(), key=f"{key_prefix}__date")
            t = st.time_input(f"{label} (time)", value=now.time().replace(second=0, microsecond=0), key=f"{key_prefix}__time")
            return dt.datetime.combine(d, t)

        start = dt_input("Start datetime", key_prefix="start_dt")
        end = dt_input("End datetime", key_prefix="end_dt")

    if isinstance(start, dt.datetime) and isinstance(end, dt.datetime) and start > end:
        st.error("Start must be <= end.")
        st.stop()

    filtered = df[df["group"] == group].copy()
    if isinstance(start, dt.datetime):
        filtered = filtered[filtered["ts"] >= start.replace(tzinfo=None)]
    if isinstance(end, dt.datetime):
        filtered = filtered[filtered["ts"] <= end.replace(tzinfo=None)]

    st.subheader("Preview")
    st.dataframe(filtered.head(200), height=320)

    st.subheader("Chart")
    st.line_chart(filtered.set_index("ts")["value"])

    st.subheader("Deferred download (callable)")
    st.write("Use a callable to generate download data only when the user clicks.")

    def generate_csv() -> bytes:
        return _make_csv_bytes(filtered)

    def download_ui() -> None:
        st.download_button(
            label="Download filtered CSV",
            data=generate_csv,
            file_name=f"demo-{group}.csv",
            mime="text/csv",
        )

    # Streamlit docs recommend wrapping downloads in a fragment to prevent full reruns on click.
    if hasattr(st, "fragment"):

        @st.fragment
        def _download_fragment() -> None:
            download_ui()

        _download_fragment()
    else:
        download_ui()

    st.divider()
    st.subheader("Chat input (audio optional in 1.52.x)")

    try:
        user_input = st.chat_input(
            "Ask about the dataset…",
            accept_audio=True,
            audio_sample_rate=16_000,
        )
    except TypeError:
        user_input = st.chat_input("Ask about the dataset…")
    if user_input:
        user_text = user_input if isinstance(user_input, str) else str(user_input)
        with st.chat_message("user"):
            st.write(user_text)

        with st.chat_message("assistant"):
            st.write("This template does not call an external LLM. Wire your provider in your app code.")

    st.divider()
    st.subheader("Safe HTML (CSS-only) example")

    css = """
    <style>
      .sma-note { padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid rgba(127,127,127,0.35); }
      .sma-note b { font-weight: 700; }
    </style>
    <div class="sma-note"><b>Note:</b> Prefer theming/config over custom HTML. Do not execute untrusted JS.</div>
    """
    if hasattr(st, "html"):
        st.html(css)
    else:
        st.info("`st.html` is unavailable in this Streamlit version; using plain text fallback.")
        st.markdown("**Note:** Prefer theming/config over custom HTML. Do not execute untrusted JS.")


if __name__ == "__main__":
    main()
