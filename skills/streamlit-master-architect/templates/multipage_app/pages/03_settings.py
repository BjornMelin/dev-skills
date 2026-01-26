from __future__ import annotations

import streamlit as st

st.title("Settings")

st.subheader("Query params demo")
qp = st.query_params.to_dict()
st.write("Current query params:", qp)

if st.button("Set example query params"):
    st.query_params["mode"] = "demo"
    st.query_params["tab"] = "settings"
    st.rerun()

