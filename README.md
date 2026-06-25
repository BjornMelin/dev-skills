# dev-skills

[![AgentSkills](https://img.shields.io/badge/AgentSkills-specification-24292f?style=flat-square)](https://agentskills.io/specification) [![skills.sh](https://img.shields.io/badge/registry-skills.sh-24292f?style=flat-square)](https://skills.sh/) [![Python](https://img.shields.io/badge/tooling-Python%203-3776AB?style=flat-square&logo=python&logoColor=white)](https://www.python.org/) [![Rust](https://img.shields.io/badge/tooling-Rust-000000?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)

A versioned collection of reusable **Agent Skills** (per the AgentSkills specification) that I use to make coding agents more reliable, consistent, and fast.

- Spec: <https://agentskills.io/specification>
- Skill registry: <https://skills.sh/>

## What Is In This Repo

This repo now contains skill packages and supporting tooling:

- reusable skills under `skills/`;
- retired skill source history under `archive/skills/`;
- reusable local Codex plugin source under `plugins/`;
- repo bootstrap pack manifests and templates under `bootstrap/`;
- a Rust shared development contract crate, `codex-dev-core`, under `crates/`;
- a Rust development CLI, `codex-dev`, under `crates/`;
- an optional Rust terminal workbench, `codex-dev-tui`, under `crates/`;
- a Rust research CLI, `codex-research`, under `crates/`;
- Codex subagent source packs under `subagents/`;
- tracked documentation under `docs/`;
- skill, bootstrap, docs, and eval helpers under `tools/`.

Start with [docs/index.md](docs/index.md) for the full guide set.

Key docs:

- [Onboarding](docs/guides/onboarding.md)
- [System overview](docs/architecture/overview.md)
- [Research architecture](docs/architecture/research-system.md)
- [codex-dev operating layer spec](docs/specs/codex-dev-operating-layer.md)
- [dev-skills v0.3/v1 roadmap](docs/specs/dev-skills-v0.3-roadmap.md)
- [Future local app surfaces](docs/specs/future-local-surfaces.md)
- [codex-dev core reference](docs/reference/codex-dev-core.md)
- [codex-dev CLI reference](docs/reference/codex-dev-cli.md)
- [codex-dev TUI reference](docs/reference/codex-dev-tui.md)
- [gsap-audit reference](docs/reference/gsap-audit.md)
- [expo-motion-audit reference](docs/reference/expo-motion-audit.md)
- [codex-research v0.2 follow-up spec](docs/specs/codex-research-v0.2.md)
- [codex-research CLI reference](docs/reference/codex-research-cli.md)
- [codex-research crate reference](docs/reference/codex-research-crate.md)
- [Global CLI workflow](docs/runbooks/global-cli-workflow.md)
- [Rust skill suite](docs/reference/rust-skill-suite.md)
- [Memory guidance proposals](docs/cookbooks/memory-guidance-proposals.md)
- [Codex prompt library](docs/prompts/codex-scenario-prompts.md)
- [Claude Code motion plugin install](docs/cookbooks/claude-code-motion-plugins.md)

For local live-provider testing, copy `.env.example` to an untracked `.env` and
export it in your shell before running provider commands.

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
archive/
  skills/
    <skill-name>/
      archive.json        # required archive metadata
      SKILL.md            # retained source; not active
.claude-plugin/
  marketplace.json        # Claude Code marketplace catalog
plugins/
  <plugin-name>/
    .codex-plugin/
      plugin.json         # Codex plugin manifest
    .claude-plugin/
      plugin.json         # Claude Code plugin manifest
    skills/
      <skill-name>/
        SKILL.md          # plugin-scoped skill entrypoint
crates/
  codex-dev-core/         # Shared task capsule contracts and read-model helpers
  codex-dev/              # Rust CLI for local task capsules, policy gates, and development evidence
  codex-dev-tui/          # Optional Ratatui workbench for codex-dev capsules
  codex-research/         # Rust CLI for evidence-first research helpers
  gsap-audit-core/        # oxc-based static-analysis engine for the gsap skill
  gsap-audit/             # Rust CLI that audits GSAP usage in JS/TS/JSX/TSX
  expo-motion-audit-core/ # oxc-based static-analysis engine for the expo-motion skill
  expo-motion-audit/      # Rust CLI that audits Expo/React Native motion (Reanimated) usage
docs/
  index.md                # documentation portal
  architecture/           # system design
  guides/                 # onboarding and setup
  reference/              # CLI, crate, skill, and template references
  cookbooks/              # operator-grade workflows
  prompts/                # copy-paste Codex prompts
  runbooks/               # validation, troubleshooting, maintenance
tools/
  bootstrap/              # repo bootstrap pack renderer
  eval/                   # offline skill/subagent eval runner
  docs/                   # documentation checks
  skill/                  # skill validation and packaging helpers
subagents/
  codex/                  # tracked global roles, public overlays, and sync helpers
```

## Research, Subagent, and Operating Stack

The main system combines research helpers, reusable subagents, and a development
operating layer:

- `deep-researcher`: skill for deep cited research with a Focused Six subagent
  pack.
- `codex-research`: Rust CLI for planning, provider routing, Context7 REST,
  GitHub REST, fetch probes, Firecrawl calls, evidence ledgers, reports,
  closeout bundles, cache, doctor, and evals.
- `codex-dev-core`: shared contract/read-model crate for task capsules,
  validation, rendered summaries, skill inventory, task index, orchestration run
  projections, policy manifest data, and PR evidence snapshots.
- `codex-dev`: current CLI for local task capsule lifecycle, structured
  evidence appenders, subspawn plan/outcome/synthesis capture, orchestration
  run verification, repo-native policy gates, read-only local workstation
  readiness checks, and PR evidence capture. It depends on `codex-dev-core` and
  keeps Clap parsing plus process execution at the CLI boundary.
- `codex-dev-tui`: optional Ratatui workbench that reads `codex-dev` capsule
  JSON contracts through `codex-dev-core`, including operator panels for skill
  health, task index, orchestration runs, PR-agent blockers, and next actions,
  without owning policy logic.
- `skill_subagent_eval.py`: offline eval lab for the full skill catalog,
  skill assets, OpenAI agent metadata, subagent templates, role contracts, and
  planner presets.
- `render_bootstrap_pack.py`: manifest-backed bootstrap packs for seeding new
  repos with agent guidance and validation docs.
- `subagent-creator`: helper skill and CLI for custom Codex agent templates.
- `subspawn`: strict subagent delegation policy with planner-generated prompts
  and mandatory wait-before-next-work synthesis.

Build and install the local CLIs from a trusted checkout:

```bash
cargo build -p codex-research
cargo build -p codex-dev
cargo build -p codex-dev-tui
cargo install --path crates/codex-research --locked --force
cargo install --path crates/codex-dev --locked --force
cargo install --path crates/codex-dev-tui --locked --force
codex-research --json doctor
codex-dev --help
codex-dev-tui --help
```

Generate shell completions and manpages from the installed binaries:

```bash
codex-research completions zsh > /tmp/_codex-research
codex-dev completions zsh > /tmp/_codex-dev
codex-dev-tui completions zsh > /tmp/_codex-dev-tui
codex-research manpage > /tmp/codex-research.1
codex-dev manpage > /tmp/codex-dev.1
codex-dev-tui manpage > /tmp/codex-dev-tui.1
```

Smoke the development CLI from source:

```bash
cargo build -p codex-dev
cargo build -p codex-dev-core
cargo build -p codex-dev-tui
cargo run -q -p codex-dev -- --help
# codex-dev:policy-manifest-smoke:start
cargo run -q -p codex-dev -- --json policy manifest --profile codex_dev
cargo run -q -p codex-dev -- --json policy explain --profile codex_dev
cargo run -q -p codex-dev -- --json policy manifest --profile full_local
cargo run -q -p codex-dev -- --json policy explain --profile full_local
# codex-dev:policy-manifest-smoke:end
cargo run -q -p codex-dev -- --json policy docs-check
cargo run -q -p codex-dev -- --json local doctor
cargo run -q -p codex-dev -- --json local status
cargo run -q -p codex-dev -- --json skills inventory
cargo run -q -p codex-dev -- --json skills sync-kimi --dry-run --project-root "$PWD"
cargo run -q -p codex-dev -- --json skills catalog --out /tmp/agent-skills-lab.json
cargo run -q -p codex-dev -- --json task list
cargo run -q -p codex-dev -- --json research import-bundle --help
cargo run -q -p codex-dev -- --json orchestration verify --help
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
cargo run -q -p codex-dev -- --json pr agent --help
cargo run -q -p codex-dev -- --json pr agent-action --help
cargo run -q -p codex-dev -- --json pr review --help
cargo run -q -p codex-dev -- --json pr readiness --help
cargo run -q -p codex-dev -- --json review --help
cargo run -q -p codex-dev -- --json commit plan --help
cargo run -q -p codex-dev -- --json commit validate --subject "fix(codex-dev): preserve review-thread closeout evidence"
```

Run the canonical task capsule fixture in
[Validation](docs/runbooks/validation.md) when changing orchestration
plan/record/close/verify behavior.

For release handoff and safe updates from any directory, use the
[Global CLI Workflow](docs/runbooks/global-cli-workflow.md) and
[Local Release and Supply Chain](docs/runbooks/local-release-supply-chain.md)
runbooks. `codex-dev-core` is a library crate, not an installed binary.

Preview a repo bootstrap pack:

```bash
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json bootstrap status
cargo run -q -p codex-dev -- --json bootstrap plan --pack codex-agent-repo --out "$tmp/codex" --repo-name codex-smoke
python3 tools/bootstrap/render_bootstrap_pack.py --list
python3 tools/bootstrap/render_bootstrap_pack.py --pack codex-agent-repo --out "$tmp/codex" --repo-name codex-smoke --dry-run
```

Install the deep research agents:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target global --dry-run
python3 skills/deep-researcher/scripts/install_agents.py --target global
```

For custom subagent template ownership, packaged fallback copies, duplicate-role
validation, and skill packaging rules, see
[Subagent Templates](docs/reference/subagent-templates.md).

## Skill catalog

Active skills are stored in `skills/`. The canonical entrypoint for each active
skill is its `SKILL.md`. Machine-readable inventory for automation is available
through `codex-dev --json skills inventory`; the public Agent Skills Lab
artifact is generated with
`codex-dev --json skills catalog --out catalog/agent-skills-lab.json`. Retired
skills belong in `archive/skills/` with an `archive.json` manifest and must not
remain linked from this active catalog. The table below remains the human-facing
active catalog.

### Local Plugin Skill Bundles

Install the web-motion plugin in Claude Code:

```bash
claude plugin marketplace add BjornMelin/dev-skills --sparse .claude-plugin plugins/web-motion plugins/claude-core
claude plugin install web-motion@bjorn-dev-skills
```

(The former `native-motion` plugin was consolidated into the standalone `expo-motion` skill — install it with `skills add BjornMelin/dev-skills -g -s expo-motion`.)

After installing inside an active Claude Code session, run `/reload-plugins`.
See the [Claude Code motion plugin install cookbook](docs/cookbooks/claude-code-motion-plugins.md)
for local development and validation commands.

| Plugin | Skills | Description | Sources |
| --- | --- | --- | --- |
| `web-motion` | `typegpu`, `web-css-animations`, `web-lottie`, `web-motion-react`, `web-rive`, `web-tailwind-motion`, `web-three-r3f`, `web-waapi` | Self-contained web motion skills with TypeGPU, Motion React, CSS, WAAPI, Tailwind, Lottie, Three.js/R3F, and Rive references. | [Codex](plugins/web-motion/.codex-plugin/plugin.json), [Claude Code](plugins/web-motion/.claude-plugin/plugin.json) |

| Skill | Description | Source |
| --- | --- | --- |
| `agents-md-maintainer` | Durable `AGENTS.md` maintenance rules for deciding when repo guidance should change after implementation. | [skills/agents-md-maintainer/SKILL.md](skills/agents-md-maintainer/SKILL.md) |
| `ai-sdk-agents` | AI SDK v6+ agents with `ToolLoopAgent`: loop control, dynamic tools, and agent workflows. | [skills/ai-sdk-agents/SKILL.md](skills/ai-sdk-agents/SKILL.md) |
| `ai-sdk-core` | AI SDK Core: text/structured output, tools, MCP, embeddings/reranking, middleware, telemetry. | [skills/ai-sdk-core/SKILL.md](skills/ai-sdk-core/SKILL.md) |
| `ai-sdk-ui` | Chat and generative UIs with AI SDK React (`useChat`, persistence, streaming, backends). | [skills/ai-sdk-ui/SKILL.md](skills/ai-sdk-ui/SKILL.md) |
| `autoreview` | Codex-only structured closeout review helper for local, branch, or commit diffs. | [skills/autoreview/SKILL.md](skills/autoreview/SKILL.md) |
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
| `deep-researcher` | Deep cited research across Codex web, Context7 API, GitHub, source, rendered pages, Firecrawl, and evidence ledgers. | [skills/deep-researcher/SKILL.md](skills/deep-researcher/SKILL.md) |
| `dmc-best-practices` | DMC + Dash best practices: architecture, callbacks, styling, performance, theming. | [skills/dmc-best-practices/SKILL.md](skills/dmc-best-practices/SKILL.md) |
| `dmc-py` | Dash Mantine Components v2.x: theming, callbacks (pattern-matching, clientside), pages, charts, components. | [skills/dmc-py/SKILL.md](skills/dmc-py/SKILL.md) |
| `docker-architect` | Docker/Compose: Dockerfiles, Compose, CI, security hardening, audits. | [skills/docker-architect/SKILL.md](skills/docker-architect/SKILL.md) |
| `docs-align` | Post-implement docs alignment: drift detection, ADRs, specs, README, `AGENTS.md`. | [skills/docs-align/SKILL.md](skills/docs-align/SKILL.md) |
| `expo-motion` | Master Expo/React Native motion skill for iOS/Android: Reanimated 4, worklets, gestures, layout animations, scroll, Expo Router/native-stack transitions, NativeWind boundaries, accessibility/performance, React Native Skia, and validation — with an Expo/RN recipe cookbook and the `expo-motion-audit` CLI. | [skills/expo-motion/SKILL.md](skills/expo-motion/SKILL.md) |
| `firecrawl` | Firecrawl CLI for cache-aware search, scrape, map, crawl, interact, monitor, research, download, diagnostics, feedback, and document parse tasks. | [skills/firecrawl/SKILL.md](skills/firecrawl/SKILL.md) |
| `gh-pr-review-fix` | Resolve GitHub PR review threads end-to-end through `codex-dev pr review` with verified fixes, semantic commits, push, and hosted closeout (not local review files). | [skills/gh-pr-review-fix/SKILL.md](skills/gh-pr-review-fix/SKILL.md) |
| `grill-me` | Stress-test a plan or design with exhaustive Q&A until the decision tree is clear. | [skills/grill-me/SKILL.md](skills/grill-me/SKILL.md) |
| `gsap` | Master GSAP skill for React/Next.js and vanilla JS: tweens, timelines, ScrollTrigger, the `useGSAP` hook, every free plugin (SplitText, MorphSVG, DrawSVG, Flip, Draggable), `gsap.utils`, performance, and reduced-motion — with a Next.js recipe cookbook and the `gsap-audit` CLI. | [skills/gsap/SKILL.md](skills/gsap/SKILL.md) |
| `kimi-ui-agent` | Explicit-only Kimi-powered UI agent for repo profiling, adapter setup, and plan-first frontend worktree orchestration. | [skills/kimi-ui-agent/SKILL.md](skills/kimi-ui-agent/SKILL.md) |
| `langgraph-multiagent` | LangGraph/LangChain multi-agent: supervisors, handoffs, RAG, memory, guardrails, migrations. | [skills/langgraph-multiagent/SKILL.md](skills/langgraph-multiagent/SKILL.md) |
| `new-branch` | Create a conventional, semver-friendly branch first, then plan work and the PR. | [skills/new-branch/SKILL.md](skills/new-branch/SKILL.md) |
| `notebook-ml-architect` | ML notebooks: leakage, reproducibility, refactor, modular pipelines, notebook→script. | [skills/notebook-ml-architect/SKILL.md](skills/notebook-ml-architect/SKILL.md) |
| `opensrc` | Canonical source-level dependency inspection with the `opensrc` CLI for package internals, version diffs, and upgrade audits. | [skills/opensrc/SKILL.md](skills/opensrc/SKILL.md) |
| `platform-architect` | Full-stack/native across Next.js, Expo, Convex, monorepos: detection, planning, repo verification. | [skills/platform-architect/SKILL.md](skills/platform-architect/SKILL.md) |
| `pytest-dev` | pytest: fixtures, flakes, coverage, speed, CI sharding and tuning. | [skills/pytest-dev/SKILL.md](skills/pytest-dev/SKILL.md) |
| `repo-context-builder` | Build `REPO_CONTEXT.md` and `REVIEW_BRIEF.md` artifacts for grounded future handoffs. | [skills/repo-context-builder/SKILL.md](skills/repo-context-builder/SKILL.md) |
| `repo-docs-align` | Sync all repo docs to code and workflow across stacks (`AGENTS.md`, ADRs, runbooks, etc.). | [skills/repo-docs-align/SKILL.md](skills/repo-docs-align/SKILL.md) |
| `repo-modernizer` | Repo and monorepo dependency modernization, vulnerability remediation, and framework-aware upgrade audits. | [skills/repo-modernizer/SKILL.md](skills/repo-modernizer/SKILL.md) |
| `review-remediation` | Fix local review notes with verify-first triage; excludes hosted GitHub PR review loops. | [skills/review-remediation/SKILL.md](skills/review-remediation/SKILL.md) |
| `rust-cli-clap` | Rust CLI and Clap command design: parsers, output contracts, tests, packaging. | [skills/rust-cli-clap/SKILL.md](skills/rust-cli-clap/SKILL.md) |
| `rust-expert` | Core Rust engineering router for ownership, async, crates, tests, performance, and security. | [skills/rust-expert/SKILL.md](skills/rust-expert/SKILL.md) |
| `rust-mega-eng` | Explicit Rust architecture orchestrator for broad multi-crate strategy and release planning. | [skills/rust-mega-eng/SKILL.md](skills/rust-mega-eng/SKILL.md) |
| `rust-tauri-apps` | Tauri v2 Rust app backends: commands, secure IPC, capabilities, bundling, distribution. | [skills/rust-tauri-apps/SKILL.md](skills/rust-tauri-apps/SKILL.md) |
| `rust-tui-ratatui` | Rust terminal UI architecture with Ratatui, crossterm event loops, snapshots, and UX. | [skills/rust-tui-ratatui/SKILL.md](skills/rust-tui-ratatui/SKILL.md) |
| `rust-web-services` | Production Rust HTTP services with Axum, Tokio, Tower, SQLx, tracing, and shutdown. | [skills/rust-web-services/SKILL.md](skills/rust-web-services/SKILL.md) |
| `ship-branch` | Semantic commits, push, and open a PR to `main` with conventional title and body. | [skills/ship-branch/SKILL.md](skills/ship-branch/SKILL.md) |
| `sentry-cli-fix-issues` | Fix Sentry issues from CLI evidence: issues, events, traces, logs, replays, Seer, privacy, and verification. | [skills/sentry-cli-fix-issues/SKILL.md](skills/sentry-cli-fix-issues/SKILL.md) |
| `sentry-triage-to-pr` | Rank unresolved Sentry issues, group PR-sized fixes, render GitHub issue plans, and plan subspawn worktrees. | [skills/sentry-triage-to-pr/SKILL.md](skills/sentry-triage-to-pr/SKILL.md) |
| `streamdown` | Streamdown: streaming markdown for AI UIs, Shiki/KaTeX/Mermaid, remend, hardening. | [skills/streamdown/SKILL.md](skills/streamdown/SKILL.md) |
| `streamlit-master-architect` | Streamlit: reruns/state, caching/fragments, AppTest, components v2, security, Playwright E2E. | [skills/streamlit-master-architect/SKILL.md](skills/streamlit-master-architect/SKILL.md) |
| `subagent-creator` | Create, validate, install, diff, sync, and smoke-test Codex custom subagent TOML role packs. | [skills/subagent-creator/SKILL.md](skills/subagent-creator/SKILL.md) |
| `subspawn` | Bounded Codex subagent delegation with strict wait and evidence-first synthesis. | [skills/subspawn/SKILL.md](skills/subspawn/SKILL.md) |
| `supabase-ts` | Supabase with Next/React/TS: SSR auth, RLS, storage, realtime, Edge Functions, pgvector, deploy. | [skills/supabase-ts/SKILL.md](skills/supabase-ts/SKILL.md) |
| `taste-metaskill` | Route frontend UI work to focused visual-taste references (premium, distinct, anti-generic). | [skills/taste-metaskill/SKILL.md](skills/taste-metaskill/SKILL.md) |
| `technical-writing` | Internal engineering docs: PRDs, specs, architecture, runbooks, migrations, and maintainer guides. | [skills/technical-writing/SKILL.md](skills/technical-writing/SKILL.md) |
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

Build the Rust CLIs:

```bash
cargo build -p codex-dev
cargo build -p codex-dev-core
cargo build -p codex-dev-tui
cargo build -p codex-research
```

Use [docs/runbooks/validation.md](docs/runbooks/validation.md) for the
canonical validation matrix and
[docs/runbooks/local-release-supply-chain.md](docs/runbooks/local-release-supply-chain.md)
for the audited local install and release baseline. Use
[docs/reference/distribution-surface-gates.md](docs/reference/distribution-surface-gates.md)
before opening crates.io publication, signed-binary, cargo-vet, Tauri desktop,
or Axum local-service implementation work. README intentionally stays a portal
so command lists do not drift from the runbooks. Use
[docs/reference/subagent-templates.md](docs/reference/subagent-templates.md) for
the subagent template authority model and duplicate-role expectations.

Rust skill suite validation:

```bash
node skills/rust-expert/scripts/check-reference-links.mjs skills/rust-expert skills/rust-cli-clap skills/rust-tui-ratatui skills/rust-tauri-apps skills/rust-web-services skills/rust-mega-eng
node skills/rust-expert/scripts/check-trigger-evals.mjs skills/rust-expert skills/rust-cli-clap skills/rust-tui-ratatui skills/rust-tauri-apps skills/rust-web-services skills/rust-mega-eng
```

Notes:

- A `.skill` file is a ZIP archive containing a `<skill-name>/...` folder.
  `skills/<skill-name>/...` is the source-tree path used to create it.
- Bundles are treated as build artifacts here (gitignored) and are intended to
  be published via release assets / registries.

## Contributing (to this repo)

- Keep the skill entrypoint in `skills/<skill-name>/SKILL.md`.
- Put long-form material in `references/` and executable helpers in `scripts/`.
- Keep frontmatter minimal and spec-compatible (the validator enforces allowed keys).

## License

This repository may contain skills under different licenses. Check each skill directory for license files (for example `skills/vitest-dev/LICENSE`) and follow the terms for that specific skill.
