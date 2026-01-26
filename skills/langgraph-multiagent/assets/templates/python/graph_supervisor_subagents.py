from __future__ import annotations

from typing import TypedDict

from langchain.agents import create_agent
from langchain.chat_models import init_chat_model
from langchain.tools import tool
from langgraph.checkpoint.memory import InMemorySaver


class Output(TypedDict):
    text: str


@tool
def calendar_tool(request: str) -> str:
    """Placeholder tool: replace with a real calendar integration."""
    return f"[calendar] {request}"


@tool
def email_tool(request: str) -> str:
    """Placeholder tool: replace with a real email integration."""
    return f"[email] {request}"


def build_supervisor_agent() -> object:
    model = init_chat_model("gpt-4o-mini", temperature=0)

    calendar_agent = create_agent(
        model,
        tools=[calendar_tool],
        system_prompt="You are a calendar assistant. Be precise.",
    )

    email_agent = create_agent(
        model,
        tools=[email_tool],
        system_prompt="You are an email assistant. Be concise and professional.",
    )

    @tool
    def schedule_event(request: str) -> str:
        result = calendar_agent.invoke({"messages": [{"role": "user", "content": request}]})
        return result["messages"][-1].text

    @tool
    def manage_email(request: str) -> str:
        result = email_agent.invoke({"messages": [{"role": "user", "content": request}]})
        return result["messages"][-1].text

    supervisor = create_agent(
        model,
        tools=[schedule_event, manage_email],
        system_prompt=(
            "You are a supervisor. Delegate work to tools. "
            "Use multiple tools in sequence when needed."
        ),
        checkpointer=InMemorySaver(),
    )
    return supervisor

