from __future__ import annotations

from dataclasses import dataclass

from langchain.tools import ToolRuntime, tool


@dataclass(frozen=True)
class RuntimeContext:
    user_id: str


@tool
def example_tool(runtime: ToolRuntime[RuntimeContext]) -> str:
    # Use runtime.context for DI (user IDs, db clients, etc).
    return f"hello {runtime.context.user_id}"

