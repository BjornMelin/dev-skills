from __future__ import annotations

import datetime as dt

import pandas as pd
import streamlit as st


@st.cache_data(show_spinner="Loading report data…")
def _data() -> pd.DataFrame:
    now = dt.datetime.now()
    return pd.DataFrame(
        {
            "ts": pd.date_range(now - dt.timedelta(days=7), periods=7 * 24, freq="h"),
            "value": pd.Series(range(7 * 24)).astype(float),
        }
    )


st.title("Reports")
df = _data()
st.line_chart(df.set_index("ts")["value"])

