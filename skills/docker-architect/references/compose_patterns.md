# docker compose patterns

## Principles

- Make dev fast (bind mounts, hot reload) without compromising prod hardening.
- Prefer explicit networks and named volumes; avoid host networking.
- Use healthchecks and condition-based dependencies where helpful.

## Files strategy

Common split:
- `docker-compose.yml` (base)
- `docker-compose.dev.yml` (dev overrides)
- `docker-compose.prod.yml` (prod overrides)

Use:
- `docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build`

## Profiles

Profiles allow optional services:

- `profiles: ["dev"]` for dev-only dependencies
- `profiles: ["observability"]` for optional tracing/log stacks

Activate profiles with:
- `docker compose --profile dev up`
- or `COMPOSE_PROFILES=dev docker compose up`

## Environment and `.env`

- Prefer `env_file:` for local dev convenience.
- Prefer explicit `environment:` for required values.
- Do not commit secrets; commit `.env.example` instead.

## Healthchecks and dependencies

- Add `healthcheck:` for critical services.
- For Compose v2+, `depends_on` can be combined with service health conditions (support varies by implementation).
  - If uncertain, rely on app-level retries and healthchecks.

## Volumes and permissions

- Named volumes for state (db data)
- Bind mounts for source code in dev
- Keep mounts narrow; avoid mounting host root.

## Security knobs (Compose)

Prefer these for production-like local runs:

- `read_only: true`
- `tmpfs:` for `/tmp` and other writable paths
- `cap_drop: ["ALL"]` + minimal `cap_add`
- `security_opt: ["no-new-privileges:true"]`
- Avoid `privileged: true`, `pid: host`, `network_mode: host`.

## Resource sizing

For memory-bound services (DB, vector stores):
- set `deploy.resources.limits` (Compose uses this mostly in Swarm; still useful as documentation)
- set service-native memory flags (Postgres shared buffers, JVM heap, etc.)

## Compose validation

Always run:
- `docker compose config` (renders and validates)
