import { convertToModelMessages, streamText, type UIMessage } from "ai";
import { openai } from "@ai-sdk/openai";

export const maxDuration = 30;

export async function POST(req: Request) {
  const { messages } = (await req.json()) as { messages: UIMessage[] };

  const result = streamText({
    // Swap this for your provider/model.
    model: openai("gpt-4o-mini"),
    system:
      "You are a helpful assistant. Use markdown for clarity when it helps.",
    // v6: convertToModelMessages is async.
    messages: await convertToModelMessages(messages),
  });

  return result.toUIMessageStreamResponse({
    onError: (error) => {
      if (error instanceof Error) return error.message;
      return "Unknown error";
    },
  });
}

