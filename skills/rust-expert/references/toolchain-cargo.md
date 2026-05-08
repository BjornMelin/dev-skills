# Toolchain And Cargo

Use this reference for toolchain policy, workspace structure, feature hygiene,
dependency resolution, and verification commands.

## Existing Repo Policy

Preserve these contracts unless the user explicitly asks for a migration:

- `edition`
- `rust-version`
- workspace `resolver`
- `rust-toolchain.toml`
- `.cargo/config.toml`
- lockfile policy
- CI matrix and README MSRV policy

Changing any of these can affect dependency resolution and user compatibility.
Treat edition/resolver/MSRV changes as migrations with lockfile review and full
repo checks.

## Greenfield Defaults

Use these defaults for new Rust projects unless the target domain says
otherwise:

```toml
[package]
edition = "2024"
rust-version = "1.85"
```

For virtual workspaces:

```toml
[workspace]
resolver = "3"
members = ["crates/*"]
```

Use `rust-version = "1.85"` as the Rust 2024 compatibility floor for reusable
libraries/tools, then raise it only for chosen APIs/dependencies. For internal
apps, current stable is acceptable if documented and CI tracks it.

## Lockfiles

Commit `Cargo.lock` by default for apps, CLIs, services, workspaces, and agent
tooling. For published libraries, committing the lockfile is acceptable, but CI
should also test latest dependency resolution because downstream users are not
bound by the library lockfile.

Avoid upper-bounding dependencies to preserve old Rust compatibility unless a
real incompatibility requires it. Prefer `rust-version` plus resolver 3 and
latest-dependency CI.

## Features

- Use workspace dependency ownership when possible.
- Keep features additive.
- Disable default features only when you know what stack you are removing.
- Avoid `full` feature sets unless the task genuinely needs the entire surface.
- Test important feature combinations with `cargo hack` when public crates or
  optional stacks are involved.

Useful commands:

```bash
cargo metadata --format-version=1
cargo tree -e features
cargo tree -d
cargo hack check --feature-powerset --no-dev-deps
```

## Verification Tiers

Focused iteration:

```bash
cargo check -p <crate>
cargo test -p <crate> <test_name>
```

Normal closeout:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Public API/release:

```bash
cargo test --doc --workspace --locked
cargo semver-checks
release-plz update --dry-run
```

Supply-chain/security:

```bash
cargo deny check
cargo audit
```

Use repo-native wrappers (`just`, `mise`, `make`, `xtask`, RTK) when they exist.
