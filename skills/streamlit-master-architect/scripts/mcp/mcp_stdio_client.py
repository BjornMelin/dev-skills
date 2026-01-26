from __future__ import annotations

import json
import queue
import subprocess
import threading
import time
from dataclasses import dataclass
from typing import Any, Mapping


JsonValue = Any


class MCPProtocolError(RuntimeError):
    """Raised when MCP JSON-RPC framing or protocol assumptions fail."""


class MCPRequestError(RuntimeError):
    """Raised when an MCP request returns an error response."""


@dataclass(frozen=True)
class MCPTool:
    name: str
    description: str | None
    input_schema: Mapping[str, JsonValue] | None


@dataclass(frozen=True)
class MCPResponse:
    id: int
    result: JsonValue | None
    error: JsonValue | None


def _encode_framed_message(payload: Mapping[str, JsonValue]) -> bytes:
    body = json.dumps(payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    header = f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
    return header + body


def _read_exact(stream: Any, n: int) -> bytes:
    buf = bytearray()
    while len(buf) < n:
        chunk = stream.read(n - len(buf))
        if not chunk:
            raise MCPProtocolError("Unexpected EOF while reading framed message body.")
        buf.extend(chunk)
    return bytes(buf)


def _read_framed_message(stream: Any) -> Mapping[str, JsonValue]:
    headers: dict[str, str] = {}
    while True:
        line = stream.readline()
        if line is None or line == b"":
            raise MCPProtocolError("EOF while reading headers.")
        if line in (b"\r\n", b"\n"):
            break
        decoded = line.decode("ascii", errors="strict").strip()
        if ":" not in decoded:
            continue
        k, v = decoded.split(":", 1)
        headers[k.strip().lower()] = v.strip()

    if "content-length" not in headers:
        raise MCPProtocolError(f"Missing Content-Length header. Headers: {headers}")

    try:
        length = int(headers["content-length"])
    except ValueError as e:
        raise MCPProtocolError(f"Invalid Content-Length: {headers['content-length']!r}") from e

    body = _read_exact(stream, length)
    try:
        msg = json.loads(body.decode("utf-8"))
    except Exception as e:
        raise MCPProtocolError(f"Invalid JSON body: {body[:200]!r}...") from e

    if not isinstance(msg, dict):
        raise MCPProtocolError(f"Expected JSON object message, got: {type(msg)}")

    return msg


class MCPStdioClient:
    """Minimal MCP stdio client (JSON-RPC 2.0 with Content-Length framing)."""

    def __init__(
        self,
        command: list[str],
        *,
        cwd: str | None = None,
        env: Mapping[str, str] | None = None,
        startup_timeout_s: float = 15.0,
        request_timeout_s: float = 60.0,
        log_notifications: bool = True,
    ) -> None:
        self._command = command
        self._cwd = cwd
        self._env = dict(env) if env is not None else None
        self._startup_timeout_s = startup_timeout_s
        self._request_timeout_s = request_timeout_s
        self._log_notifications = log_notifications

        self._proc: subprocess.Popen[bytes] | None = None
        self._rx_thread: threading.Thread | None = None
        self._rx_queue: queue.Queue[Mapping[str, JsonValue]] = queue.Queue()
        self._next_id = 1

    def start(self) -> None:
        if self._proc is not None:
            return
        self._proc = subprocess.Popen(
            self._command,
            cwd=self._cwd,
            env=self._env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        if self._proc.stdin is None or self._proc.stdout is None:
            raise MCPProtocolError("Failed to open stdio pipes to MCP server process.")

        self._rx_thread = threading.Thread(target=self._reader_loop, daemon=True)
        self._rx_thread.start()

        deadline = time.time() + self._startup_timeout_s
        while time.time() < deadline:
            if self._proc.poll() is not None:
                stderr = self._safe_read_stderr()
                raise MCPProtocolError(
                    f"MCP server process exited early (code {self._proc.returncode}).\n{stderr}"
                )
            if not self._rx_queue.empty():
                return
            time.sleep(0.05)

    def terminate(self) -> None:
        if self._proc is None:
            return
        if self._proc.poll() is None:
            self._proc.terminate()
            try:
                self._proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._proc.kill()
        self._proc = None

    def initialize(
        self,
        *,
        protocol_version: str = "2024-11-05",
        client_name: str = "streamlit-master-architect",
        client_version: str = "0.1.0",
        capabilities: Mapping[str, JsonValue] | None = None,
    ) -> JsonValue:
        caps = dict(capabilities) if capabilities is not None else {}
        result = self.request(
            "initialize",
            {
                "protocolVersion": protocol_version,
                "clientInfo": {"name": client_name, "version": client_version},
                "capabilities": caps,
            },
        )
        try:
            self.notify("initialized", {})
        except Exception:
            pass
        return result

    def list_tools(self) -> list[MCPTool]:
        res = self.request("tools/list", {})
        tools_raw = res.get("tools", []) if isinstance(res, dict) else []
        tools: list[MCPTool] = []
        for t in tools_raw:
            if not isinstance(t, dict):
                continue
            tools.append(
                MCPTool(
                    name=str(t.get("name", "")),
                    description=(str(t["description"]) if t.get("description") is not None else None),
                    input_schema=(t.get("inputSchema") if isinstance(t.get("inputSchema"), dict) else None),
                )
            )
        return tools

    def call_tool(self, name: str, arguments: Mapping[str, JsonValue]) -> JsonValue:
        return self.request("tools/call", {"name": name, "arguments": dict(arguments)})

    def request(self, method: str, params: Mapping[str, JsonValue]) -> JsonValue:
        req_id = self._next_id
        self._next_id += 1

        self._send({"jsonrpc": "2.0", "id": req_id, "method": method, "params": dict(params)})

        deadline = time.time() + self._request_timeout_s
        while time.time() < deadline:
            msg = self._rx_queue.get(timeout=max(0.05, self._request_timeout_s / 200))
            if "id" not in msg:
                self._handle_notification(msg)
                continue
            if msg.get("id") != req_id:
                self._rx_queue.put(msg)
                continue

            resp = MCPResponse(id=req_id, result=msg.get("result"), error=msg.get("error"))
            if resp.error is not None:
                raise MCPRequestError(f"MCP request {method} failed: {resp.error}")
            return resp.result

        raise TimeoutError(f"MCP request timed out: {method}")

    def notify(self, method: str, params: Mapping[str, JsonValue]) -> None:
        self._send({"jsonrpc": "2.0", "method": method, "params": dict(params)})

    def _send(self, payload: Mapping[str, JsonValue]) -> None:
        if self._proc is None or self._proc.stdin is None:
            raise MCPProtocolError("MCP process not started.")
        framed = _encode_framed_message(payload)
        try:
            self._proc.stdin.write(framed)
            self._proc.stdin.flush()
        except BrokenPipeError as e:
            stderr = self._safe_read_stderr()
            raise MCPProtocolError(f"MCP server stdin closed.\n{stderr}") from e

    def _reader_loop(self) -> None:
        assert self._proc is not None
        assert self._proc.stdout is not None
        stdout = self._proc.stdout
        while True:
            if self._proc.poll() is not None:
                return
            try:
                msg = _read_framed_message(stdout)
            except Exception as e:
                if self._proc.poll() is not None:
                    return
                self._rx_queue.put({"jsonrpc": "2.0", "method": "mcp.protocol_error", "params": {"error": str(e)}})
                return
            self._rx_queue.put(msg)

    def _handle_notification(self, msg: Mapping[str, JsonValue]) -> None:
        if not self._log_notifications:
            return
        method = msg.get("method")
        if not method:
            return
        if method in ("notifications/message", "log", "logging/message", "mcp.protocol_error"):
            params = msg.get("params", {})
            print(f"[MCP notification] {method}: {params}")

    def _safe_read_stderr(self) -> str:
        if self._proc is None or self._proc.stderr is None:
            return ""
        try:
            return self._proc.stderr.read().decode("utf-8", errors="replace")
        except Exception:
            return ""

