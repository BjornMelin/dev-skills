import { Buffer } from "node:buffer";
import { applyRunCommand, startRun, statusRun } from "./lifecycle";
import { projectRootFrom, safeSegment } from "./paths";

type JsonRpc = {
  jsonrpc?: string;
  id?: string | number | null;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
};

export type McpFrameFormat = "line" | "content-length";

type ParsedFrame = {
  format: McpFrameFormat;
  request: JsonRpc;
};

const LATEST_PROTOCOL_VERSION = "2025-11-25";
const SUPPORTED_PROTOCOL_VERSIONS = [LATEST_PROTOCOL_VERSION, "2025-06-18", "2025-03-26", "2024-11-05"] as const;
const MAX_MESSAGE_BYTES = 10 * 1024 * 1024;

function toolSchemas(): Record<string, unknown>[] {
  return [
    {
      name: "start",
      description: "Create a plan-first Kimi UI Agent worktree run. Mutates only when apply is true.",
      inputSchema: {
        type: "object",
        additionalProperties: false,
        required: ["task"],
        properties: {
          task: { type: "string" },
          projectRoot: { type: "string" },
          apply: { type: "boolean", default: false },
        },
      },
    },
    {
      name: "status",
      description: "Read Kimi UI Agent run status.",
      inputSchema: {
        type: "object",
        additionalProperties: false,
        required: ["runId"],
        properties: { runId: { type: "string" } },
      },
    },
    {
      name: "reply",
      description: "Append a user reply or approval message to a run.",
      inputSchema: {
        type: "object",
        additionalProperties: false,
        required: ["runId", "message", "apply"],
        properties: { runId: { type: "string" }, message: { type: "string" }, apply: { type: "boolean" } },
      },
    },
    {
      name: "continue",
      description: "Mark a run ready to continue.",
      inputSchema: {
        type: "object",
        additionalProperties: false,
        required: ["runId", "apply"],
        properties: { runId: { type: "string" }, apply: { type: "boolean" } },
      },
    },
    {
      name: "finalize",
      description: "Mark a run finalized after parent review.",
      inputSchema: {
        type: "object",
        additionalProperties: false,
        required: ["runId", "apply"],
        properties: { runId: { type: "string" }, apply: { type: "boolean" } },
      },
    },
    {
      name: "abort",
      description: "Mark a run aborted.",
      inputSchema: {
        type: "object",
        additionalProperties: false,
        required: ["runId", "apply"],
        properties: { runId: { type: "string" }, reason: { type: "string" }, apply: { type: "boolean" } },
      },
    },
  ];
}

