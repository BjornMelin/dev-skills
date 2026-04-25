# Repository Guidelines

This repository is a catalog of **Agent Skills** (per the AgentSkills specification) intended to be reusable and publishable (e.g. via `skills.sh`). Contributions should keep skills spec-compliant, self-contained, and easy to install.

## Project Structure & Module Organization

- `skills/<skill-name>/SKILL.md`: canonical entrypoint (required) with YAML frontmatter + instructions.
- `skills/<skill-name>/references/`: optional long-form docs to load on demand.
- `skills/<skill-name>/scripts/`: optional helper scripts (prefer deterministic tooling here).
- `skills/<skill-name>/assets/` / `templates/`: optional reusable artifacts.
- `skills/<skill-name>/agents/`: optional agent-runtime metadata (for example OpenAI YAML).
- `skills/dist/`: prebuilt `.skill` bundles (ZIP archives) for selected skills.

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

## Coding Style & Naming Conventions

- Skill names must be **hyphen-case** and match the folder name (e.g. `langgraph-multiagent`).
- `SKILL.md` frontmatter should only use allowed keys: `name`, `description`, `license`, `allowed-tools`, `metadata`.
- Keep `SKILL.md` concise; put large content in `references/`. Prefer scripts over massive inline code blocks.

## Testing Guidelines

There is no repo-wide test harness. Treat `tools/skill/quick_validate.py` as the required “gate” for changes to any skill. If you add scripts, keep them runnable without external secrets and avoid network calls unless the skill explicitly requires it.

## Commit & Pull Request Guidelines

This repo may not have established git history conventions yet. Use clear, scoped commits (recommended: Conventional Commits), e.g. `feat(dmc-py): add callbacks scaffold` or `docs: expand README catalog`. PRs should include:

- What changed + why
- Validation commands run (at minimum `python3 tools/skill/quick_validate.py skills/<skill-name>`)
- If you add or rename a skill, update the catalog table in `README.md` (keep rows sorted by skill name)
- If you built/published bundles, say where (release assets/registry)

## Security & Configuration Tips

Do not commit credentials or private tokens (use `.env.example` patterns). Assume any `.skill` bundle is redistributable; keep sensitive material out of skill folders and packaged artifacts.
