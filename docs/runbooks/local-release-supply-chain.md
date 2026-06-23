# Local Release and Supply-Chain Runbook

Use this runbook before handing off local installs, release assets, or any PR
that changes workspace Cargo metadata. Pair it with the
[Global CLI Workflow](global-cli-workflow.md) when installing, updating, or
generating shell artifacts for the local binaries. Pair any request for public
registry publication, signed binary artifacts, `cargo-vet`, Tauri desktop, or
Axum local-service work with the
[`distribution_surface_gate.v1`](../reference/distribution-surface-gates.md)
contract before implementation starts. This runbook is intentionally
Cargo-native: the repo records policy in `Cargo.toml`, `Cargo.lock`, `deny.toml`,
and `codex-dev` policy profiles instead of adding a separate release framework.

References:

- Cargo manifest reference: <https://doc.rust-lang.org/cargo/reference/manifest.html>
- Cargo install command: <https://doc.rust-lang.org/cargo/commands/cargo-install.html>
- Cargo package command: <https://doc.rust-lang.org/cargo/commands/cargo-package.html>
- Cargo publish command: <https://doc.rust-lang.org/cargo/commands/cargo-publish.html>
- Rust 2024 Cargo resolver: <https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html>
- cargo-deny: <https://embarkstudios.github.io/cargo-deny/>
- RustSec and cargo-audit: <https://rustsec.org/>
- cargo-vet: <https://mozilla.github.io/cargo-vet/>

## Workspace Policy

- The workspace uses Rust 2024 and resolver `3`. Resolver `3` makes dependency
  resolution aware of package `rust-version` values for Rust 2024 workspaces.
- All Rust crates inherit `rust-version = "1.88"` from `[workspace.package]`.
  Rust 1.85 is the first stable toolchain for Rust 2024, but the current locked
  dependency graph contains transitive crates that require Rust 1.88, so 1.88 is
  the repo MSRV until those dependencies are replaced or downgraded.
- `codex-dev-core` is a library crate. Installable local binaries are
  `codex-research`, `codex-dev`, and `codex-dev-tui`. `gsap-audit-core` is a
  library crate; `gsap-audit` is an optional companion CLI for the standalone
  `gsap` skill.
- The `gsap-audit-core` oxc dependency tree (`oxc_allocator`, `oxc_ast`,
  `oxc_ast_visit`, `oxc_parser`, `oxc_semantic`, and `oxc_span` at `0.137.0`,
  plus `walkdir`) was reviewed under the `cargo deny check licenses bans sources`
  gates. Treat new oxc or related transitive additions as supply-chain review
  items before merge.
- All workspace crates set `publish = false`; this repo supports local install
  handoff, not crates.io publication.
- Path dependencies between workspace crates include package versions so
  `cargo package --list` and cargo-deny wildcard checks exercise publish-like
  metadata without actually publishing anything.
- Distribution escalation is gate-driven. A future issue must use
  [`distribution_surface_gate.v1`](../reference/distribution-surface-gates.md)
  before changing `publish`, adding release signing, adopting `cargo-vet`, or
  building new local app surfaces.

## Release Baseline Gates

Run these from the repository root:

