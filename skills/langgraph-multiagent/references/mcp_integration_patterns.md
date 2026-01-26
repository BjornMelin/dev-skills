# MCP integration patterns (LangChain/LangGraph)

MCP (Model Context Protocol) standardizes how servers expose tools and context to LLM apps.

## Default posture

Treat MCP tools as **untrusted** until proven otherwise:

- validate args at boundaries
- enforce allowlists/permissions
- add timeouts/retries
- trace everything

## Multi-server tools (Python)

Use `langchain-mcp-adapters` to load tools from multiple MCP servers and expose them to a LangChain agent.

Key concepts:

- `MultiServerMCPClient`: aggregates tools from many servers
- tool interceptors: wrap tool calls (auth gates, argument injection, logging)
- ToolRuntime context: access `state`, `config`, `store`, and user `context`

## High-value interceptor patterns

### 1) Auth gating

- Block sensitive tools unless `runtime.state` indicates the user is authenticated.
- Return a `ToolMessage` error instead of executing the tool.

### 2) Dependency injection

- Inject user-scoped credentials or identifiers from `runtime.context` into tool args.
- Never allow the model to see secrets; inject them server-side.

### 3) Tool allowlists + domain restrictions

- For URL fetch tools: allow only specific domains/prefixes to prevent SSRF/data exfil.
- For DB tools: enforce parameterized queries and least-privilege DB users.

## When MCP is a great fit

- enterprise tool ecosystems (auth, governance, central tool registry)
- multi-tenant apps where tools must be user-scoped
- heterogeneous tool stacks (mixing internal + third-party services)

