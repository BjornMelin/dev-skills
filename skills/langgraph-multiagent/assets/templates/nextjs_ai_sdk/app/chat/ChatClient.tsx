"use client";

import { useState } from "react";
import { DefaultChatTransport } from "ai";
import { useChat } from "@ai-sdk/react";

import Response from "./Response";

type Props = {
  api?: string;
};

function safeJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

export default function ChatClient({ api = "/api/chat" }: Props) {
  const { messages, sendMessage, status, error, stop, regenerate } = useChat({
    transport: new DefaultChatTransport({ api }),
  });

  const [input, setInput] = useState("");

  return (
    <div className="mx-auto flex h-dvh max-w-4xl flex-col gap-4 p-4">
      <header className="flex items-center justify-between gap-2">
        <div className="flex flex-col">
          <div className="text-sm font-medium">AI SDK Chat</div>
          <div className="text-xs text-muted-foreground">
            transport: <span className="font-mono">{api}</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            className="rounded-md border px-3 py-1.5 text-sm"
            onClick={() => regenerate()}
            disabled={!(status === "ready" || status === "error")}
          >
            Regenerate
          </button>
          <button
            type="button"
            className="rounded-md border px-3 py-1.5 text-sm"
            onClick={() => stop()}
            disabled={status !== "streaming"}
          >
            Stop
          </button>
        </div>
      </header>

      <main className="flex-1 overflow-auto rounded-lg border bg-background p-3">
        <div className="flex flex-col gap-3">
          {messages.map((message) => (
            <div key={message.id} className="flex flex-col gap-2">
              <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                {message.role}
              </div>
              <div className="rounded-md border bg-background p-2">
                {message.parts.map((part, idx) => {
                  switch (part.type) {
                    case "text":
                      return (
                        <Response
                          key={idx}
                          isAnimating={status === "streaming"}
                        >
                          {part.text}
                        </Response>
                      );
                    case "tool-invocation":
                      return (
                        <pre
                          key={idx}
                          className="mt-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px]"
                        >
                          {safeJson(part)}
                        </pre>
                      );
                    case "tool-result":
                      return (
                        <pre
                          key={idx}
                          className="mt-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px]"
                        >
                          {safeJson(part)}
                        </pre>
                      );
                    case "file":
                      return (
                        <pre
                          key={idx}
                          className="mt-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px]"
                        >
                          {safeJson(part)}
                        </pre>
                      );
                    default:
                      return (
                        <pre
                          key={idx}
                          className="mt-2 overflow-auto rounded bg-muted p-2 font-mono text-[11px]"
                        >
                          {safeJson(part)}
                        </pre>
                      );
                  }
                })}
              </div>
            </div>
          ))}

          {status === "submitted" || status === "streaming" ? (
            <div className="text-sm text-muted-foreground">Thinking…</div>
          ) : null}
          {error ? (
            <div className="rounded-md border border-red-500/40 bg-red-500/10 p-3 text-sm text-red-500">
              {error.message}
            </div>
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
              void sendMessage({ text: input });
              setInput("");
            }
          }}
        />
        <button
          type="button"
          className="rounded-md border bg-background px-3 py-2 text-sm"
          disabled={status !== "ready"}
          onClick={() => {
            void sendMessage({ text: input });
            setInput("");
          }}
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

