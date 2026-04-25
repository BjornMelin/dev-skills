# dev-skills

[![AgentSkills](https://img.shields.io/badge/AgentSkills-specification-24292f?style=flat-square)](https://agentskills.io/specification) [![skills.sh](https://img.shields.io/badge/registry-skills.sh-24292f?style=flat-square)](https://skills.sh/) [![Python](https://img.shields.io/badge/tooling-Python%203-3776AB?style=flat-square&logo=python&logoColor=white)](https://www.python.org/)

A versioned collection of reusable **Agent Skills** (per the AgentSkills specification) that I use to make coding agents more reliable, consistent, and fast.

- Spec: <https://agentskills.io/specification>
- Skill registry: <https://skills.sh/>

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
| `ai-sdk-agents` | AI SDK v6+ agents with `ToolLoopAgent`: loop control, dynamic tools, and agent workflows. | [skills/ai-sdk-agents/SKILL.md](skills/ai-sdk-agents/SKILL.md) |
| `ai-sdk-core` | AI SDK Core: text/structured output, tools, MCP, embeddings/reranking, middleware, telemetry. | [skills/ai-sdk-core/SKILL.md](skills/ai-sdk-core/SKILL.md) |
| `ai-sdk-ui` | Chat and generative UIs with AI SDK React (`useChat`, persistence, streaming, backends). | [skills/ai-sdk-ui/SKILL.md](skills/ai-sdk-ui/SKILL.md) |
| `aws-architecture` | AWS architecture: service selection, boundaries, rollout, and verification. | [skills/aws-architecture/SKILL.md](skills/aws-architecture/SKILL.md) |
| `browser-workbench-setup` | Bootstrap browser/UI QA: Playwright-interactive primary, agent-browser smoke, auth and local conventions. | [skills/browser-workbench-setup/SKILL.md](skills/browser-workbench-setup/SKILL.md) |
| `bun-audit` | Bun-first audit/remediation router (delegates to the shared engine in `bun-dev`). | [skills/bun-audit/SKILL.md](skills/bun-audit/SKILL.md) |
| `bun-dev` | Bun development and runtime: PM, lockfiles, monorepos, tests/build, TypeScript, Vercel Bun runtime. | [skills/bun-dev/SKILL.md](skills/bun-dev/SKILL.md) |
| `caveman-compress` | Compress docs and prose for fewer tokens while keeping substance, code, URLs, and structure. | [skills/caveman-compress/SKILL.md](skills/caveman-compress/SKILL.md) |
| `codex-sdk` | Codex SDK/CLI: JSONL threads, MCP, multi-agent orchestration, SQLite memory, sandbox patterns. | [skills/codex-sdk/SKILL.md](skills/codex-sdk/SKILL.md) |
| `codex-utils` | Codex session utilities: plans, short MCQs before risky edits, images, web/MCP discovery, parallel tool use. | [skills/codex-utils/SKILL.md](skills/codex-utils/SKILL.md) |
| `commit` | Stage and commit in semantic, reviewable groups. | [skills/commit/SKILL.md](skills/commit/SKILL.md) |
| `context7-research` | Library/API documentation research via Context7 MCP (version-specific, migrations, primary sources). | [skills/context7-research/SKILL.md](skills/context7-research/SKILL.md) |
| `convex-audit` | Audit Convex backends: schema, security, runtime edges, migrations, function-surface risk. | [skills/convex-audit/SKILL.md](skills/convex-audit/SKILL.md) |
| `convex-best-practices` | Production Convex: organization, queries, validation, TypeScript, errors, design philosophy. | [skills/convex-best-practices/SKILL.md](skills/convex-best-practices/SKILL.md) |
| `convex-component-adoption-planner` | Research Convex components vs a live graph; scored Q&A; adoption or rejection packs (`PLAN.md`, prompts). | [skills/convex-component-adoption-planner/SKILL.md](skills/convex-component-adoption-planner/SKILL.md) |
| `convex-feature-spec` | Convex-first feature specs: model, API, rollout, verification (not implementation audits). | [skills/convex-feature-spec/SKILL.md](skills/convex-feature-spec/SKILL.md) |
| `dash-audit` | Audit Dash apps: callbacks, state, layout, accessibility, Dash-specific UX. | [skills/dash-audit/SKILL.md](skills/dash-audit/SKILL.md) |
| `dmc-best-practices` | DMC + Dash best practices: architecture, callbacks, styling, performance, theming. | [skills/dmc-best-practices/SKILL.md](skills/dmc-best-practices/SKILL.md) |
| `dmc-py` | Dash Mantine Components v2.x: theming, callbacks (pattern-matching, clientside), pages, charts, components. | [skills/dmc-py/SKILL.md](skills/dmc-py/SKILL.md) |
| `docker-architect` | Docker/Compose: Dockerfiles, Compose, CI, security hardening, audits. | [skills/docker-architect/SKILL.md](skills/docker-architect/SKILL.md) |
| `docs-align` | Post-implement docs alignment: drift detection, ADRs, specs, README, `AGENTS.md`. | [skills/docs-align/SKILL.md](skills/docs-align/SKILL.md) |
| `gh-deps-intel` | JS/TS + Python dependency intel for monorepos: outdated checks, releases/changelogs → Markdown + JSON. | [skills/gh-deps-intel/SKILL.md](skills/gh-deps-intel/SKILL.md) |
| `gh-pr-review-fix` | Resolve GitHub PR review threads end-to-end with minimal verified fixes (not local review files). | [skills/gh-pr-review-fix/SKILL.md](skills/gh-pr-review-fix/SKILL.md) |
| `grill-me` | Stress-test a plan or design with exhaustive Q&A until the decision tree is clear. | [skills/grill-me/SKILL.md](skills/grill-me/SKILL.md) |
| `langgraph-multiagent` | LangGraph/LangChain multi-agent: supervisors, handoffs, RAG, memory, guardrails, migrations. | [skills/langgraph-multiagent/SKILL.md](skills/langgraph-multiagent/SKILL.md) |
| `new-branch` | Create a conventional, semver-friendly branch first, then plan work and the PR. | [skills/new-branch/SKILL.md](skills/new-branch/SKILL.md) |
| `notebook-ml-architect` | ML notebooks: leakage, reproducibility, refactor, modular pipelines, notebook→script. | [skills/notebook-ml-architect/SKILL.md](skills/notebook-ml-architect/SKILL.md) |
| `opensrc-inspect` | Local `opensrc` snapshots for dependency/upstream source inspection (not general web research). | [skills/opensrc-inspect/SKILL.md](skills/opensrc-inspect/SKILL.md) |
| `platform-architect` | Full-stack/native across Next.js, Expo, Convex, monorepos: detection, planning, repo verification. | [skills/platform-architect/SKILL.md](skills/platform-architect/SKILL.md) |
| `pytest-dev` | pytest: fixtures, flakes, coverage, speed, CI sharding and tuning. | [skills/pytest-dev/SKILL.md](skills/pytest-dev/SKILL.md) |
| `repo-docs-align` | Sync all repo docs to code and workflow across stacks (`AGENTS.md`, ADRs, runbooks, etc.). | [skills/repo-docs-align/SKILL.md](skills/repo-docs-align/SKILL.md) |
| `ship-branch` | Semantic commits, push, and open a PR to `main` with conventional title and body. | [skills/ship-branch/SKILL.md](skills/ship-branch/SKILL.md) |
| `signr-pr-closure-loop` | Signr-style PR closure: review threads, CI, Expo/EAS, Vercel/Turborepo, docs, babysit to merge-ready. | [skills/signr-pr-closure-loop/SKILL.md](skills/signr-pr-closure-loop/SKILL.md) |
| `streamdown` | Streamdown: streaming markdown for AI UIs, Shiki/KaTeX/Mermaid, remend, hardening. | [skills/streamdown/SKILL.md](skills/streamdown/SKILL.md) |
| `streamlit-master-architect` | Streamlit: reruns/state, caching/fragments, AppTest, components v2, security, Playwright E2E. | [skills/streamlit-master-architect/SKILL.md](skills/streamlit-master-architect/SKILL.md) |
| `subspawn` | Bounded Codex subagent delegation and synthesis (`spawn_agent`, scopes, evidence-first merge). | [skills/subspawn/SKILL.md](skills/subspawn/SKILL.md) |
| `supabase-ts` | Supabase with Next/React/TS: SSR auth, RLS, storage, realtime, Edge Functions, pgvector, deploy. | [skills/supabase-ts/SKILL.md](skills/supabase-ts/SKILL.md) |
| `taste-metaskill` | Route frontend UI work to focused visual-taste references (premium, distinct, anti-generic). | [skills/taste-metaskill/SKILL.md](skills/taste-metaskill/SKILL.md) |
| `upgrade-pack-generator` | Repo-local upgrade packs under `.agents/plans/upgrade/` (playbook, prompts, manifest). | [skills/upgrade-pack-generator/SKILL.md](skills/upgrade-pack-generator/SKILL.md) |
| `vitest-dev` | Vitest for TypeScript + Next.js: stable suites, fast local + CI, sharding. | [skills/vitest-dev/SKILL.md](skills/vitest-dev/SKILL.md) |
| `zod-v4` | Zod v4: schemas, v3 migration, errors, JSON Schema/OpenAPI, framework hooks. | [skills/zod-v4/SKILL.md](skills/zod-v4/SKILL.md) |

## Using these skills

How you “install” a skill depends on your agent runtime. In general, you need the skill folder available on disk so the runtime can read `SKILL.md` and (optionally) bundled resources.

Common approaches:

- **From source:** copy or symlink `skills/<skill-name>/` into your runtime’s skills directory.
- **From a bundle:** use a `.skill` bundle (ZIP) and install it using your runtime/registry tooling (for example via <https://skills.sh/>).

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
