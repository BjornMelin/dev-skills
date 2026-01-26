from __future__ import annotations

import argparse
import base64
import json
import os
import shlex
import socket
import subprocess
import time
import urllib.request
from pathlib import Path
from typing import Any, Mapping

from mcp_stdio_client import MCPStdioClient, MCPTool


def _find_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def _wait_http_ok(url: str, timeout_s: float = 30.0) -> None:
    deadline = time.time() + timeout_s
    last_err: Exception | None = None
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=2) as resp:
                if 200 <= resp.status < 500:
                    return
        except Exception as e:
            last_err = e
        time.sleep(0.25)
    raise TimeoutError(f"App did not become reachable at {url}. Last error: {last_err}")


def _start_streamlit(app_path: Path, *, port: int, streamlit_cmd: str) -> subprocess.Popen[str]:
    cmd = shlex.split(streamlit_cmd) + [
        "run",
        str(app_path),
        "--server.headless=true",
        f"--server.port={port}",
        "--server.address=127.0.0.1",
    ]
    return subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        env=os.environ.copy(),
    )


def _stop_process(proc: subprocess.Popen[Any], name: str) -> None:
    if proc.poll() is None:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
    try:
        if proc.stdout is not None:
            out = proc.stdout.read()
            if out:
                print(f"\n[{name} output]\n{out}\n")
    except Exception:
        pass


def _tool_props(tool: MCPTool) -> list[str]:
    schema = tool.input_schema or {}
    props = schema.get("properties", {}) if isinstance(schema, dict) else {}
    if isinstance(props, dict):
        return [str(k) for k in props.keys()]
    return []


def _tool_required(tool: MCPTool) -> list[str]:
    schema = tool.input_schema or {}
    required = schema.get("required", []) if isinstance(schema, dict) else []
    if isinstance(required, list):
        return [str(k) for k in required]
    return []


def _pick_tool(tools: list[MCPTool], keywords: list[str]) -> MCPTool | None:
    for kw in keywords:
        for t in tools:
            if kw.lower() in t.name.lower():
                return t
    return None


def _best_effort_args(tool: MCPTool, *, url: str, screenshot_path: Path) -> Mapping[str, Any]:
    props = [p.lower() for p in _tool_props(tool)]
    args: dict[str, Any] = {}

    if "url" in props:
        args["url"] = url
    elif "href" in props:
        args["href"] = url
    elif "target" in props:
        args["target"] = url

    if "path" in props:
        args["path"] = str(screenshot_path)
    if "filename" in props:
        args["filename"] = str(screenshot_path)
    if "fullpage" in props:
        args["fullPage"] = True
    if "full_page" in props:
        args["full_page"] = True

    # Common "console messages" flags
    if "errorsonly" in props:
        args["errorsOnly"] = True
    if "errors_only" in props:
        args["errors_only"] = True

    return args


def main() -> int:
    parser = argparse.ArgumentParser(description="Run Streamlit E2E via Playwright MCP server (stdio).")
    parser.add_argument("--app", type=str, required=True, help="Path to Streamlit app entrypoint (.py).")
    parser.add_argument("--artifacts", type=str, default="artifacts", help="Artifacts output directory.")
    parser.add_argument(
        "--playwright-mcp-cmd",
        type=str,
        default="npx -y @playwright/mcp@latest",
        help="Command to start Playwright MCP server (stdio).",
    )
    parser.add_argument("--streamlit-cmd", type=str, default="streamlit", help="Streamlit CLI command.")
    parser.add_argument("--headless", action="store_true", help="Pass --headless to Playwright MCP server if supported.")
    args = parser.parse_args()

    app_path = Path(args.app).resolve()
    artifacts_dir = Path(args.artifacts).resolve()
    artifacts_dir.mkdir(parents=True, exist_ok=True)

    port = _find_free_port()
    base_url = f"http://127.0.0.1:{port}"
    screenshot_path = artifacts_dir / "smoke.png"
    tools_dump_path = artifacts_dir / "tools.json"
    console_dump_path = artifacts_dir / "console.json"

    streamlit_proc = _start_streamlit(app_path, port=port, streamlit_cmd=args.streamlit_cmd)
    try:
        _wait_http_ok(base_url, timeout_s=45.0)
        print(f"[ok] Streamlit reachable at {base_url}")

        mcp_cmd = shlex.split(args.playwright_mcp_cmd)
        if args.headless:
            mcp_cmd.append("--headless")

        mcp = MCPStdioClient(mcp_cmd, request_timeout_s=120.0)
        mcp.start()
        init_res = mcp.initialize(protocol_version="2024-11-05")
        print(f"[ok] MCP initialize: {json.dumps(init_res)[:200]}...")

        tools = mcp.list_tools()
        tools_dump_path.write_text(
            json.dumps(
                [{"name": t.name, "description": t.description, "inputSchema": t.input_schema} for t in tools],
                indent=2,
            ),
            encoding="utf-8",
        )
        print(f"[ok] Discovered {len(tools)} MCP tools (saved to {tools_dump_path})")

        nav_tool = _pick_tool(tools, ["navigate", "goto", "open", "url"])
        shot_tool = _pick_tool(tools, ["screenshot", "snapshot", "capture"])
        console_tool = _pick_tool(tools, ["console_messages", "console", "console-messages", "logs"])

        if nav_tool is None:
            raise RuntimeError("Could not find a navigation-like tool in tools/list output.")
        if shot_tool is None:
            raise RuntimeError("Could not find a screenshot-like tool in tools/list output.")

        nav_args = _best_effort_args(nav_tool, url=base_url, screenshot_path=screenshot_path)
        print(f"[run] Calling navigate tool: {nav_tool.name} args={nav_args}")
        mcp.call_tool(nav_tool.name, nav_args)

        shot_args = _best_effort_args(shot_tool, url=base_url, screenshot_path=screenshot_path)
        print(f"[run] Calling screenshot tool: {shot_tool.name} args={shot_args}")
        shot_res = mcp.call_tool(shot_tool.name, shot_args)

        if not screenshot_path.exists() and isinstance(shot_res, dict):
            maybe_b64 = shot_res.get("data") or shot_res.get("base64")
            if isinstance(maybe_b64, str) and maybe_b64:
                screenshot_path.write_bytes(base64.b64decode(maybe_b64))

        if screenshot_path.exists():
            print(f"[ok] Screenshot saved to {screenshot_path}")
        else:
            print("[warn] Screenshot not written to disk. Inspect tool response in logs/artifacts.")

        # Best-effort console capture (non-fatal).
        if console_tool is not None:
            try:
                console_args = _best_effort_args(console_tool, url=base_url, screenshot_path=screenshot_path)
                required = {k.lower() for k in _tool_required(console_tool)}
                if required and not required.issubset({k.lower() for k in console_args.keys()}):
                    print(f"[warn] Console tool requires {sorted(required)}; skipping capture.")
                else:
                    print(f"[run] Calling console tool: {console_tool.name} args={console_args}")
                    console_res = mcp.call_tool(console_tool.name, console_args)
                    console_dump_path.write_text(
                        json.dumps(console_res, indent=2, sort_keys=True, default=str),
                        encoding="utf-8",
                    )
                    print(f"[ok] Console messages saved to {console_dump_path}")
            except Exception as e:
                print(f"[warn] Console capture failed: {e}")

        mcp.terminate()
        return 0
    finally:
        _stop_process(streamlit_proc, name="streamlit")


if __name__ == "__main__":
    raise SystemExit(main())
