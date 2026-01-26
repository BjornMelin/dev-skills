from __future__ import annotations

import asyncio
import json
import uuid
from dataclasses import asdict, is_dataclass
from typing import Any, AsyncIterator, Literal

from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field

# NOTE:
# - This template demonstrates in-process LangGraph/LangChain streaming behind FastAPI.
# - For most production UIs, prefer LangGraph Agent Server + useStream() (see references/ui_nextjs_rsc.md).


class ChatMessage(BaseModel):
    role: str
    content: str


class ChatStreamRequest(BaseModel):
    thread_id: str | None = Field(default=None, description="Thread ID for persistence.")
    messages: list[ChatMessage] | None = Field(
        default=None, description="New messages to append (normal run)."
    )
    resume: dict[str, Any] | None = Field(
        default=None,
        description=(
            "Resume payload for Command(resume=...). Must match the interrupt schema you received."
        ),
    )
    stream_modes: list[Literal["messages", "updates", "custom"]] = Field(
        default_factory=lambda: ["messages", "updates"],
        description="Which LangGraph stream modes to consume.",
    )


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


def _sse(event: dict[str, Any], *, event_id: int | None = None, name: str | None = None) -> str:
    lines: list[str] = []
    if event_id is not None:
        lines.append(f"id: {event_id}")
    if name is not None:
        lines.append(f"event: {name}")
    lines.append(
        "data: " + json.dumps(event, separators=(",", ":"), default=str)  # type: ignore[arg-type]
    )
    lines.append("")
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Agent wiring (replace with your real multi-agent runtime)
# ---------------------------------------------------------------------------

def build_agent() -> object:
    """
    Replace this with your real agent/graph import.

    For example:
      from src.agent import agent
      return agent
    """
    from langchain.agents import create_agent
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
        checkpointer=InMemorySaver(),
    )


AGENT = build_agent()

app = FastAPI(title="LangGraph SSE (in-process)")


@app.post("/chat/stream")
async def chat_stream(req: ChatStreamRequest) -> StreamingResponse:
    thread_id = req.thread_id or str(uuid.uuid4())

    async def gen() -> AsyncIterator[str]:
        event_id = 0
        yield _sse({"type": "meta", "threadId": thread_id}, event_id=event_id, name="meta")
        event_id += 1

        config = {"configurable": {"thread_id": thread_id}}
        stream_modes = req.stream_modes

        # If resuming after HITL, pass a Command(resume=...) object into the agent.
        if req.resume is not None:
            from langgraph.types import Command

            agent_input: Any = Command(resume=req.resume)
        else:
            if not req.messages:
                yield _sse(
                    {"type": "error", "message": "messages is required when resume is not provided"},
                    event_id=event_id,
                    name="error",
                )
                return
            agent_input = {
                "messages": [m.model_dump() for m in req.messages],
            }

        try:
            # Multi-mode streaming yields (mode, chunk) tuples.
            async for mode, chunk in AGENT.astream(  # type: ignore[attr-defined]
                agent_input,
                config=config,
                stream_mode=stream_modes,
            ):
                if mode == "messages":
                    token, metadata = chunk
                    content = getattr(token, "content", None)
                    if content:
                        meta_json = _jsonable(metadata)
                        yield _sse(
                            {
                                "type": "token",
                                "content": content,
                                "metadata": meta_json,
                                "node": meta_json.get("langgraph_node") if isinstance(meta_json, dict) else None,
                            },
                            event_id=event_id,
                            name="token",
                        )
                        event_id += 1
                elif mode == "updates":
                    data = _jsonable(chunk)
                    yield _sse({"type": "update", "data": data}, event_id=event_id, name="update")
                    event_id += 1

                    # Interrupts often appear under a __interrupt__ key during updates.
                    if isinstance(data, dict) and "__interrupt__" in data:
                        yield _sse(
                            {"type": "interrupt", "interrupt": data["__interrupt__"]},
                            event_id=event_id,
                            name="interrupt",
                        )
                        event_id += 1
                elif mode == "custom":
                    yield _sse(
                        {"type": "custom", "event": _jsonable(chunk)},
                        event_id=event_id,
                        name="custom",
                    )
                    event_id += 1
                else:
                    yield _sse(
                        {"type": "custom", "event": {"mode": mode, "chunk": _jsonable(chunk)}},
                        event_id=event_id,
                        name="custom",
                    )
                    event_id += 1

                await asyncio.sleep(0)
        except Exception as e:  # noqa: BLE001
            yield _sse({"type": "error", "message": str(e)}, event_id=event_id, name="error")
            return

        yield _sse({"type": "done"}, event_id=event_id, name="done")

    headers = {
        "Cache-Control": "no-cache",
        "Connection": "keep-alive",
    }
    return StreamingResponse(gen(), media_type="text/event-stream", headers=headers)


# Run with:
#   uvicorn fastapi_sse_multiagent:app --reload --port 8000
