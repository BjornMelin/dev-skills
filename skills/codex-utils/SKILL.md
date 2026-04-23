---
name: codex-utils
description: Use built-in Codex coordination, retrieval, and utility tools effectively during live sessions. Trigger when work benefits from maintaining a step plan, asking the user 1-3 short multiple-choice questions before risky edits, inspecting a local image from disk, searching or browsing the web for current or source-backed information, discovering additional MCP tools or plugins, or running independent developer-tool calls in parallel.
---

# Codex Utils

Use these built-ins when they materially improve accuracy, safety, or speed.
Prefer the lightest tool that resolves the need.

## Coordination Utilities

- Use `functions.update_plan` for non-trivial multi-step work. Keep one step
  `in_progress` and mark completed steps as you advance.
- Use `functions.request_user_input` before risky edits or when a decision
  cannot be inferred safely. Ask 1-3 short multiple-choice questions, put the
  recommended option first, and keep options mutually exclusive.
- Use `functions.view_image` to inspect a local image from disk when visual
  details matter. Pass an absolute path. Use `detail: original` only when
  high-fidelity inspection is necessary.

## Web + Internet

- Use `web.search_query` for current facts, source-backed answers, official
  documentation, or external verification. Search narrowly and prefer primary
  sources.
- Use `web.open` to read a result directly or jump to relevant lines.
- Use `web.click` to follow numbered links inside an opened page.
- Use `web.find` to locate exact text in a long page before opening more
  context.
- Use `web.screenshot` only for PDFs when a page image is needed.
- Use `web.image_query` when reference imagery or visual comparison helps.
- Use `web.time` when you need the current time for a specific UTC offset.

## Tooling + MCP Discovery

- Use `tool_search.tool_search_tool` to discover extra MCP tools or plugins
  before assuming a capability is unavailable.
- Use `multi_tool_use.parallel` only for independent developer-tool calls that
  are safe to run concurrently, such as parallel reads or separate inspections.
  Do not parallelize dependent steps or tools marked as non-parallel.

## Defaults

- Verify unstable or high-stakes claims with tools instead of memory.
- Keep tool use proportional to the task; do not browse, ask, or plan when the
  answer is already clear and stable.
