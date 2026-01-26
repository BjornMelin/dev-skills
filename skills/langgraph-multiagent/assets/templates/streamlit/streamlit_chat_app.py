from __future__ import annotations

import json
import os
import uuid
from dataclasses import asdict, is_dataclass
from typing import Any

import streamlit as st


def _jsonable(value: Any) -> Any:
    if is_dataclass(value):
        return asdict(value)
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    if isinstance(value, dict):
        return {str(k): _jsonable(v) for k, v in value.items()}
    if isinstance(value, list):
        return [_jsonable(v) for v in value]
    if isinstance(value, tuple):
        return [_jsonable(v) for v in value]
    return str(value)


@st.cache_resource
def build_agent() -> object:
    """
    Replace with your real multi-agent system.

    For example:
      from src.agent import agent
      return agent
    """
    if os.environ.get("LANGGRAPH_UI_TEST_MODE") == "1":
        class _Token:
            def __init__(self, content: str) -> None:
                self.content = content

        class _StubAgent:
            def stream(self, agent_input: Any, *_args: Any, **_kwargs: Any):  # type: ignore[no-untyped-def]
                user_text = ""
                if isinstance(agent_input, dict):
                    messages = agent_input.get("messages") or []
                    if messages:
                        last = messages[-1]
                        if isinstance(last, dict):
                            user_text = str(last.get("content") or "")

                response = f"Stub response (set LANGGRAPH_UI_TEST_MODE=0 for real LLM): {user_text}"
                for token in response.split(" "):
                    yield ("messages", (_Token(token + " "), {}))

        return _StubAgent()

    from langchain.agents import create_agent
    from langchain.agents.middleware import HumanInTheLoopMiddleware
    from langchain.chat_models import init_chat_model
    from langchain.tools import tool
    from langgraph.checkpoint.memory import InMemorySaver

    @tool
    def search(query: str) -> str:
        return f"[search results] {query}"

    model = init_chat_model("gpt-4o-mini", temperature=0)
    return create_agent(
        model=model,
        tools=[search],
        system_prompt="You are a helpful assistant. Use tools when useful.",
        middleware=[
            # Example: require approvals for a specific tool (update to match your tools).
            HumanInTheLoopMiddleware(
                interrupt_on={"search": True},
                description_prefix="Tool execution pending approval",
            ),
        ],
        checkpointer=InMemorySaver(),
    )


AGENT = build_agent()

st.set_page_config(page_title="LangGraph Multi-Agent (Streamlit)", layout="wide")


def init_state() -> None:
    st.session_state.setdefault("thread_id", str(uuid.uuid4()))
    st.session_state.setdefault("messages", [])
    st.session_state.setdefault("pending_interrupts", None)
    st.session_state.setdefault("reject_reason", "User rejected")


def reset_thread() -> None:
    st.session_state.thread_id = str(uuid.uuid4())
    st.session_state.messages = []
    st.session_state.pending_interrupts = None


def extract_action_requests(interrupt_value: Any) -> list[dict[str, Any]]:
    if isinstance(interrupt_value, dict):
        if "action_requests" in interrupt_value:
            return interrupt_value.get("action_requests") or []
        if "actionRequests" in interrupt_value:
            return interrupt_value.get("actionRequests") or []
    return []


def stream_run(agent_input: Any) -> None:
    config = {"configurable": {"thread_id": st.session_state.thread_id}}

    assistant_container = st.chat_message("assistant")
    text_placeholder = assistant_container.empty()
    token_buffer = ""
    interrupts: Any = None

    for mode, chunk in AGENT.stream(  # type: ignore[attr-defined]
        agent_input,
        config=config,
        stream_mode=["messages", "updates"],
    ):
        if mode == "messages":
            token, _meta = chunk
            content = getattr(token, "content", None)
            if content:
                token_buffer += content
                text_placeholder.markdown(token_buffer)
        elif mode == "updates":
            data = _jsonable(chunk)
            if isinstance(data, dict) and "__interrupt__" in data:
                interrupts = data["__interrupt__"]

    if token_buffer.strip():
        st.session_state.messages.append({"role": "assistant", "content": token_buffer})

    st.session_state.pending_interrupts = interrupts


