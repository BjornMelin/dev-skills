from __future__ import annotations

from typing import Literal

from langchain.agents import AgentState, create_agent
from langchain.messages import AIMessage, ToolMessage
from langchain.tools import tool, ToolRuntime
from langgraph.graph import END, START, StateGraph
from langgraph.types import Command
from typing_extensions import NotRequired


class MultiAgentState(AgentState):
    active_agent: NotRequired[str]


@tool
def transfer_to_sales(runtime: ToolRuntime) -> Command:
    """Transfer to the sales agent."""
    last_ai_message = next(msg for msg in reversed(runtime.state["messages"]) if isinstance(msg, AIMessage))
    transfer_message = ToolMessage(
        content="Transferred to sales agent from support agent",
        tool_call_id=runtime.tool_call_id,
    )
    return Command(
        goto="sales_agent",
        update={"active_agent": "sales_agent", "messages": [last_ai_message, transfer_message]},
        graph=Command.PARENT,
    )


@tool
def transfer_to_support(runtime: ToolRuntime) -> Command:
    """Transfer to the support agent."""
    last_ai_message = next(msg for msg in reversed(runtime.state["messages"]) if isinstance(msg, AIMessage))
    transfer_message = ToolMessage(
        content="Transferred to support agent from sales agent",
        tool_call_id=runtime.tool_call_id,
    )
    return Command(
        goto="support_agent",
        update={"active_agent": "support_agent", "messages": [last_ai_message, transfer_message]},
        graph=Command.PARENT,
    )


sales_agent = create_agent(
    model="anthropic:claude-sonnet-4-20250514",
    tools=[transfer_to_support],
    system_prompt="You are a sales agent. If asked about support, transfer to support.",
)

support_agent = create_agent(
    model="anthropic:claude-sonnet-4-20250514",
    tools=[transfer_to_sales],
    system_prompt="You are a support agent. If asked about sales, transfer to sales.",
)


def call_sales(state: MultiAgentState) -> Command:
    return sales_agent.invoke(state)


def call_support(state: MultiAgentState) -> Command:
    return support_agent.invoke(state)


def route_after_agent(state: MultiAgentState) -> Literal["sales_agent", "support_agent", END]:
    messages = state.get("messages", [])
    if messages:
        last = messages[-1]
        if isinstance(last, AIMessage) and not last.tool_calls:
            return END
    return state.get("active_agent") or "sales_agent"


def route_initial(state: MultiAgentState) -> Literal["sales_agent", "support_agent"]:
    return state.get("active_agent") or "sales_agent"


def build_graph() -> object:
    builder = StateGraph(MultiAgentState)
    builder.add_node("sales_agent", call_sales)
    builder.add_node("support_agent", call_support)
    builder.add_conditional_edges(START, route_initial, ["sales_agent", "support_agent"])
    builder.add_conditional_edges("sales_agent", route_after_agent, ["sales_agent", "support_agent", END])
    builder.add_conditional_edges("support_agent", route_after_agent, ["sales_agent", "support_agent", END])
    return builder.compile()

