# Repository Guidelines

This repository is a catalog of **Agent Skills** (per the AgentSkills specification) plus supporting tooling for reusable Codex workflows. Contributions should keep skills spec-compliant, self-contained, easy to install, and documented in `docs/` when they change public behavior.

## Project Structure & Module Organization

- `skills/<skill-name>/SKILL.md`: canonical entrypoint (required) with YAML frontmatter + instructions.
- `skills/<skill-name>/references/`: optional long-form docs to load on demand.
- `skills/<skill-name>/scripts/`: optional helper scripts (prefer deterministic tooling here).
- `skills/<skill-name>/assets/` / `templates/`: optional reusable artifacts.
- `skills/<skill-name>/agents/`: optional agent-runtime metadata (for example OpenAI YAML).
- `skills/dist/`: prebuilt `.skill` bundles (ZIP archives) for selected skills.
- `crates/codex-research/`: Rust CLI for evidence-first research helpers.
- `docs/`: tracked documentation portal, references, cookbooks, prompts, and runbooks.

Example skill path: `skills/docker-architect/SKILL.md`.

## Build, Test, and Development Commands

- Validate a skill (frontmatter/spec checks):  
  `python3 tools/skill/quick_validate.py skills/<skill-name>`
- Validate all skills:  
  `for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done`
- Package a `.skill` bundle:  
  `python3 tools/skill/package_skill.py skills/<skill-name> skills/dist`
- Optional sanity check for Python scripts:  
  `python3 -m compileall -q skills`
- Build/check the research CLI:
  `cargo check -p codex-research`
- Run the research CLI smoke checks:
  `codex-research --json doctor && codex-research --json eval`

## Coding Style & Naming Conventions

- Skill names must be **hyphen-case** and match the folder name (e.g. `langgraph-multiagent`).
- `SKILL.md` frontmatter should only use allowed keys: `name`, `description`, `license`, `allowed-tools`, `metadata`.
- Keep `SKILL.md` concise; put large content in `references/`. Prefer scripts over massive inline code blocks.
- Custom subagent TOML names must be **snake_case** and must not shadow Codex built-ins (`default`, `worker`, `explorer`) unless explicitly requested.
- Keep generated Rust docs and `target/` out of git; document Rust APIs by updating `docs/reference/codex-research-crate.md`.

## Testing Guidelines

There is no single repo-wide test harness. Treat the following as the required gates based on touched files:

- Any skill: `python3 tools/skill/quick_validate.py skills/<skill-name>`
- All skills: `for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done`
- Python helpers: `python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts`
- Custom agent templates: `python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents`
- Rust CLI: `cargo fmt --all --check`, `cargo clippy -p codex-research --all-targets -- -D warnings`, `cargo check -p codex-research`, `cargo test -p codex-research`
- CLI smoke: `codex-research --json doctor`, `codex-research --json eval`
- Docs links: `python3 tools/docs/check_links.py docs README.md AGENTS.md`
- Final whitespace check: `git diff --check`

If you add scripts, keep them runnable without external secrets and avoid network calls unless the skill explicitly requires it. Live provider checks should be optional.

## Documentation Guidelines

- `docs/index.md` is the documentation portal.
- Update docs when changing CLI commands, data models, skill behavior, subagent templates, validation rules, or install workflow.
- Keep README as a portal and catalog, not a full manual.
- Keep AGENTS.md focused on contributor and agent operating rules.
- Put command references in `docs/reference/`, workflows in `docs/cookbooks/`, prompts in `docs/prompts/`, and validation/troubleshooting in `docs/runbooks/`.
- Do not track generated rustdoc, `target/`, provider dumps with private data, or run-specific `.codex/research/` artifacts unless explicitly requested.

## Research/Subagent Stack Rules

- Use `skills/deep-researcher` for deep cited research workflows.
- Use `codex-research` for provider planning, Context7 REST, GitHub REST, direct fetch probes, Firecrawl calls, evidence ledgers, reports, cache, doctor, and evals.
- Use native Codex web tools for current official facts; `codex-research` records provider evidence and handles external calls it owns directly.
- Use `skills/subagent-creator/scripts/subagent_creator.py` to validate or install custom agent templates.
- Use `skills/subspawn` when spawning agents. After spawning a planned batch, wait for every spawned subagent before substantive next work or final synthesis.

## Commit & Pull Request Guidelines

This repo may not have established git history conventions yet. Use clear, scoped commits (recommended: Conventional Commits), e.g. `feat(dmc-py): add callbacks scaffold` or `docs: expand README catalog`. PRs should include:

- What changed + why
- Validation commands run (at minimum `python3 tools/skill/quick_validate.py skills/<skill-name>`)
- If you add or rename a skill, update the catalog table in `README.md` (keep rows sorted by skill name)
- If you add or materially change docs, update `docs/index.md`
- If you change `codex-research`, update the CLI and crate references under `docs/reference/`
- If you built/published bundles, say where (release assets/registry)

## Security & Configuration Tips

Do not commit credentials or private tokens (use `.env.example` patterns). Assume any `.skill` bundle is redistributable; keep sensitive material out of skill folders and packaged artifacts.
