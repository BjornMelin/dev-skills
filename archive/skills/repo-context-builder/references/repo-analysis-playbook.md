# Repo analysis playbook

Use this playbook to gather evidence efficiently and avoid generic repo summaries.

## 1. Source priority

Trust signals in this order:

1. Executable configuration and code
   - manifests
   - lockfiles
   - CI workflows
   - deployment config
   - container config
   - infra definitions
   - entrypoint code
2. Tests and test configuration
3. Repository docs such as `README*`, `docs/`, ADRs, specs, runbooks
4. Recent git history and change hotspots
5. Generated artifacts, vendored code, build output, coverage output

If prose and code disagree, prefer the executable source of truth and note the mismatch.

## 2. Fast scan order

### Root-level must-read files

Read what exists:

- `README*`
- `AGENTS.md`
- `.env.example`, `.env.sample`, `sample.env`
- manifest files such as `pyproject.toml`, `package.json`, `Cargo.toml`, `go.mod`, `pom.xml`
- lockfiles such as `uv.lock`, `package-lock.json`, `pnpm-lock.yaml`, `bun.lock`, `poetry.lock`, `Cargo.lock`
- CI and automation such as `.github/workflows/*`, `Makefile`, `justfile`, `Taskfile.yml`
- deployment and infra such as `Dockerfile*`, `compose*.yml`, `vercel.json`, `fly.toml`, `render.yaml`, `serverless.yml`, `*.tf`, `template.yaml`, `cdk.json`

### Runtime and architecture files

Look for:

- API and web entrypoints
- CLI entrypoints
- job / worker / scheduler entrypoints
- config loaders
- dependency injection, app factory, or bootstrap code
- routing layers
- service boundaries
- domain models, schemas, migrations, queues, adapters

### Quality signals

Look for:

- lint config
- typecheck config
- format config
- test config
- representative tests
- pre-commit hooks or similar enforcement

## 3. Repo type heuristics

### Python repo

Prioritize:

- `pyproject.toml`
- `uv.lock`, `poetry.lock`, `requirements*.txt`
- `src/`, package dirs, app entrypoints
- pytest config and tests
- ASGI / WSGI / FastAPI / Django / Flask entrypoints

### JavaScript / TypeScript repo

Prioritize:

- `package.json`
- workspace config such as `pnpm-workspace.yaml`, `turbo.json`, `nx.json`
- build config such as `next.config.*`, `vite.config.*`, `tsconfig*.json`
- `apps/`, `packages/`, `src/`
- scripts in `package.json`

### Monorepo

Prioritize:

- root workspace config
- root CI and deployment config
- each deployable app or service
- shared packages used by more than one app
- package-specific run and test commands

## 4. What to extract for `REPO_CONTEXT.md`

Always extract these when available:

- repository purpose and product / service role
- main deployable surfaces
- important packages / apps / services
- architecture boundaries and data flows
- local setup and execution commands
- lint / typecheck / test / build commands
- deployment targets and release workflow
- environment and secrets conventions
- integrations and external systems
- operational signals such as logging, metrics, alerts
- important files and directories to read first
- current risks, debt, and unknowns

## 5. What to extract for `REVIEW_BRIEF.md`

Always extract these when available:

- the current ask or best inferred next task
- affected code areas and systems
- existing implementation pattern to follow
- current-state findings with evidence
- constraints, assumptions, and non-goals
- concrete plan and file touch list
- verification commands and manual checks
- unresolved questions that block safe implementation

## 6. Evidence anchor style

Use file paths inline. Good examples:

- `pyproject.toml`
- `.github/workflows/ci.yml`
- `apps/web/package.json`
- `src/api/main.py`
- `infra/prod/template.yaml`

Prefer path-based evidence over vague phrases like "the config" or "the backend".

## 7. Anti-patterns to avoid

Do not:

- dump the entire tree
- rewrite the README into a new README
- infer architecture from folder names alone
- claim test coverage or deployment health without evidence
- assume a repo is production-ready because deployment files exist
- silently omit missing sections
- invent environment variables, secrets, commands, or ownership

## 8. Good default when the user did not specify a task

When no specific task is given, choose the most valuable review brief by ranking these:

1. broken or missing verification pipeline
2. unclear or fragile deployment path
3. large architecture risk or cross-cutting debt
4. missing docs that block future work
5. highest-leverage next feature clearly implied by the repo

State that the brief is inferred from repository evidence.
