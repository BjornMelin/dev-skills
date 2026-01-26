from __future__ import annotations

from pathlib import Path

from streamlit.testing.v1 import AppTest


def test_app_loads() -> None:
    app = Path(__file__).resolve().parents[1] / "streamlit_app.py"
    at = AppTest.from_file(str(app)).run()
    assert any("Streamlit patterns demo" in h.value for h in at.title), "Expected page title not found."
