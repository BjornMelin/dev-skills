from __future__ import annotations

from dataclasses import dataclass

from langchain.agents import create_agent
from langchain.messages import ToolMessage
from langchain_mcp_adapters.client import MultiServerMCPClient
from langchain_mcp_adapters.interceptors import MCPToolCallRequest


@dataclass(frozen=True)
class Context:
    user_id: str
    api_key: str


async def require_authentication(request: MCPToolCallRequest, handler):
    runtime = request.runtime
    if not runtime.state.get("authenticated", False) and request.name in {"delete_file", "export_data"}:
        return ToolMessage(
            content="Authentication required.",
            tool_call_id=runtime.tool_call_id,
        )
    return await handler(request)


async def inject_user_context(request: MCPToolCallRequest, handler):
    runtime = request.runtime
    modified = request.override(args={**request.args, "user_id": runtime.context.user_id})
    return await handler(modified)


async def build_agent() -> object:
    client = MultiServerMCPClient(
        {
            "internal": {"url": "http://localhost:8000/mcp"},
        },
        tool_interceptors=[require_authentication, inject_user_context],
    )
    tools = await client.get_tools()
    return create_agent(
        model="gpt-4o-mini",
        tools=tools,
        context_schema=Context,
    )

