# dev-skills

A versioned collection of reusable **Agent Skills** (per the AgentSkills specification) that I use to make coding agents more reliable, consistent, and fast.

- Spec: https://agentskills.io/specification
- Skill registry: https://skills.sh/

Each skill lives in `skills/<skill-name>/` and is designed to be:

- **Self-contained** (instructions + optional scripts/assets)
- **Discoverable** (clear metadata + predictable layout)
- **Packagable** (can be shipped as a `.skill` ZIP archive)

## Repository layout

```
skills/
  <skill-name>/
    SKILL.md              # required (YAML frontmatter + instructions)
    references/           # optional (docs to load on demand)
    scripts/              # optional (deterministic helpers)
    assets/               # optional (templates/snippets)
    templates/            # optional (scaffolds)
  dist/                   # local .skill bundles (ZIP; gitignored)
```

## Skill catalog

All skills are stored in `skills/`. The canonical entrypoint for each skill is its `SKILL.md`.

| Skill | Description | Source |
| --- | --- | --- |
| `ai-sdk-agents` | Expert guidance for building AI agents with `ToolLoopAgent` (AI SDK v6+): loop control, tool selection, workflows. | [skills/ai-sdk-agents/SKILL.md](skills/ai-sdk-agents/SKILL.md) |
| `ai-sdk-core` | AI SDK Core patterns: text/structured output, tools, MCP integration, embeddings/reranking, middleware, production setup. | [skills/ai-sdk-core/SKILL.md](skills/ai-sdk-core/SKILL.md) |
| `ai-sdk-ui` | Build chat & generative UIs with AI SDK React hooks: `useChat`, tool UIs, persistence, streaming, backend integration. | [skills/ai-sdk-ui/SKILL.md](skills/ai-sdk-ui/SKILL.md) |
| `codex-sdk` | Architect-level workflows for OpenAI Codex SDK/CLI: JSONL threads, automation, MCP, multi-agent orchestration, SQLite memory. | [skills/codex-sdk/SKILL.md](skills/codex-sdk/SKILL.md) |
| `dmc-py` | Dash Mantine Components (DMC) v2.4.0: theming, callbacks (incl. pattern-matching), pages, charts, and component patterns. | [skills/dmc-py/SKILL.md](skills/dmc-py/SKILL.md) |
| `docker-architect` | Docker/Compose architecture + security hardening: Dockerfiles, Compose patterns, CI pipelines, least-privilege, audits. | [skills/docker-architect/SKILL.md](skills/docker-architect/SKILL.md) |
| `langgraph-multiagent` | Multi-agent systems with LangGraph/LangChain: supervisor/subagent patterns, handoffs, agentic RAG, memory, guardrails, migrations. | [skills/langgraph-multiagent/SKILL.md](skills/langgraph-multiagent/SKILL.md) |
| `notebook-ml-architect` | Audit/refactor production-quality ML Jupyter notebooks: leakage checks, reproducibility, modularization, notebook→script conversion. | [skills/notebook-ml-architect/SKILL.md](skills/notebook-ml-architect/SKILL.md) |
| `pytest-dev` | pytest test-engineering: fixtures/markers, flake fixes, coverage, suite speedups, and CI optimization/sharding. | [skills/pytest-dev/SKILL.md](skills/pytest-dev/SKILL.md) |
| `repo-docs-align` | Align `AGENTS.md`, README, ADRs, specs, runbooks, comments, and exec-plan artifacts with current repo reality and branch changes. | [skills/repo-docs-align/SKILL.md](skills/repo-docs-align/SKILL.md) |
| `streamdown` | Vercel Streamdown: streaming markdown rendering for AI apps, Shiki/KaTeX/Mermaid, remend, and output hardening. | [skills/streamdown/SKILL.md](skills/streamdown/SKILL.md) |
| `streamlit-master-architect` | Streamlit architecture + testing + deployment: state/reruns, caching/fragments, AppTest, components v2, security, Playwright MCP. | [skills/streamlit-master-architect/SKILL.md](skills/streamlit-master-architect/SKILL.md) |
| `supabase-ts` | Supabase + Next.js/React/TS: SSR auth, RLS, storage, realtime, edge functions, pgvector, CLI/typegen, deployment patterns. | [skills/supabase-ts/SKILL.md](skills/supabase-ts/SKILL.md) |
| `taste-metaskill` | Route frontend UI work to focused visual taste references for premium, distinct, anti-generic design output. | [skills/taste-metaskill/SKILL.md](skills/taste-metaskill/SKILL.md) |
| `vitest-dev` | Vitest test-engineering for TypeScript + Next.js: low-flake suites, fast local DX, CI throughput, sharding, reporting. | [skills/vitest-dev/SKILL.md](skills/vitest-dev/SKILL.md) |
| `zod-v4` | Zod v4 patterns: schema design, migration from v3, error handling, JSON schema/OpenAPI, and framework integrations. | [skills/zod-v4/SKILL.md](skills/zod-v4/SKILL.md) |

## Using these skills

How you “install” a skill depends on your agent runtime. In general, you need the skill folder available on disk so the runtime can read `SKILL.md` and (optionally) bundled resources.

Common approaches:

- **From source:** copy or symlink `skills/<skill-name>/` into your runtime’s skills directory.
- **From a bundle:** use a `.skill` bundle (ZIP) and install it using your runtime/registry tooling (for example via https://skills.sh/).

## Validating and packaging

This repo includes lightweight validation and packaging helpers under `tools/skill/`.

Validate a skill directory:

```bash
python3 tools/skill/quick_validate.py skills/<skill-name>
```

Package a skill to `skills/dist/<skill-name>.skill`:

```bash
python3 tools/skill/package_skill.py skills/<skill-name> skills/dist
```

Notes:

- A `.skill` file is a ZIP archive containing the `skills/<skill-name>/...` folder.
- Bundles are treated as build artifacts here (gitignored) and are intended to be published via release assets / registries.

## Contributing (to this repo)

- Keep the skill entrypoint in `skills/<skill-name>/SKILL.md`.
- Put long-form material in `references/` and executable helpers in `scripts/`.
- Keep frontmatter minimal and spec-compatible (the validator enforces allowed keys).

## License

This repository may contain skills under different licenses. Check each skill directory for license files (for example `skills/vitest-dev/LICENSE`) and follow the terms for that specific skill.
