# Container security hardening checklist

## High-risk anti-patterns (fix first)

- Secrets in images or build args (`ARG TOKEN=...`, `ENV API_KEY=...`)
- `privileged: true`, `network_mode: host`, `pid: host`
- Mounting `/var/run/docker.sock`
- Broad host mounts (`/:/host`, `/etc:/etc`, etc.)
- Using `:latest` tags or untagged images

## Build hardening

- Prefer official base images; pin versions; optionally pin digests.
- Avoid `curl | sh` installers; verify checksums/signatures.
- Use multi-stage builds to keep runtime minimal.
- Use BuildKit secrets for private registries and tokens.
- Keep layers small and deterministic (lockfiles, pinned deps).
- Prefer JSON array `CMD`/`ENTRYPOINT`; consider Docker build checks to catch anti-patterns early.

## Runtime hardening (Compose/K8s)

Default to:

- Non-root user (`USER` in final stage; `user:` in Compose if needed)
- Read-only root filesystem (`read_only: true`)
- `tmpfs` for writable paths (`/tmp`, app cache directories)
- Drop Linux capabilities (`cap_drop: ["ALL"]`) and add only what’s required
- `security_opt: ["no-new-privileges:true"]`

If your platform supports it, also consider:
- Seccomp/apparmor profiles
- User namespaces

## Supply chain (optional but recommended)

If available in your environment:
- Vulnerability scanning (e.g., Trivy, Docker Scout)
- SBOM generation (SPDX/CycloneDX)
- Provenance attestations (SLSA-style provenance)
- Signing (cosign) and verification policies

Keep CI permissions minimal and pin Actions (ideally by commit SHA for high assurance).

Note: provenance attestations can include build argument values. Never pass secrets via build args; use secret mounts.
