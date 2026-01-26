# E2E: Playwright MCP (real browser automation)

## When to use E2E

Use Playwright E2E for:
- multipage navigation
- file upload/download UX
- chat UX (streaming output)
- custom components rendering
- screenshot regressions and console errors

## Setup

1) Install Playwright browsers once:

```bash
npx playwright install
```

2) Run the MCP server (stdio or HTTP).

### Stdio (used by bundled scripts)

The script `scripts/mcp/run_playwright_mcp_e2e.py` starts Streamlit and runs the Playwright MCP server via `npx`.

### HTTP (useful for connecting multiple clients)

```bash
npx @playwright/mcp@latest --port 8931
```

## Bundled E2E smoke script

```bash
python scripts/mcp/run_playwright_mcp_e2e.py --app path/to/streamlit_app.py
```

Artifacts:
- `artifacts/tools.json` — discovered MCP tools + schemas
- `artifacts/smoke.png` — screenshot when supported by the MCP server/toolset
- `artifacts/console.json` — best-effort browser console messages (when supported)

## Security notes

Treat browser automation like remote code execution:
- avoid real credentials in repos
- use test accounts + environment-injected secrets
- restrict allowed origins when running the MCP server in shared environments
