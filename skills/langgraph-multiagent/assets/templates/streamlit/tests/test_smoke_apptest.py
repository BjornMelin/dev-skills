import os

from streamlit.testing.v1 import AppTest


def test_streamlit_app_loads_smoke() -> None:
    # Keep tests offline; the template supports a stub runtime in test mode.
    os.environ["LANGGRAPH_UI_TEST_MODE"] = "1"

    # NOTE: Adjust the path if you rename/move the Streamlit entrypoint.
    AppTest.from_file("streamlit_chat_app.py").run()

