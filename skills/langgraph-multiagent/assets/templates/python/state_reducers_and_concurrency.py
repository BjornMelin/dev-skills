from __future__ import annotations

import operator
from typing import Annotated
from typing_extensions import TypedDict

from langgraph.graph import START, StateGraph


class State(TypedDict):
    # Append-only list reducer allows parallel nodes to safely write updates.
    completed_sections: Annotated[list[str], operator.add]


def worker_a(state: State) -> dict:
    return {"completed_sections": ["a"]}


def worker_b(state: State) -> dict:
    return {"completed_sections": ["b"]}


def build_graph() -> object:
    g = StateGraph(State)
    g.add_node("a", worker_a)
    g.add_node("b", worker_b)
    g.add_edge(START, "a")
    g.add_edge(START, "b")
    return g.compile()