```bash
cargo fmt --all --check
cargo metadata --locked --no-deps --format-version 1
cargo tree -d --target all
cargo deny check bans licenses sources
cargo deny check advisories
cargo audit
cargo package --list -p codex-dev-core
cargo package --list -p codex-dev
cargo package --list -p bun-platform-core
cargo package --list -p bun-platform
cargo package --list -p gsap-audit-core
cargo package --list -p gsap-audit
cargo package --list -p codex-dev-tui
cargo package --list -p codex-research
cargo run -q -p codex-dev -- --json policy manifest --profile release
cargo run -q -p codex-dev -- --json policy manifest --profile full_local
cargo run -q -p codex-dev -- --json policy docs-check
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

`cargo deny check advisories` and `cargo audit` fetch the RustSec advisory
database unless their local databases are already fresh. Treat those as explicit
networked release evidence. The built-in `codex-dev` policy profiles keep their
required gates local and non-secret by default, so automated local runs do not
silently depend on external network state.

When cargo-deny is available but network access is not, run:

```bash
cargo deny check bans licenses sources
cargo deny check advisories --disable-fetch
```

The second command requires a previously cached advisory database.

## Package Dry Runs

The package gates use `cargo package --list` instead of `cargo publish` or
`cargo package` archive creation. This catches missing package metadata,
invalid path dependency metadata, and unexpected file inclusion without creating
archives or pushing to a registry.

On a dirty working tree, add `--allow-dirty` for a pre-commit package preview.
Final release evidence should run without `--allow-dirty` after the release
branch is committed so Cargo proves the packaged contents match versioned files.

Before any real registry publication, run a dedicated release PR that clears the
[`crates_io_publish`](../reference/distribution-surface-gates.md#crates_io_publish)
gate, adds registry-specific package metadata, confirms archive contents, and
documents the publishing account, token handling, SemVer owner, and rollback
plan. This repo currently supports local installation handoff, not crates.io
publication.

## Signed Binary Status

Signed downloadable binaries are deferred. The repo does not yet have a release
owner, artifact signing key policy, checksum/provenance workflow, target matrix,
or rollback process that justifies binary distribution. Adopt signed artifacts
only in a dedicated issue that clears the
[`signed_binary_release`](../reference/distribution-surface-gates.md#signed_binary_release)
gate.

## Duplicate Dependency Baseline

`cargo tree -d --target all` is the cross-target duplicate report. cargo-deny
sets `multiple-versions = "deny"` and skips only the exact accepted older
versions in `deny.toml`, so new duplicate families or changed duplicate
versions should fail `cargo deny check bans licenses sources`.

As of the release baseline, accepted duplicate families are:

| Crate family | Why accepted for now | Revisit when |
| --- | --- | --- |
| `hashbrown` 0.16 and 0.17 | Pulled by current transitive versions under `rusqlite`/`hashlink`, `ratatui-core`/`lru`/`kasuari`, and `indexmap` users. | `ratatui`, `rusqlite`, `reqwest`, or `toml_edit` update their transitive graph enough to converge. |
| `core-foundation` 0.9 and 0.10 | Target-specific transitive drift in the current dependency graph. | macOS-target dependencies converge or a direct dependency update removes one side. |
| `getrandom` 0.2 and 0.3 | Common transition-period duplicate across async/networking and crypto-adjacent crates. | upstream crates complete the `getrandom` 0.3 migration. |
| `windows-sys` 0.61 repeated entries | `cargo tree -d --target all` can show same-version target-specific paths; cargo-deny does not treat this as a multiple-version duplicate. | a future Cargo tree report no longer repeats same-version target paths. |

Do not add direct dependencies solely to deduplicate these families. Prefer
upstream package updates or replacement only when the direct application value
outweighs churn.

## cargo-vet Status

`cargo-vet` is deferred. The repo does not yet have an owned audit database, a
publisher rotation, or a public registry release process that justifies local
audit attestation maintenance. Adopt it in a dedicated issue that clears the
[`cargo_vet_attestation`](../reference/distribution-surface-gates.md#cargo_vet_attestation)
gate when distribution moves beyond local installs or release assets, and
include:

- owner for audit decisions;
- `cargo vet init` output and policy config;
- review process for new transitive imports;
- CI strategy for cached or networked vet checks.

## Global Local Installs

Install or update the local CLIs from a trusted checkout:

```bash
git checkout main
git pull --ff-only
cargo install --path crates/codex-research --locked --force
cargo install --path crates/codex-dev --locked --force
cargo install --path crates/codex-dev-tui --locked --force
```

Use [Global CLI Workflow](global-cli-workflow.md) for completion generation,
manpage generation, and isolated `cargo install --root` smoke checks that prove
the binaries execute from another directory without mutating `~/.cargo/bin`.

Smoke the installed binaries from any directory:

```bash
codex-research --json doctor
codex-research --json eval
codex-dev --help
codex-dev-tui --help
```

If you need to verify behavior before installing, use
`cargo run -q -p <package> -- ...` from this repo. Keep `cargo install --path`
pointed at the trusted local checkout; do not install from copied build
artifacts.
