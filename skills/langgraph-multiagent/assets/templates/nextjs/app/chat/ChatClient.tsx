"use client";

import { useEffect, useMemo, useState } from "react";
import { useStream } from "@langchain/langgraph-sdk/react";
import type { Message } from "@langchain/langgraph-sdk";

import type { AgentState, HITLRequest, HITLResponse } from "./types";
import { HITLRequestSchema } from "./types";
import Response from "./Response";

type Props = {
  apiUrl: string;
  assistantId: string;
};

type DecisionType = "approve" | "reject" | "edit";
type StreamRef = ReturnType<typeof useStream>;

const THREAD_ID_STORAGE_KEY = "langgraph:thread_id";

function safeStringify(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function renderContent(content: unknown): string {
  if (typeof content === "string") return content;
  if (content == null) return "";
  return safeStringify(content);
}

export default function ChatClient({ apiUrl, assistantId }: Props) {
  const [threadId, setThreadId] = useState<string | null>(null);
  const [input, setInput] = useState("");

  useEffect(() => {
    setThreadId(window.localStorage.getItem(THREAD_ID_STORAGE_KEY));
  }, []);

  useEffect(() => {
    if (!threadId) return;
    window.localStorage.setItem(THREAD_ID_STORAGE_KEY, threadId);
  }, [threadId]);

  const stream = useStream<AgentState, { InterruptType: HITLRequest }>({
    apiUrl,
    assistantId,
    threadId,
    onThreadId: setThreadId,
    reconnectOnMount: true,
  });

  const [hitlUiError, setHitlUiError] = useState<string | null>(null);
  const [decisionModeByIndex, setDecisionModeByIndex] = useState<
    Record<number, DecisionType>
  >({});
  const [editedArgsJsonByIndex, setEditedArgsJsonByIndex] = useState<
    Record<number, string>
  >({});
  const [rejectReason, setRejectReason] = useState("User rejected");

  const hitlRequest = useMemo(() => {
    const candidate = stream.interrupt?.value;
    const parsed = HITLRequestSchema.safeParse(candidate);
    return parsed.success ? parsed.data : undefined;
  }, [stream.interrupt]);

  useEffect(() => {
    if (!hitlRequest) return;
    const nextDecisionModes: Record<number, DecisionType> = {};
    const nextArgs: Record<number, string> = {};
    hitlRequest.actionRequests.forEach((action, idx) => {
      nextDecisionModes[idx] = "approve";
      nextArgs[idx] = safeStringify(action.args ?? action.arguments ?? {});
    });
    setDecisionModeByIndex(nextDecisionModes);
    setEditedArgsJsonByIndex(nextArgs);
    setHitlUiError(null);
  }, [hitlRequest]);

  const handleSubmit = async () => {
    const text = input.trim();
    if (!text) return;
    setInput("");
    await stream.submit({
      messages: [{ type: "human", content: text }],
    });
  };

  const handleNewThread = () => {
    window.localStorage.removeItem(THREAD_ID_STORAGE_KEY);
    setThreadId(null);
    stream.stop();
  };

  const handleResumeFromInterrupt = async () => {
    if (!hitlRequest) return;
    setHitlUiError(null);

    const decisions: HITLResponse["decisions"] = hitlRequest.actionRequests.map(
      (action, idx) => {
        const mode = decisionModeByIndex[idx] ?? "approve";
        if (mode === "approve") return { type: "approve" };
        if (mode === "reject") return { type: "reject", message: rejectReason };

        const raw = editedArgsJsonByIndex[idx] ?? "{}";
        try {
          const args = JSON.parse(raw) as Record<string, unknown>;
          const editedAction =
            action.arguments !== undefined
              ? { name: action.name, arguments: args }
              : { name: action.name, args };
          return {
            type: "edit",
            editedAction,
          };
        } catch {
          return { type: "reject", message: "Invalid edited JSON args" };
        }
      }
    );

    const resume: HITLResponse = { decisions };

    await stream.submit(null, {
      command: { resume },
    });
  };

  return (
    <div className="mx-auto flex h-dvh max-w-4xl flex-col gap-4 p-4">
      <header className="flex items-center justify-between gap-2">
        <div className="flex flex-col">
          <div className="text-sm font-medium">LangGraph Chat</div>
          <div className="text-xs text-muted-foreground">
            assistant: <span className="font-mono">{assistantId}</span> · thread:{" "}
            <span className="font-mono">{threadId ?? "—"}</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            className="rounded-md border px-3 py-1.5 text-sm"
            onClick={handleNewThread}
          >
            New thread
          </button>
        </div>
      </header>

      <main className="flex-1 overflow-auto rounded-lg border bg-background p-3">
        <div className="flex flex-col gap-3">
          {stream.messages.map((message, idx) => (
            <MessageRow
              key={(message.id ?? idx) as string}
              message={message as Message}
              stream={stream}
            />
          ))}

          {stream.isLoading ? (
            <div className="text-sm text-muted-foreground">Thinking…</div>
          ) : null}
          {stream.error ? (
            <div className="rounded-md border border-red-500/40 bg-red-500/10 p-3 text-sm text-red-500">
              {stream.error.message}
            </div>
          ) : null}

          {hitlRequest && hitlRequest.actionRequests.length > 0 ? (
            <section className="rounded-lg border border-amber-500/40 bg-amber-500/10 p-3">
              <div className="mb-2 text-sm font-semibold text-amber-600">
                Human approval required
              </div>
              <div className="mb-3 text-xs text-muted-foreground">
                Decide for each action, then resume the run.
              </div>

              <div className="flex flex-col gap-3">
                {hitlRequest.actionRequests.map((action, idx) => (
                  <div
                    key={idx}
                    className="rounded-md border bg-background p-3"
                  >
                    <div className="mb-2 flex items-center justify-between gap-2">
                      <div className="text-sm font-mono">{action.name}</div>
                      <select
                        className="rounded-md border bg-background px-2 py-1 text-xs"
                        value={decisionModeByIndex[idx] ?? "approve"}
                        onChange={(e) =>
                          setDecisionModeByIndex((prev) => ({
                            ...prev,
                            [idx]: e.target.value as DecisionType,
                          }))
                        }
                      >
                        <option value="approve">approve</option>
                        <option value="reject">reject</option>
                        <option value="edit">edit args</option>
                      </select>
                    </div>

                    <div className="mb-2 text-xs text-muted-foreground">
                      {action.description ?? "Tool call pending review."}
                    </div>

                    {(decisionModeByIndex[idx] ?? "approve") === "edit" ? (
                      <textarea
                        className="h-28 w-full rounded-md border bg-background p-2 font-mono text-xs"
                        value={editedArgsJsonByIndex[idx] ?? "{}"}
                        onChange={(e) =>
                          setEditedArgsJsonByIndex((prev) => ({
                            ...prev,
                            [idx]: e.target.value,
                          }))
                        }
                      />
                    ) : (
                      <pre className="overflow-auto rounded-md border bg-muted p-2 text-xs">
                        {safeStringify(action.args ?? action.arguments ?? {})}
                      </pre>
                    )}
                  </div>
                ))}
              </div>

              <div className="mt-3 flex flex-col gap-2">
                <div className="flex items-center gap-2">
                  <input
                    className="w-full rounded-md border bg-background px-2 py-1 text-xs"
                    value={rejectReason}
                    onChange={(e) => setRejectReason(e.target.value)}
                    placeholder="Reject reason (used for any reject decisions)"
                  />
                  <button
                    type="button"
                    className="rounded-md border bg-background px-3 py-1.5 text-xs"
                    onClick={handleResumeFromInterrupt}
                  >
                    Resume
                  </button>
                </div>

                {hitlUiError ? (
                  <div className="text-xs text-red-500">{hitlUiError}</div>
                ) : null}
              </div>
            </section>
          ) : null}
        </div>
      </main>

      <footer className="flex items-end gap-2">
        <textarea
          className="h-20 flex-1 resize-none rounded-md border bg-background p-2 text-sm"
          placeholder="Type a message…"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
              e.preventDefault();
              handleSubmit();
            }
          }}
        />
        <button
          type="button"
          className="rounded-md border bg-background px-3 py-2 text-sm"
          disabled={stream.isLoading}
          onClick={handleSubmit}
        >
          Send
        </button>
      </footer>
      <div className="text-xs text-muted-foreground">
        Tip: press <span className="font-mono">Ctrl+Enter</span> to send.
      </div>
    </div>
  );
}

