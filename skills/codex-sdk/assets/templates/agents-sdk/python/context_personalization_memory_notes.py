import asyncio
import datetime as dt
import json
import re
from dataclasses import dataclass, field

from agents import Agent, RunContextWrapper, Runner, SQLiteSession, function_tool


"""
Context personalization skeleton (memory notes):

- Keep a structured state object in code (source of truth)
- Capture candidate memories via a dedicated tool (distillation)
- Consolidate session notes into durable notes (dedupe + "latest wins")
- Inject only the relevant slice into agent instructions for the next run

This avoids relying on the raw transcript as your only memory store.
"""


@dataclass
class MemoryNote:
    type: str  # "preference" | "constraint" | "fact"
    key: str
    value: object
    confidence: float
    source: str  # prefer "user"
    created_at: str


@dataclass
class UserState:
    profile: dict[str, object] = field(default_factory=dict)
    global_notes: list[MemoryNote] = field(default_factory=list)
    session_notes: list[MemoryNote] = field(default_factory=list)


def now_iso() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


def looks_sensitive(text: str) -> bool:
    # Minimal guardrail examples (extend for your domain).
    if re.search(r"\b\d{3}-\d{2}-\d{4}\b", text):  # SSN-like
        return True
    if re.search(r"\b(?:api[_-]?key|secret|password)\b", text, re.I):
        return True
    return False


@function_tool
async def save_memory_note(wrapper: RunContextWrapper[UserState], note_json: str) -> str:
    """
    Save a candidate memory note as JSON.
    Only store durable preferences/constraints; never store secrets.
    """
    try:
        obj = json.loads(note_json)
    except json.JSONDecodeError:
        return "Rejected: invalid JSON."

    note_type = str(obj.get("type") or "")
    key = str(obj.get("key") or "")
    value = obj.get("value")
    confidence = float(obj.get("confidence") or 0.5)
    source = str(obj.get("source") or "user")

    if note_type not in {"preference", "constraint", "fact"}:
        return "Rejected: invalid type."
    if not key:
        return "Rejected: missing key."

    serialized = json.dumps({"key": key, "value": value}, ensure_ascii=False)
    if looks_sensitive(serialized):
        return "Rejected: looks sensitive."

    wrapper.context.session_notes.append(
        MemoryNote(
            type=note_type,
            key=key,
            value=value,
            confidence=max(0.0, min(1.0, confidence)),
            source=source,
            created_at=now_iso(),
        )
    )
    return "Saved."


def inject_memory(state: UserState, max_notes: int = 8) -> str:
    # Keep injection small. Prefer top-K by recency and/or relevance (add relevance ranking as needed).
    notes = state.global_notes[-max_notes:]
    lines = [
        "You are a helpful assistant.",
        "",
        "Personalization context (advisory):",
        "<profile>",
        json.dumps(state.profile, ensure_ascii=False),
        "</profile>",
        "<memories>",
    ]
    for n in notes:
        lines.append(f"- ({n.type}) {n.key}: {json.dumps(n.value, ensure_ascii=False)}")
    lines += ["</memories>", "", "Precedence: current user input > session context > memories."]
    return "\n".join(lines)


def consolidate(state: UserState) -> None:
    # Simple “latest wins” per key.
    by_key: dict[str, MemoryNote] = {n.key: n for n in state.global_notes}
    for note in state.session_notes:
        by_key[note.key] = note
    state.global_notes = sorted(by_key.values(), key=lambda n: n.created_at)
    state.session_notes = []


async def main() -> None:
    state = UserState(profile={"locale": "en-US"})

    session = SQLiteSession("user-123", "agent_history.sqlite")

    # Run 1
    agent = Agent[UserState](
        name="Concierge",
        instructions=inject_memory(state),
        tools=[save_memory_note],
    )
    await Runner.run(agent, "I prefer vegetarian meals. Please remember that.", context=state, session=session)
    consolidate(state)

    # Run 2 (memory should be injected)
    agent2 = Agent[UserState](
        name="Concierge",
        instructions=inject_memory(state),
        tools=[save_memory_note],
    )
    result = await Runner.run(agent2, "Plan a dinner recommendation.", context=state, session=session)
    print(result.final_output)


if __name__ == "__main__":
    asyncio.run(main())

