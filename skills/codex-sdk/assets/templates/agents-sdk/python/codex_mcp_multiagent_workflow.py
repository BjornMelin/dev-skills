import asyncio
import os

from agents import Agent, Runner
from agents.mcp import MCPServerStdio
from agents.tracing import trace


"""
Multi-agent workflow skeleton:

- Run Codex CLI as an MCP server (stdio)
- Orchestrate specialized agents via handoffs
- Enforce gated handoffs (file existence / checks)
- Keep it traceable

This is intentionally minimal: adapt scopes, gates, and deliverables to your project.
"""


async def main() -> None:
    repo_root = os.getcwd()

    async with MCPServerStdio(
        name="Codex CLI",
        params={"command": "npx", "args": ["-y", "codex", "mcp-server"]},
        # Long workflows can take time; keep the MCP client session alive.
        client_session_timeout_seconds=3600,
    ) as codex_server:
        # Workers: keep scopes narrow. They should call the MCP tool `codex`/`codex-reply`
        # with explicit safety args when they need to read/write.
        designer = Agent(
            name="Designer",
            instructions=(
                "Write a short design spec. Save it as design/design_spec.md.\n"
                "When creating files, call Codex MCP with "
                '{"approval-policy":"never","sandbox":"workspace-write","cwd":"."}.\n'
                "When done, hand off back to the Project Manager."
            ),
            mcp_servers=[codex_server],
        )

        developer = Agent(
            name="Developer",
            instructions=(
                "Implement only what the design spec requires.\n"
                "When creating files, call Codex MCP with "
                '{"approval-policy":"never","sandbox":"workspace-write","cwd":"."}.\n'
                "When done, hand off back to the Project Manager."
            ),
            mcp_servers=[codex_server],
        )

        tester = Agent(
            name="Tester",
            instructions=(
                "Write a minimal test plan and run the checks the PM requests.\n"
                "When creating files, call Codex MCP with "
                '{"approval-policy":"never","sandbox":"workspace-write","cwd":"."}.\n'
                "When done, hand off back to the Project Manager."
            ),
            mcp_servers=[codex_server],
        )

        pm = Agent(
            name="Project Manager",
            instructions=(
                "You are the orchestrator.\n"
                "Create an ExecPlan in execplans/ and keep it updated.\n"
                "Do not advance to the next stage until gates are satisfied.\n"
                "\n"
                "Gates (example):\n"
                "- After Designer: design/design_spec.md exists.\n"
                "- After Developer: expected code artifacts exist.\n"
                "- After Tester: test plan exists and checks pass.\n"
                "\n"
                "Handoff order:\n"
                "1) Designer\n"
                "2) Developer\n"
                "3) Tester\n"
                "\n"
                "If a gate fails, ask the owning agent to fix only that failure.\n"
            ),
            handoffs=[designer, developer, tester],
            mcp_servers=[codex_server],
        )

        with trace("codex-mcp-multiagent", group_id="example-run"):
            await Runner.run(
                pm,
                f"Repo root is {repo_root}. Build a tiny demo feature and validate it. Keep everything small.",
                max_turns=30,
            )


if __name__ == "__main__":
    asyncio.run(main())

