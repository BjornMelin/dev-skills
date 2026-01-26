import { z } from "zod";
import type { Message } from "@langchain/langgraph-sdk";

// ---------------------------------------------------------------------------
// Tool-call typing (fill this in per app)
// ---------------------------------------------------------------------------

export const ToolCallSchema = z.discriminatedUnion("name", [
  z.object({
    name: z.literal("search"),
    args: z.object({ query: z.string() }),
    id: z.string().optional(),
  }),
  z.object({
    name: z.literal("calculate"),
    args: z.object({ expression: z.string() }),
    id: z.string().optional(),
  }),
]);

export type ToolCall = z.infer<typeof ToolCallSchema>;

export type AgentMessage = Message<ToolCall>;

export interface AgentState {
  // Embed tool-call types in messages for end-to-end type safety in useStream().
  messages: AgentMessage[];
  // Optional: keep custom UI messages in graph state for Generative UI.
  ui?: unknown[];
}

// ---------------------------------------------------------------------------
// HITL typing (runtime value is versioned; validate defensively)
// ---------------------------------------------------------------------------

export const HITLActionRequestSchema = z.object({
  name: z.string(),
  // JS docs typically use `args`, but some examples show `arguments`.
  args: z.record(z.unknown()).optional(),
  arguments: z.record(z.unknown()).optional(),
  description: z.string().optional(),
});

export const HITLRequestSchema = z.preprocess(
  (value) => {
    if (value == null || typeof value !== "object") return value;
    const obj = value as Record<string, unknown>;
    return {
      actionRequests: (obj.actionRequests ?? obj.action_requests) as unknown,
      reviewConfigs: (obj.reviewConfigs ?? obj.review_configs) as unknown,
    };
  },
  z.object({
    actionRequests: z.array(HITLActionRequestSchema).default([]),
    reviewConfigs: z.array(z.unknown()).optional(),
  })
);

export type HITLRequest = z.infer<typeof HITLRequestSchema>;

const HITLEditedActionSchema = z.object({
  name: z.string(),
  args: z.record(z.unknown()).optional(),
  arguments: z.record(z.unknown()).optional(),
});

export const HITLDecisionSchema = z.discriminatedUnion("type", [
  z.object({ type: z.literal("approve") }),
  z.object({ type: z.literal("reject"), message: z.string().optional() }),
  z.object({
    type: z.literal("edit"),
    editedAction: HITLEditedActionSchema,
  }),
]);

export const HITLResponseSchema = z.object({
  decisions: z.array(HITLDecisionSchema),
});

export type HITLResponse = z.infer<typeof HITLResponseSchema>;