def resume_from_interrupts(decisions: dict[str, Any]) -> None:
    from langgraph.types import Command

    stream_run(Command(resume=decisions))


init_state()

st.sidebar.header("Session")
st.sidebar.caption(f"Streamlit {st.__version__}")
st.sidebar.caption("Thread IDs are required for persistence, interrupts, and resume.")
st.sidebar.code(st.session_state.thread_id, language="text")

if st.sidebar.button("New thread", use_container_width=True):
    reset_thread()
    st.rerun()

st.title("LangGraph Multi-Agent (Streamlit)")
st.caption("Streams tokens + handles interrupts (HITL) using thread-scoped persistence.")

for msg in st.session_state.messages:
    with st.chat_message(msg["role"]):
        st.markdown(msg["content"])

user_text = st.chat_input("Ask something…")
if user_text:
    st.session_state.messages.append({"role": "user", "content": user_text})
    with st.chat_message("user"):
        st.markdown(user_text)
    stream_run({"messages": [{"role": "user", "content": user_text}]})

pending = st.session_state.pending_interrupts
if pending:
    st.divider()
    st.subheader("Human approval required")

    # In Python, interrupts are commonly a list of Interrupt objects with {id, value}.
    pending_json = _jsonable(pending)
    st.caption("Review each tool call. Decisions must match the action order.")

    reject_reason = st.text_input(
        "Reject reason (used for any reject decisions)",
        value=st.session_state.reject_reason,
        key="hitl_reject_reason",
    )
    st.session_state.reject_reason = reject_reason

    interrupts_list = pending_json if isinstance(pending_json, list) else [pending_json]
    for item in interrupts_list:
        interrupt_id = (item.get("id") if isinstance(item, dict) else None) or str(uuid.uuid4())
        value = item.get("value") if isinstance(item, dict) else item
        action_requests = extract_action_requests(value)

        st.markdown(f"**Interrupt** `{interrupt_id}`")
        st.code(json.dumps(_jsonable(value), indent=2), language="json")

        for idx, action in enumerate(action_requests):
            name = action.get("name") if isinstance(action, dict) else "tool_call"
            args = {}
            if isinstance(action, dict):
                args = action.get("args") or action.get("arguments") or {}

            st.markdown(f"**{name}**")

            decision_key = f"hitl_decision::{interrupt_id}::{idx}"
            mode = st.selectbox(
                "Decision",
                options=["approve", "reject", "edit"],
                index=0,
                key=decision_key,
            )

            if mode == "edit":
                st.text_area(
                    "Edited args (JSON)",
                    value=json.dumps(args, indent=2),
                    key=f"hitl_edit_args::{interrupt_id}::{idx}",
                    height=120,
                )
            else:
                st.code(json.dumps(args, indent=2), language="json")

            st.divider()

    if st.button("Resume", type="primary"):
        resume: dict[str, Any] = {}
        errors: list[str] = []

        for item in interrupts_list:
            interrupt_id = (item.get("id") if isinstance(item, dict) else None) or str(uuid.uuid4())
            value = item.get("value") if isinstance(item, dict) else item
            action_requests = extract_action_requests(value)

            decisions: list[dict[str, Any]] = []
            for idx, action in enumerate(action_requests):
                name = action.get("name") if isinstance(action, dict) else "tool_call"

                mode = st.session_state.get(f"hitl_decision::{interrupt_id}::{idx}", "approve")
                if mode == "approve":
                    decisions.append({"type": "approve"})
                elif mode == "reject":
                    decisions.append({"type": "reject", "message": st.session_state.reject_reason})
                else:
                    raw = st.session_state.get(f"hitl_edit_args::{interrupt_id}::{idx}", "{}")
                    try:
                        edited_args = json.loads(raw)
                    except Exception:
                        errors.append(f"Invalid JSON for edited args: interrupt={interrupt_id} action={idx}")
                        edited_args = {}
                    decisions.append(
                        {
                            "type": "edit",
                            # Python uses snake_case for decision payload keys.
                            "edited_action": {"name": name, "args": edited_args},
                        }
                    )

            resume[interrupt_id] = {"decisions": decisions}

        if errors:
            st.error("\n".join(errors))
        else:
            st.session_state.pending_interrupts = None
            resume_from_interrupts(resume)
            st.rerun()
