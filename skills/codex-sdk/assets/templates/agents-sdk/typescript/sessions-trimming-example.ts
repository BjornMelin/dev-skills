import { ConversationalAgent } from "@openai/agents";
import { MemorySession } from "@openai/agents/sessions/memory";
import type { AgentInputItem } from "@openai/agents";

// Sessions example (TypeScript):
// - Use a session to persist history
// - Use sessionInputCallback to deterministically trim what the model sees

const session = new MemorySession();
const agent = new ConversationalAgent({ model: "gpt-4o" });

const sessionInputCallback = async (history: AgentInputItem[], newInputs: AgentInputItem[]) => {
  const combined = [...history, ...newInputs];
  return combined.slice(Math.max(0, combined.length - 6)); // keep last N items
};

const r1 = await agent.run({ input: "My name is Alice. I prefer vegetarian meals." }, { session, sessionInputCallback });
console.log(r1.output);

const r2 = await agent.run({ input: "Suggest dinner ideas." }, { session, sessionInputCallback });
console.log(r2.output);

