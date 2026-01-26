# Deployment (Community Cloud and beyond)

## Community Cloud basics

- Pin dependencies (requirements.txt or pyproject).
- Include `.streamlit/config.toml` where appropriate.
- Use secrets via the Cloud UI (do not commit secrets.toml).

## Docker (when needed)

Baseline requirements:
- expose port 8501
- set `server.headless=true`
- set `server.address=0.0.0.0`

## Smoke checks

- Start the app headless and hit `/` once.
- Run AppTest smoke tests in CI.
- For critical apps: run Playwright MCP E2E smoke in CI (headless).

