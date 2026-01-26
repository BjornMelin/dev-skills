from __future__ import annotations

from langchain.agents import create_agent
from langchain.agents.middleware import HumanInTheLoopMiddleware, PIIMiddleware, SummarizationMiddleware
from langchain.chat_models import init_chat_model
from langchain.tools import tool
from langgraph.checkpoint.memory import InMemorySaver


@tool
def read_data(query: str) -> str:
    return f"[data] {query}"


@tool
def send_email(to: str, subject: str, body: str) -> str:
    return f"Sent email to {to} with subject {subject}"


def build_agent() -> object:
    model = init_chat_model("gpt-4o-mini", temperature=0)

    agent = create_agent(
        model=model,
        tools=[read_data, send_email],
        system_prompt="You are a helpful assistant. Use tools when appropriate.",
        middleware=[
            # Redact emails in user input before sending to model
            PIIMiddleware("email", strategy="redact", apply_to_input=True),
            # Keep long conversations bounded
            SummarizationMiddleware(model=model, trigger={"tokens": 1200}),
            # Require approval for side effects
            HumanInTheLoopMiddleware(interrupt_on={"send_email": True}),
        ],
        # Required for interrupts / HITL
        checkpointer=InMemorySaver(),
    )
    return agent

