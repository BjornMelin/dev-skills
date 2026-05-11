# Onboarding

This guide gets a new contributor or Codex session from a fresh checkout to a
validated research/subagent workflow.

## Repository Shape

Important paths:

```text
Cargo.toml                         # Cargo workspace
crates/codex-research/             # Rust research CLI
skills/deep-researcher/            # Deep research skill and Focused Six agents
skills/subagent-creator/           # Custom subagent authoring skill and helper
skills/subspawn/                   # Subagent delegation policy
docs/                              # Handwritten tracked documentation
tools/skill/                       # Skill validation and packaging helpers
```

Build output (`target/`) and skill bundles (`skills/dist/`) are ignored.

## Prerequisites

Required:

- `python3`
- Rust toolchain with `cargo`
- Codex runtime that can read `SKILL.md` files

Recommended for full research capability:

- `gh` authenticated for GitHub REST fallback
- `agent-browser` for local rendered-page extraction
- `opensrc` for dependency source inspection
- `CONTEXT7_API_KEY` for direct Context7 API calls
- `FIRECRAWL_API_KEY` for Firecrawl scrape/crawl fallback
- `EXA_API_KEY` for Exa MCP usage in Codex sessions

Local provider secrets should live in an untracked root `.env` copied from
`.env.example`:

```bash
cp .env.example .env
$EDITOR .env
set -a; source .env; set +a
```

The CLI reads process environment variables; it does not auto-load `.env`.
Use `gh auth login` instead of `GITHUB_TOKEN` when possible. Local
`.codex/research` run ledgers and reports are ignored because they can contain
source excerpts or private research context.

Inspect local readiness:

```bash
codex-research doctor
codex-research --json doctor
```

## Build and Install the CLI

From the repository root:

```bash
cargo build -p codex-research
cargo install --path crates/codex-research --locked --force
codex-research --help
```

For installing or updating every local Rust CLI, use the
[Local Release and Supply Chain](../runbooks/local-release-supply-chain.md)
runbook.

If you do not install the binary, the skill wrapper can run it from this repo:

```bash
skills/deep-researcher/scripts/codex-research doctor
```

## Install Deep Research Agents

Project-scoped install:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target project --dry-run
python3 skills/deep-researcher/scripts/install_agents.py --target project
```

Global install:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target global --dry-run
python3 skills/deep-researcher/scripts/install_agents.py --target global
```

Validate installed templates:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate ~/.codex/agents
```

## First Research Workflow

1. Create a plan:

   ```bash
   codex-research plan "latest official guidance for Codex custom subagents" --profile standard
   ```

2. Use native Codex web tools for current official facts.

3. Use provider-specific CLI lanes only when needed:

   ```bash
   codex-research context7 search --library "Turborepo" --query "package configurations and task graph"
   codex-research github search-repos "openai codex in:name" --per-page 3
   codex-research fetch probe "https://docs.github.com/en/rest/search/search"
   ```

4. Record evidence:

   ```bash
   codex-research ledger init
   codex-research ledger add-source --provider github --url https://github.com/openai/codex --title "openai/codex" --route github
   codex-research ledger add-claim --text "Hydrated GitHub sources should be preferred over scraped GitHub HTML." --confidence 0.9 --source <source-id>
   codex-research report --out .codex/research/report.md
   ```

5. Run validation before committing:

   ```bash
   cargo fmt --all --check
   cargo test -p codex-research
   for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
   python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts
   git diff --check
   ```

## Mental Model

- Skills tell Codex when and how to operate.
- Custom agent TOML files define reusable specialist roles.
- `$subspawn` governs delegation and forces wait-before-synthesis behavior.
- `codex-research` makes research evidence replayable and auditable.
- The main Codex session owns final synthesis and decisions.
