# Distribution Surface Gates

`distribution_surface_gate.v1` is the stable planning contract for deciding
when this repo is allowed to move beyond local Cargo installs, CLI/TUI
workflows, and docs-only future-surface design. It keeps publication,
signed-binary, dependency-attestation, desktop, and local-service decisions
evidence-based instead of turning roadmap pressure into premature
implementation work.

The current release posture remains local-first:

- `cargo install --path` from a trusted checkout is the supported install path.
- Workspace crates keep `publish = false` until a dedicated registry
  publication issue clears this gate.
- The shipped local operator surfaces are `codex-dev`, `codex-research`, and
  `codex-dev-tui`.
- Tauri desktop and Axum local HTTP surfaces stay deferred design candidates.

## Contract

Every future escalation issue must include a `distribution_surface_gate.v1`
block with these fields:

```json
{
  "schema": "distribution_surface_gate.v1",
  "gate_id": "crates_io_publish",
  "surface": "registry",
  "status": "deferred",
  "decision": "do_not_implement_yet",
  "trigger": "One sentence describing the real user or operator need.",
  "owner": "GitHub handle or team that owns the recurring operational burden.",
  "required_evidence": ["Evidence item"],
  "minimum_validation": ["Command or hosted check"],
  "security_review": ["Threat or control to review"],
  "rollback_plan": "How a bad release or surface is contained.",
  "decision_score": {
    "solution_leverage": 0.0,
    "application_value": 0.0,
    "maintenance_load": 0.0,
    "adaptability": 0.0,
    "weighted": 0.0
  },
  "authority": ["Canonical docs or source URLs used for the decision."]
}
```

Use these major-choice weights when scoring a gate:

| Criterion | Weight |
| --- | ---: |
| Solution leverage | 35% |
| Application value | 30% |
| Maintenance and cognitive load | 25% |
| Architectural adaptability | 10% |

Implementation may start only when the weighted score is at least 9.0 and no
`required_evidence` item is `UNVERIFIED`. If a gate cannot reach 9.0, open a
docs/planning issue or keep the existing surface.

## Gate Ledger

| Gate ID | Surface | Current decision | Current score | Implementation trigger |
| --- | --- | --- | ---: | --- |
| `local_cli_tui` | Local Cargo install and terminal UI | Shipped baseline | 9.4 | Keep as the default until another surface proves higher value. |
| `crates_io_publish` | Public registry package publication | Deferred | 6.2 | External users need registry install/update semantics that local path installs and release assets cannot satisfy. |
| `signed_binary_release` | Signed downloadable binary artifacts | Deferred | 6.7 | Non-Rust operators need verifiable binaries and the repo has release owners, signing keys, checksums, and rollback practice. |
| `cargo_vet_attestation` | Dependency audit attestation | Deferred | 7.4 | Distribution becomes public or recurring third-party dependency audit work justifies an owned audit database. |
| `tauri_v2_desktop` | Native desktop workbench | Deferred | 7.7 | A desktop workflow beats the TUI by evidence and can ship with scoped capabilities, signed updates, and a threat model. |
| `axum_local_service` | Local HTTP API over `codex-dev-core` | Deferred | 8.2 | Browser or agent clients need a local API that CLI JSON and TUI panels cannot satisfy safely. |

These scores are intentionally conservative. They reflect the current repo
shape after the local CLI/TUI, PR-agent, bootstrap, scanner, and docs waves. A
future issue must rescore from the then-current code and current official docs.

## `crates_io_publish`

Registry publication is blocked while the workspace is intentionally
`publish = false`. Cargo publication is permanent for each version: a version
cannot be overwritten or deleted after upload. Before this gate can open:

- each crate intended for publication has stable public API docs and a SemVer
  owner;
- `Cargo.toml` metadata is publication-ready, including license, description,
  homepage or repository, and readme metadata;
- package contents are reviewed with `cargo package --list` and archive dry-run
  behavior is recorded with `cargo publish --dry-run`;
- path dependencies, crate names, feature flags, binary names, and workspace
  versioning are intentionally shaped for external users;
- a token-handling plan exists for publishing credentials, including who owns
  crates.io permissions and how tokens are stored outside the repo;
- a yank/rollback policy is documented for broken releases;
- docs describe install, upgrade, compatibility, and support expectations for
  registry users.

Minimum validation:

```bash
cargo fmt --all --check
cargo metadata --locked --no-deps --format-version 1
cargo package --list -p codex-dev-core
cargo package --list -p codex-dev
cargo package --list -p codex-dev-tui
cargo package --list -p codex-research
cargo publish --dry-run -p <crate>
cargo run -q -p codex-dev -- --json policy docs-check
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

References:

- Cargo package command: <https://doc.rust-lang.org/cargo/commands/cargo-package.html>
- Cargo publish command: <https://doc.rust-lang.org/cargo/commands/cargo-publish.html>
- Publishing on crates.io: <https://doc.rust-lang.org/cargo/reference/publishing.html>

## `signed_binary_release`

Signed downloadable binaries are blocked until the repo has a release owner and
artifact trust model. Before this gate can open:

- supported target triples and shell completion/manpage artifacts are listed;
- artifacts have checksums and a documented verification flow;
- signing keys or signing service ownership is documented, including rotation,
  revocation, and recovery;
- release notes, tags, and artifact provenance are tied to a specific commit;
- smoke checks prove binaries execute from outside the repo on every claimed
  platform;
- rollback instructions explain whether to replace, yank, deprecate, or leave
  a flawed artifact in place.

Minimum validation:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo clippy -p codex-dev-tui --all-targets -- -D warnings
cargo clippy -p codex-research --all-targets -- -D warnings
cargo test -p codex-dev
cargo test -p codex-dev-tui
cargo test -p codex-research
cargo run -q -p codex-dev -- completions zsh >/tmp/codex-dev.zsh
cargo run -q -p codex-dev -- manpage >/tmp/codex-dev.1
cargo run -q -p codex-dev-tui -- completions zsh >/tmp/codex-dev-tui.zsh
cargo run -q -p codex-dev-tui -- manpage >/tmp/codex-dev-tui.1
cargo run -q -p codex-research -- completions zsh >/tmp/codex-research.zsh
cargo run -q -p codex-research -- manpage >/tmp/codex-research.1
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

References:

- GitHub release asset integrity:
  <https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/verifying-the-integrity-of-a-release>
- Sigstore/Cosign signing overview:
  <https://docs.sigstore.dev/cosign/signing/overview/>
- SLSA provenance: <https://slsa.dev/spec/v1.2/provenance>

## `cargo_vet_attestation`

`cargo-vet` stays deferred while this repo is local-first. The tool is useful
when a project needs a recorded audit database for third-party code, but that
database is recurring security work and needs an owner. Before this gate can
open:

- one owner is responsible for audit criteria, exemptions, imports, and updates;
- `cargo vet init` output is reviewed and committed intentionally;
- CI behavior is explicit for cached and networked runs;
- the release process says how new dependencies, transitive updates, and
  unpublished path dependencies are reviewed;
- the repo documents which third-party audit sources it trusts.

Minimum validation:

```bash
cargo vet
cargo run -q -p codex-dev -- --json policy docs-check
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

Reference:

- Cargo Vet: <https://mozilla.github.io/cargo-vet/>

## `tauri_v2_desktop`

Tauri remains design-only until there is a desktop job that the terminal TUI
cannot perform well. Before this gate can open:

- the proposed UI consumes existing `codex-dev-core` read models instead of
  inventing a second task capsule model;
- every `#[tauri::command]` has a typed request/response contract, redaction
  rules, and tests or fixture evidence;
- capabilities are deny-by-default, window-scoped, explicitly referenced from
  config, and audited for broad filesystem, shell, process, localhost,
  clipboard, and updater permissions;
- updater support has signed artifacts, protected private keys, HTTPS metadata,
  and operator recovery instructions;
- provider tokens, hosted-write credentials, paths, PR comments, CI logs, and
  command output are treated as sensitive or attacker-controlled by default;
- the branch includes a threat model and a browser/app verification plan.

Minimum validation:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo test -p codex-dev-core
cargo run -q -p codex-dev -- --json policy docs-check
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

References:

- Tauri security: <https://v2.tauri.app/security/>
- Tauri calling Rust: <https://v2.tauri.app/develop/calling-rust/>
- Tauri plugin permissions: <https://v2.tauri.app/learn/security/using-plugin-permissions/>
- Tauri updater: <https://v2.tauri.app/plugin/updater/>

## `axum_local_service`

Axum remains design-only until there is a local HTTP workflow that CLI JSON and
the TUI cannot satisfy. Before this gate can open:

- the service binds to `127.0.0.1` by default and never exposes LAN access
  without a separate security issue;
- every non-health endpoint uses per-session local auth, strict Host, Origin,
  and CORS validation, and redacted request/response logging;
- handlers are thin adapters over `codex-dev-core` read models or existing
  apply-gated `codex-dev` commands;
- shared state uses typed `State` extractors and clone-cheap state;
- middleware is explicit and testable, including request IDs, body limits,
  timeouts, tracing redaction, CORS, panic handling, and graceful shutdown;
- the service has no hidden daemon autostart and no lingering port after the
  parent command exits.

Minimum validation:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo test -p codex-dev-core
cargo run -q -p codex-dev -- --json policy docs-check
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

References:

- Axum: <https://docs.rs/axum/latest/axum/>
- Tower HTTP: <https://docs.rs/tower-http/latest/tower_http/>