function textContent(value: unknown): { content: { type: "text"; text: string }[] } {
  return { content: [{ type: "text", text: JSON.stringify(value, null, 2) }] };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function getString(args: Record<string, unknown>, key: string, required = true): string | undefined {
  const value = args[key];
  if (value === undefined || value === null) {
    if (required) throw new Error(`${key} is required`);
    return undefined;
  }
  if (typeof value !== "string" || !value.trim()) throw new Error(`${key} must be a non-empty string`);
  return value;
}

function getBoolean(args: Record<string, unknown>, key: string): boolean {
  const value = args[key];
  if (typeof value !== "boolean") throw new Error(`${key} must be a boolean`);
  return value;
}

function assertNoUnknown(args: Record<string, unknown>, allowed: string[]): void {
  for (const key of Object.keys(args)) {
    if (!allowed.includes(key)) throw new Error(`unknown field: ${key}`);
  }
}

async function callTool(name: string, rawArgs: Record<string, unknown>): Promise<unknown> {
  if (name === "start") {
    assertNoUnknown(rawArgs, ["task", "projectRoot", "apply"]);
    const projectRoot = projectRootFrom(process.cwd(), getString(rawArgs, "projectRoot", false));
    const task = getString(rawArgs, "task")!;
    const apply = rawArgs.apply === true;
    return startRun({ projectRoot, task, apply });
  }
  if (name === "status") {
    assertNoUnknown(rawArgs, ["runId"]);
    return statusRun(safeSegment(getString(rawArgs, "runId")!, "run id"));
  }
  if (name === "reply") {
    assertNoUnknown(rawArgs, ["runId", "message", "apply"]);
    return applyRunCommand(safeSegment(getString(rawArgs, "runId")!, "run id"), "reply", {
      apply: getBoolean(rawArgs, "apply"),
      message: getString(rawArgs, "message")!,
    });
  }
  if (name === "continue") {
    assertNoUnknown(rawArgs, ["runId", "apply"]);
    return applyRunCommand(safeSegment(getString(rawArgs, "runId")!, "run id"), "continue", { apply: getBoolean(rawArgs, "apply") });
  }
  if (name === "finalize") {
    assertNoUnknown(rawArgs, ["runId", "apply"]);
    return applyRunCommand(safeSegment(getString(rawArgs, "runId")!, "run id"), "finalize", { apply: getBoolean(rawArgs, "apply") });
  }
  if (name === "abort") {
    assertNoUnknown(rawArgs, ["runId", "reason", "apply"]);
    const reason = getString(rawArgs, "reason", false);
    return applyRunCommand(safeSegment(getString(rawArgs, "runId")!, "run id"), "abort", {
      apply: getBoolean(rawArgs, "apply"),
      ...(reason ? { reason } : {}),
    });
  }
  throw new Error(`unknown tool: ${name}`);
}

function parseJsonRpc(source: string): JsonRpc {
  const value = JSON.parse(source) as unknown;
  if (!isRecord(value)) throw new Error("JSON-RPC message must be an object");
  return value as JsonRpc;
}

function firstLine(buffer: Buffer): string {
  const newline = buffer.indexOf("\n");
  const end = newline === -1 ? Math.min(buffer.length, 128) : newline;
  return buffer.toString("ascii", 0, end).replace(/\r$/, "");
}

function startsWithContentLength(buffer: Buffer): boolean {
  return /^Content-Length\s*:/i.test(firstLine(buffer));
}

function headerEnd(buffer: Buffer): { index: number; length: number } | undefined {
  const crlf = buffer.indexOf("\r\n\r\n");
  const lf = buffer.indexOf("\n\n");
  if (crlf === -1 && lf === -1) return undefined;
  if (crlf !== -1 && (lf === -1 || crlf < lf)) return { index: crlf, length: 4 };
  return { index: lf, length: 2 };
}

function contentLength(header: string): number {
  const line = header.split(/\r?\n/).find((entry) => /^Content-Length\s*:/i.test(entry));
  if (!line) throw new Error("missing Content-Length header");
  const value = line.slice(line.indexOf(":") + 1).trim();
  if (!/^\d+$/.test(value)) throw new Error("invalid Content-Length header");
  const length = Number(value);
  if (!Number.isSafeInteger(length) || length < 0) throw new Error("invalid Content-Length header");
  if (length > MAX_MESSAGE_BYTES) throw new Error("MCP message exceeds maximum size");
  return length;
}

export function readMcpFrames(buffer: Buffer): { frames: ParsedFrame[]; remaining: Buffer } {
  const frames: ParsedFrame[] = [];
  let remaining = buffer;

  while (remaining.length > 0) {
    if (startsWithContentLength(remaining)) {
      const end = headerEnd(remaining);
      if (!end) break;
      const length = contentLength(remaining.toString("ascii", 0, end.index));
      const bodyStart = end.index + end.length;
      const bodyEnd = bodyStart + length;
      if (remaining.length < bodyEnd) break;
      frames.push({ format: "content-length", request: parseJsonRpc(remaining.toString("utf8", bodyStart, bodyEnd)) });
      remaining = remaining.subarray(bodyEnd);
      continue;
    }

    const newline = remaining.indexOf("\n");
    if (newline === -1) {
      if (remaining.length > MAX_MESSAGE_BYTES) throw new Error("MCP message exceeds maximum size");
      break;
    }

    const line = remaining.toString("utf8", 0, newline).replace(/\r$/, "").trim();
    remaining = remaining.subarray(newline + 1);
    if (!line) continue;
    frames.push({ format: "line", request: parseJsonRpc(line) });
  }

  return { frames, remaining };
}

export function encodeMcpMessage(message: JsonRpc, format: McpFrameFormat): string {
  const body = JSON.stringify(message);
  if (format === "content-length") {
    return `Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`;
  }
  return `${body}\n`;
}

function negotiateProtocolVersion(params: unknown): string {
  const requested = isRecord(params) && typeof params.protocolVersion === "string" ? params.protocolVersion : undefined;
  if (requested && SUPPORTED_PROTOCOL_VERSIONS.includes(requested as (typeof SUPPORTED_PROTOCOL_VERSIONS)[number])) return requested;
  return LATEST_PROTOCOL_VERSION;
}

function rpcError(id: JsonRpc["id"], code: number, message: string): JsonRpc {
  return { jsonrpc: "2.0", id: id ?? null, error: { code, message } };
}

export async function handleJsonRpc(request: JsonRpc): Promise<JsonRpc | undefined> {
  if (request.id === undefined) return undefined;

  try {
    let result: unknown;
    if (request.method === "initialize") {
      result = {
        protocolVersion: negotiateProtocolVersion(request.params),
        capabilities: { tools: {} },
        serverInfo: { name: "kimi-ui-agent", title: "Kimi UI Agent", version: "0.1.0" },
      };
    } else if (request.method === "tools/list") {
      result = { tools: toolSchemas() };
    } else if (request.method === "tools/call") {
      const params = isRecord(request.params) ? request.params : {};
      const name = getString(params, "name")!;
      const args = isRecord(params.arguments) ? params.arguments : {};
      result = textContent(await callTool(name, args));
    } else {
      return rpcError(request.id, -32601, `method not found: ${request.method}`);
    }
    return { jsonrpc: "2.0", id: request.id, result };
  } catch (error) {
    return rpcError(request.id, -32000, error instanceof Error ? error.message : String(error));
  }
}

export async function runMcpServer(): Promise<void> {
  let buffer: Buffer<ArrayBufferLike> = Buffer.alloc(0);
  let responseFormat: McpFrameFormat = "line";
  for await (const chunk of process.stdin) {
    const incoming = Buffer.isBuffer(chunk) ? chunk : Buffer.from(String(chunk));
    buffer = Buffer.concat([buffer, incoming]);

    let frames: ParsedFrame[];
    try {
      const parsed = readMcpFrames(buffer);
      frames = parsed.frames;
      buffer = parsed.remaining;
    } catch (error) {
      if (startsWithContentLength(buffer)) responseFormat = "content-length";
      buffer = Buffer.alloc(0);
      process.stdout.write(encodeMcpMessage(rpcError(null, -32700, error instanceof Error ? error.message : String(error)), responseFormat));
      continue;
    }

    for (const frame of frames) {
      responseFormat = frame.format;
      const response = await handleJsonRpc(frame.request);
      if (response) {
        process.stdout.write(encodeMcpMessage(response, responseFormat));
      }
    }
  }
}
