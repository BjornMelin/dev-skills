# GitHub Actions patterns for Docker (build/test/scan/publish)

## Baseline CI goals

- Build the image deterministically (BuildKit/buildx).
- Run unit/integration tests (either inside image or via compose).
- Validate compose configuration (`docker compose config`).
- Optionally run build checks, scan, generate SBOM/provenance, and push to registry.

## Action pinning guidance

- Security-hardening option: pin actions by commit SHA.
- Maintainability option: use major version tags (e.g., `@v3`) and keep them updated.

If unsure, use major versions and add a follow-up task to pin by SHA for production repos.

## Caching

- Prefer `cache-from`/`cache-to` with `type=gha` in CI.
- Use `--mount=type=cache` in Dockerfile for package caches.

## Registry publishing

Use `docker/login-action` with:
- GitHub Container Registry (ghcr.io) via `GITHUB_TOKEN` with scoped permissions, or
- a dedicated registry token stored as a secret.

## Scanning

Scanning tools vary by org; treat as optional:
- Trivy action
- Docker Scout
- Grype/Syft

Always verify the current recommended setup via Exa (official docs), since scanning actions evolve quickly.

## Attestations (SBOM + provenance)

Docker’s official guidance for GitHub Actions is to use `docker/metadata-action` to generate tags
and `docker/build-push-action` with:

- `provenance: mode=max` (recommended for stronger provenance)
- `sbom: true` (SBOM isn’t automatic)

Important constraints (per Docker docs):
- Attestations require pushing to a registry (`push: true`). Images loaded to the runner’s local store
  don’t support attestations.
- Do not pass secrets via build args: build args can be included in provenance; use BuildKit secret mounts.
