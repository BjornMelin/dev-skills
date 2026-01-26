# Dockerfile patterns (BuildKit-first)

## Goals

- Reproducible builds (pin base images/tags; optionally digests)
- Small runtime images (multi-stage; minimal runtime deps)
- Secure defaults (non-root runtime; least privilege; no secrets in layers)
- Fast builds (cache-friendly COPY order; BuildKit cache mounts)

## Recommended Dockerfile header

Use the Dockerfile frontend syntax directive recommended by Docker to unlock BuildKit features:

```dockerfile
# syntax=docker/dockerfile:1
```

Optional (advanced): enable BuildKit “build checks” (lint/dry-run) in CI by using a `check=` directive
or `docker build --check` (requires newer Buildx). Prefer documenting this choice, since it can make
builds fail when new checks are introduced.

## Multi-stage baseline

- **builder stage**: compilers, package managers, dependency resolution
- **runtime stage**: only runtime deps + app artifact

Common mistakes:
- Installing build tooling in the runtime stage
- Copying the whole repo before restoring dependencies (kills cache)

## Pinning base images (tags vs digests)

- Prefer explicit version tags (avoid implicit `latest`).
- For highly reproducible production builds, pin a digest:
  - `FROM python:3.12-slim-bookworm@sha256:...`
  - Keep digest bumping as an explicit maintenance chore.

Use Exa (official sources) + `docker buildx imagetools inspect` to confirm current tags and platforms.

## BuildKit cache + secrets mounts

- Cache mounts (speed up dependency installs):

```dockerfile
RUN --mount=type=cache,target=/root/.cache ...
```

- Secret mounts (avoid baking tokens into layers):

```dockerfile
RUN --mount=type=secret,id=npmrc,target=/root/.npmrc ...
```

Never use `ARG` or `ENV` for secrets that must not leak into layers/history.

## Build checks (lint/dry-run)

Docker can run “build checks” to flag common anti-patterns (like non-JSON `CMD`/`ENTRYPOINT`).

- Check without building: `docker build --check .`
- Fail builds on violations: add a `# check=error=true` directive (see Docker docs) or use
  the `BUILDKIT_DOCKERFILE_CHECK` build arg.

## Non-root runtime

Default to a non-root user in the final stage:

- Create user/group with stable UID/GID (helps volume permissions)
- Own only necessary directories

If you must run as root, document the reason and consider dropping privileges after startup.

## Filesystem + process hardening (runtime)

Prefer these at runtime (Compose/K8s), not always in Dockerfile:

- `read_only: true`
- `tmpfs` for writable paths
- `security_opt: ["no-new-privileges:true"]`
- Drop capabilities and add back only what’s required

## Healthchecks

Use one of:
- `HEALTHCHECK` in Dockerfile (portable)
- `healthcheck:` in Compose (environment-specific)

If the app exposes HTTP, healthcheck should hit a lightweight endpoint.

## Multi-arch builds (buildx)

If you publish images:

- Use `docker buildx build --platform linux/amd64,linux/arm64 ...`
- Prefer `buildx` cache (`type=gha` in CI, `type=local` locally)
- Consider SBOM/provenance if supported by your environment.
