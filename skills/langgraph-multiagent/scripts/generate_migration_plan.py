#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


@dataclass(frozen=True)
class FindingRule:
    key: str
    title: str
    migration_focus: str
    docs_queries: tuple[str, ...]


RULES: tuple[FindingRule, ...] = (
    FindingRule(
        key="deprecated-create-react-agent",
        title="Migrate `create_react_agent` → `create_agent`",
        migration_focus="Replace deprecated LangGraph prebuilt agent usage with LangChain v1 `create_agent` + middleware equivalents.",
        docs_queries=(
            "migrate langchain v1 create_agent",
            "migrate langgraph v1 create_react_agent",
            "decorator-based middleware before_model after_model wrap_tool_call",
        ),
    ),
    FindingRule(
        key="langgraph-supervisor",
        title="Migrate `langgraph-supervisor(-py)` → supervisor/subagents via tools",
        migration_focus="Adopt the LangChain multi-agent subagents pattern (supervisor calls subagents as tools) unless you have a strong reason to keep the library.",
        docs_queries=(
            "subagents-personal-assistant",
            "langchain multi-agent subagents",
            "langchain multi-agent handoffs",
        ),
    ),
    FindingRule(
        key="llamaindex-agents",
        title="Port LlamaIndex agent orchestration",
        migration_focus="Extract tools, then port orchestration to LangChain/LangGraph. Keep LlamaIndex only for retrieval/indexing pieces that are uniquely valuable.",
        docs_queries=("langgraph workflows agents", "langchain tools", "langgraph agentic rag"),
    ),
    FindingRule(
        key="crewai",
        title="Port CrewAI crews/tasks to LangGraph",
        migration_focus="Map roles→subagents, tasks→tool calls or graph nodes, manager→supervisor. Preserve behavior with tests.",
        docs_queries=("langchain multi-agent subagents", "langgraph workflows agents"),
    ),
    FindingRule(
        key="agno",
        title="Port Agno orchestration to LangGraph",
        migration_focus="Rebuild orchestration explicitly as a graph; keep tools and prompts, add tests, then harden with middleware.",
        docs_queries=("thinking in langgraph", "langgraph graph api", "langchain middleware"),
    ),
    FindingRule(
        key="openai-agents",
        title="Port provider-specific Agents stack to LangChain/LangGraph",
        migration_focus="Extract tools into LangChain tools, rebuild orchestration with create_agent/StateGraph, and add provider-agnostic tracing + guardrails.",
        docs_queries=("langchain agents", "langchain guardrails", "langchain mcp"),
    ),
)


def _load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _rule_for(key: str) -> FindingRule | None:
    for r in RULES:
        if r.key == key:
            return r
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate a migration plan skeleton from audit_repo_agents.py JSON output.")
    parser.add_argument("--audit-json", required=True, help="Path to JSON output from audit_repo_agents.py")
    parser.add_argument("--out", default="migration_plan.md", help="Output markdown file (default: migration_plan.md)")
    args = parser.parse_args()

    audit_path = Path(args.audit_json).resolve()
    out_path = Path(args.out).resolve()
    payload = _load_json(audit_path)

    now = datetime.now(timezone.utc).isoformat()
    root = payload.get("root", "")
    versions = payload.get("versions", {}) or {}
    matches = payload.get("matches", {}) or {}

    detected = [k for k, v in matches.items() if v]

    lines: list[str] = []
    lines.append("# Migration plan (draft)")
    lines.append("")
    lines.append(f"- Generated: `{now}`")
    lines.append(f"- Audit: `{audit_path}`")
    if root:
        lines.append(f"- Root: `{root}`")
    lines.append("")

    lines.append("## Version baseline")
    lines.append("")
    for k in sorted(versions.keys()):
        v = versions.get(k)
        if v:
            lines.append(f"- `{k}=={v}`")
    lines.append("")

    lines.append("## Goals")
    lines.append("")
    lines.append("- Consolidate agent orchestration on LangChain v1 + LangGraph v1+ patterns.")
    lines.append("- Remove deprecated APIs and reduce framework fragmentation.")
    lines.append("- Add tests + observability to prevent regressions during upgrades.")
    lines.append("")

    lines.append("## Work plan (sliced)")
    lines.append("")
    lines.append("### Slice 1 — Stabilize behavior with tests")
    lines.append("- Add unit tests for side-effectful tools (schemas, idempotency).")
    lines.append("- Add integration tests for key agent flows (tool ordering, state transitions).")
    lines.append("")

    lines.append("### Slice 2 — Extract tools and boundaries")
    lines.append("- Standardize tool schemas and error handling.")
    lines.append("- Introduce guardrails/HITL for high-stakes tools.")
    lines.append("")

    lines.append("### Slice 3 — Migrate orchestration")
    lines.append("- Replace agent framework wiring with LangChain `create_agent` or LangGraph `StateGraph` where needed.")
    lines.append("- Preserve external behavior; change internals incrementally.")
    lines.append("")

    lines.append("### Slice 4 — Memory + durability")
    lines.append("- Add checkpointer/thread IDs for durability + HITL.")
    lines.append("- Add long-term store for durable preferences/facts (namespaced per tenant).")
    lines.append("")

    lines.append("### Slice 5 — Observability + evaluation")
    lines.append("- Add tracing and per-tool latency metrics.")
    lines.append("- Add regression evaluation runs (golden prompts/traces).")
    lines.append("")

    lines.append("## Findings-specific tasks")
    lines.append("")
    if not detected:
        lines.append("- No findings detected in the audit JSON.")
    else:
        for key in detected:
            rule = _rule_for(key)
            hits = matches.get(key, [])
            title = rule.title if rule else key
            lines.append(f"### {title}")
            lines.append("")
            if rule:
                lines.append(f"- Focus: {rule.migration_focus}")
                lines.append("- Docs research queries (use langchain-docs.SearchDocsByLangChain):")
                for q in rule.docs_queries:
                    lines.append(f"  - `{q}`")
            else:
                lines.append("- Focus: (no rule for this finding key; add one to scripts/generate_migration_plan.py)")
            lines.append("")
            lines.append("Hit list:")
            for h in hits[:50]:
                lines.append(f"- `{h}`")
            if len(hits) > 50:
                lines.append("- `[truncated]`")
            lines.append("")

    out_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"Wrote: {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

