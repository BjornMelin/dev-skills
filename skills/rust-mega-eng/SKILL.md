---
name: rust-mega-eng
description: Explicit-only Rust architecture orchestrator. Use only when the user explicitly invokes rust-mega-eng for broad Rust ecosystem architecture, multi-crate workspace strategy, large refactors, release engineering, crate selection portfolios, or end-to-end Rust product planning across CLI, TUI, Tauri, services, libraries, CI, security, and distribution.
license: MIT
metadata:
  category: rust
  invocation: explicit-only
  domains:
    - architecture
    - multi-crate-workspaces
    - rust-strategy
    - release-engineering
---

# Rust Mega Eng

Use this skill as the explicit orchestration layer for broad Rust architecture and product engineering work. For ordinary local Rust edits, use `rust-expert` or a specialist skill instead.

## Invocation Rule

Do not invoke this skill implicitly for normal Rust questions, compiler errors, single-crate fixes, broad architecture wording, or isolated CLI/TUI/Tauri/service work. Invoke only when the user explicitly names `rust-mega-eng` for broad, cross-cutting Rust architecture and execution planning.

## Operating Model

1. Map the product surface: libraries, binaries, services, desktop apps, terminal UI, deployment targets, public APIs, and release obligations.
2. Identify canonical ownership: crate boundaries, app/service boundaries, shared domain types, generated code, test fixtures, config, docs, and CI gates.
3. Choose leverage first: mature crates, official frameworks, platform conventions, and repo-native workflows before custom infrastructure.
4. Split delivery into reviewable lanes: architecture/design, code movement, behavior changes, test hardening, release/distribution, and docs.
5. Keep a decision ledger for major choices, with explicit tradeoffs and verification evidence.

## Reference Map

- `references/orchestration.md` for workspace strategy, crate portfolio defaults, decision scoring, fanout planning, release engineering, and anti-patterns.

Use the specialist skills for detailed implementation:

- `rust-cli-clap` for CLI and `clap` command surfaces.
- `rust-tui-ratatui` for terminal UI architecture.
- `rust-tauri-apps` for Tauri v2 Rust app backends and IPC.
- `rust-web-services` for Axum/Tokio/Tower services.
- `rust-expert` for core Rust ownership, async, errors, crates, testing, performance, and security.

## Verification

For broad Rust work, define a validation ladder early:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo nextest run --all-targets --all-features
cargo deny check
cargo audit
cargo semver-checks check-release
```

Run only gates that are relevant and available, but report any skipped gate with the reason.
