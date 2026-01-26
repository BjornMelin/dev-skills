import ChatClient from "./ChatClient";

export default function Page() {
  const apiUrl =
    process.env.NEXT_PUBLIC_LANGGRAPH_API_URL ?? "http://localhost:2024";
  const assistantId = process.env.NEXT_PUBLIC_LANGGRAPH_ASSISTANT_ID ?? "agent";

  return <ChatClient apiUrl={apiUrl} assistantId={assistantId} />;
}