function MessageRow({
  message,
  stream,
}: {
  message: Message;
  stream: StreamRef;
}) {
  const meta = stream.getMessageMetadata(message);
  const node = (meta as { langgraph_node?: string } | undefined)?.langgraph_node;
  const role = message.type;
  const toolCalls =
    role === "ai" ? (stream.getToolCalls(message) as unknown[]) : [];
  const content = renderContent(message.content);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          {role}
          {node ? <span className="ml-2 font-mono lowercase">{node}</span> : null}
        </div>
      </div>

      {toolCalls.length > 0 ? (
        <div className="flex flex-col gap-2">
          {toolCalls.map((tc, idx) => (
            <ToolCallCard key={idx} toolCall={tc} />
          ))}
        </div>
      ) : null}

      {role === "ai" && typeof message.content === "string" ? (
        <div className="rounded-md border bg-background p-2">
          <Response isAnimating={stream.isLoading}>{content}</Response>
        </div>
      ) : (
        <pre className="whitespace-pre-wrap rounded-md border bg-muted p-2 text-sm">
          {content}
        </pre>
      )}
    </div>
  );
}

function ToolCallCard({ toolCall }: { toolCall: unknown }) {
  const tc = toolCall as {
    id?: string;
    state?: string;
    call?: { name?: string; args?: unknown };
    result?: unknown;
    error?: unknown;
  };
  return (
    <div className="rounded-md border bg-background p-2 text-xs">
      <div className="flex items-center justify-between gap-2">
        <div className="font-mono">
          {tc.call?.name ?? "tool_call"}
          {tc.id ? <span className="text-muted-foreground"> · {tc.id}</span> : null}
        </div>
        {tc.state ? (
          <div className="rounded bg-muted px-2 py-0.5 font-mono text-[10px]">
            {tc.state}
          </div>
        ) : null}
      </div>
      <pre className="mt-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px]">
        {safeStringify(tc.call?.args ?? {})}
      </pre>
      {tc.result != null ? (
        <pre className="mt-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px]">
          {safeStringify(tc.result)}
        </pre>
      ) : null}
      {tc.error != null ? (
        <pre className="mt-2 overflow-auto rounded bg-red-500/10 p-2 font-mono text-[11px] text-red-500">
          {safeStringify(tc.error)}
        </pre>
      ) : null}
    </div>
  );
}
