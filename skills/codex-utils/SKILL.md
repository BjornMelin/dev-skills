---
name: codex-utils
description: Discover available tools, coordinate safe parallel reads, clarify decisions, inspect images, and verify current sources.
---

# Codex Utils

Use the lightest available tool that resolves the need. The current session's
tool catalog, schemas, and restrictions are authoritative; never assume a tool
name from another host or session exists.

## Coordination

- Use a planning tool when available and useful for non-trivial work; otherwise
  keep a short checklist. Do not create a persistent goal unless requested.
- Use the available user-input tool for optional clarification, following its
  current mode restrictions. Ask directly when explicit approval is required;
  a recommended or preselected option is not approval.
- Inspect local images with the available image-viewing tool before editing or
  judging them. Use original detail only when necessary.

## Discovery and execution

- Search the exposed tool catalog before declaring a capability missing.
  In Code Mode, filter `ALL_TOOLS` by name/description and call the matching
  `tools` method. Use tool search only when the session exposes it.
- Match the actual schema before calling. Never print credentials, raw auth
  configuration, or secret-bearing subprocess arguments while inspecting tools.
- Batch independent read-only calls with `Promise.allSettled` when Code Mode is
  available; inspect every result. Keep dependent calls, mutations, approvals,
  waits, and tools marked non-parallel sequential.
- Request plugin installation only for a plugin or connector explicitly requested
  by the user, after the available discovery/install tools confirm an exact match.
  Installed, enabled, exposed, and healthy are different states; verify the one
  the task needs.

## Sources

Use the available browsing tool's search/open/find operations for current facts
and source verification. Prefer primary sources; inspect the actual page before
relying on search snippets. For OpenAI product questions, inspect local code/help
first and use official OpenAI documentation as the fallback. Keep private source
and credentials out of external research services.

## Defaults

Verify unstable or high-stakes claims with tools instead of memory. Keep tool use
proportional: do not browse, ask, plan, install, or delegate when unnecessary.
