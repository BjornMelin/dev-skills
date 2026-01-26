from __future__ import annotations

from graph_supervisor_subagents import build_supervisor_agent


def test_supervisor_smoke() -> None:
    agent = build_supervisor_agent()
    out = agent.invoke({"messages": [{"role": "user", "content": "Draft an email to Bob about lunch."}]})
    assert out["messages"]
