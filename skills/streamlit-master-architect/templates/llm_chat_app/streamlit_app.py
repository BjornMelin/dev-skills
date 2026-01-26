from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Generator, Literal, TypedDict

import streamlit as st


Role = Literal["user", "assistant"]


class ChatMessage(TypedDict):
    role: Role
    content: str


@dataclass(frozen=True)
class ChatConfig:
    title: str = "SMA — Chat App Skeleton"
    icon: str = "💬"


def _set_page() -> None:
    st.set_page_config(page_title=ChatConfig.title, page_icon=ChatConfig.icon, layout="wide")


def _ss_init() -> None:
    if "messages" not in st.session_state:
        st.session_state["messages"] = []


def _fake_streaming_llm(prompt: str) -> Generator[str, None, None]:
    for tok in (prompt.upper().split()[:50] or ["(empty)"]):
        yield tok + " "
        time.sleep(0.03)


def main() -> None:
    _set_page()
    _ss_init()

    st.title("Chat App Skeleton (streaming-ready)")
    st.caption("Wire your LLM provider in a dedicated module; keep rendering separate from logic.")

    for msg in st.session_state["messages"]:
        with st.chat_message(msg["role"]):
            st.write(msg["content"])

    try:
        user_input = st.chat_input(
            "Send a message (audio optional)…",
            accept_audio=True,
            audio_sample_rate=16_000,
        )
    except TypeError:
        user_input = st.chat_input("Send a message…")

    if user_input:
        user_text = user_input if isinstance(user_input, str) else str(user_input)
        st.session_state["messages"].append({"role": "user", "content": user_text})
        with st.chat_message("user"):
            st.write(user_text)

        with st.chat_message("assistant"):
            streamed = st.write_stream(_fake_streaming_llm(user_text))

        st.session_state["messages"].append({"role": "assistant", "content": str(streamed)})


if __name__ == "__main__":
    main()
