# Docker audit report template

Use this structure for reviews and refactors (adjust as needed):

## Executive summary

- What’s being containerized / deployed
- Current risk level (high/medium/low) and why
- “Most important fixes” (3–5 bullets)

## Inventory

- Dockerfiles found
- Compose files found
- Build/publish targets (local only vs registry)

## Findings

Group by severity.

- **High**
  - Finding + impact + proposed fix
- **Medium**
- **Low / Info**

## Recommended target state

- Dockerfile strategy (multi-stage, base images, build system)
- Runtime hardening defaults (user, read-only fs, caps)
- Compose split (dev vs prod), profiles, healthchecks
- CI pipeline (build/test/scan/publish)

## Validation plan (local)

- `docker buildx build ...`
- `docker compose config`
- `docker compose up --build`
- App-specific smoke checks (endpoint/CLI)

## Deliverables checklist

- [ ] `.dockerignore`
- [ ] Dockerfile(s)
- [ ] Compose files (+ dev/prod overrides)
- [ ] CI workflow(s)
- [ ] Documentation notes in PR description (run commands, env vars)

