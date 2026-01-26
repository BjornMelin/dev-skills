from __future__ import annotations

from typing import Literal, TypedDict

from langchain.chat_models import init_chat_model
from langgraph.graph import END, START, MessagesState, StateGraph


class State(MessagesState):
    # Keep additional aggregation state here (lists/dicts) and ensure reducers if parallelizing.
    pass


def build_graph() -> object:
    model = init_chat_model("gpt-4o-mini", temperature=0)

    def plan(state: State) -> dict:
        # Replace with planning logic; keep it small and structured.
        response = model.invoke(state["messages"])
        return {"messages": [response]}

    def should_continue(state: State) -> Literal[END, "plan"]:
        last = state["messages"][-1]
        return END if not getattr(last, "tool_calls", None) else "plan"

    builder = StateGraph(State)
    builder.add_node("plan", plan)
    builder.add_edge(START, "plan")
    builder.add_conditional_edges("plan", should_continue)
    return builder.compile()

