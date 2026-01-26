from __future__ import annotations

import urllib.request

from langchain.agents import create_agent
from langchain.chat_models import init_chat_model
from langchain.tools import tool


ALLOWED_PREFIXES = (
    "https://langchain-ai.github.io/langgraph/",
    "https://docs.langchain.com/oss/python/",
)


@tool
def fetch_documentation(url: str) -> str:
    """Fetch documentation from an allowlisted URL (SSRF-safe by prefix allowlist)."""
    if not any(url.startswith(p) for p in ALLOWED_PREFIXES):
        return f"Error: URL not allowed. Must start with one of: {', '.join(ALLOWED_PREFIXES)}"
    req = urllib.request.Request(url, headers={"User-Agent": "langgraph-multiagent-skill/1.0"})
    with urllib.request.urlopen(req, timeout=20) as resp:
        return resp.read().decode("utf-8", errors="replace")


def build_docs_agent() -> object:
    model = init_chat_model("gpt-4o-mini", temperature=0)
    system_prompt = (
        "You are a docs-grounded assistant for LangGraph/LangChain.\n"
        "If the question involves API details, you MUST call fetch_documentation\n"
        "on the relevant official docs URL before answering."
    )
    return create_agent(
        model=model,
        tools=[fetch_documentation],
        system_prompt=system_prompt,
    )

