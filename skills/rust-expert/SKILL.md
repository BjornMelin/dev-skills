---
name: rust-expert
description: "Core Rust engineering. Use for implementation, debugging, review, compiler errors, ownership and lifetimes, traits and generics, async and concurrency, typed errors, Cargo and MSRV, crate choice, tests, performance, unsafe and security. Excludes focused product surfaces owned by narrower Rust skills."
---

# Rust Expert

Use this as the default Rust engineering router. Inspect the repo first, load
only the references needed for the task, then use compiler/test feedback as
design evidence.

## Operating Model

1. Read the local guidance and manifests first: `AGENTS.md`, `Cargo.toml`,
   `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, CI, `justfile`,
   `mise.toml`, and nearby tests.
2. Classify the task before editing: compile failure, implementation, review,
   refactor, dependency, public API, release, performance, or security.
3. Fix the root cause. Avoid silencing Rust with reflexive `.clone()`,
   `Arc<Mutex<_>>`, broad trait bounds, `unwrap`, or `unsafe`.
4. Use repo-native gates first. When command surface is unclear, run
   `scripts/discover-rust-gates.mjs <repo-root>`.
5. Research current docs/source when facts can drift: dependency additions or
   upgrades, version-sensitive APIs, Tauri/plugin support, public API/release
   decisions, security, unsafe/FFI, or novel crates.

## Domain Routing

| Signal | Prefer |
| --- | --- |
| Command-line app, subcommands, flags, config precedence, JSON/stdout contracts | `rust-cli-clap` |
| Terminal UI, interactive dashboards, Ratatui widgets/layout/events | `rust-tui-ratatui` |
| Tauri v2, `src-tauri`, commands, capabilities, IPC, updater, mobile layout | `rust-tauri-apps` |
| HTTP APIs/services, Axum, Tower, Tokio server runtime, DB pools | `rust-web-services` |
| Broad architecture review or multi-domain Rust planning explicitly requested | `rust-mega-eng` |

If no specialist owns the task, stay in `rust-expert`. Keep `rust-mega-eng`
explicit-only; broad wording alone does not trigger it.

## Reference Map

Load the smallest needed set:

- `references/toolchain-cargo.md`: editions, MSRV, resolver, lockfiles, features,
  workspace policy, verification commands.
- `references/ownership-async-errors.md`: borrow checker, ownership,
  lifetimes, async `Send`, task ownership, typed errors.
- `references/crate-selection.md`: preferred crates, dependency decision rules,
  feature hygiene, source-refresh triggers.
- `references/testing-quality.md`: unit/integration/doctest/property/snapshot/
  benchmark checks and reviewable test design.
- `references/performance-security.md`: measurement, allocations, unsafe/FFI,
  supply chain, cargo-deny/audit, secrets and subprocesses.

## Toolchain Policy

Existing repos: preserve detected contracts first. Do not change edition,
resolver, `rust-version`, lockfile policy, or exact toolchain pins unless the
task is explicitly a migration or the repo is missing required metadata.

Greenfield defaults:

- `edition = "2024"`.
- Set `[workspace] resolver = "3"` explicitly for virtual workspaces.
- Add an explicit `rust-version` and document the policy.
- Commit `Cargo.lock` by default; for public libraries, also test latest
  dependency resolution because consumers use `Cargo.toml`.
- Avoid exact `rust-toolchain.toml` version pins unless reproducibility,
  nightly, embedded/custom targets, or release policy requires them.

## Verification Ladder

Start narrow, then broaden:

```bash
cargo check -p <crate>
cargo test -p <crate> <test_name>
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test --doc --workspace --locked
```

Use `cargo nextest run` when configured. Add `cargo hack`, `cargo deny`,
`cargo audit`, `cargo semver-checks`, or release tooling only when the repo or
change surface warrants it.

## Helper Scripts

- `scripts/discover-rust-gates.mjs [repo-root]`: print likely Rust gates.
- `scripts/check-reference-links.mjs <skill-dir...>`: validate local skill links.
- `scripts/check-trigger-evals.mjs <skill-dir...>`: validate trigger eval JSON.
