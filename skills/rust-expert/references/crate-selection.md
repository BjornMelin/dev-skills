# Crate Selection

Use existing repo choices first. Add or replace dependencies only when the
benefit exceeds API, maintenance, compile-time, feature, license, and supply
chain cost.

## Research Order

1. Inspect manifests, lockfile, features, MSRV, and current dependency graph.
2. Check official docs and examples for current API shape.
3. Check source/changelog/release notes when behavior or version risk matters.
4. Inspect transitive features with `cargo tree -e features`.
5. Prefer one canonical crate per capability; remove parallel stacks when doing
   a deliberate hard cut.

## Opinionated Defaults

| Need | Prefer |
| --- | --- |
| Serialization | `serde`, `serde_json`, format-specific serde crates |
| CLI | `clap` derive; builder for dynamic command construction |
| TUI | `ratatui` plus `crossterm`; `color-eyre` for app reports |
| Desktop/mobile shell | Tauri v2 when webview UI plus Rust backend fits |
| Async runtime | `tokio` with minimal features; `full` only when justified |
| HTTP service | `axum` on Tokio with `tower` middleware |
| HTTP client | `reqwest`, prefer rustls when OpenSSL friction matters |
| SQL | `sqlx` for async SQL and compile-time query checking when feasible |
| Errors | `thiserror` for library boundaries; `anyhow`/`miette` for apps |
| Observability | `tracing`; binaries install subscribers, libraries emit spans/events |
| Benchmarks | `criterion` |
| Property tests | `proptest` |
| Snapshots | `insta` with reviewable redactions |
| Supply chain | `cargo-deny`, `cargo-audit` |
| Public API release | `cargo-semver-checks`, `release-plz` |
| Binary distribution | `cargo-dist` when shipping downloadable artifacts |

## Anti-Patterns

| Smell | Better |
| --- | --- |
| Hand-parsed CLI args | `clap` typed parser |
| `anyhow::Error` in public library API | concrete `thiserror` type |
| Library installs global tracing subscriber | library emits `tracing`; binary installs subscriber |
| SQL built by string concatenation | bind parameters or checked `sqlx::query!` |
| Broad default features | minimal required feature set |
| Git dependency with no policy | crates.io release or explicit approval |
| Duplicate HTTP/JSON/error stacks | choose one canonical stack |
| Tiny utility crate for stdlib behavior | standard library |

## Freshness Triggers

Research live docs/source before relying on:

- new dependencies or major upgrades;
- Tauri plugin/platform support;
- Clap/Ratatui/Tauri current APIs;
- security-sensitive crates;
- MSRV-sensitive dependencies;
- release automation or public API compatibility;
- crates with low maintenance signals or unclear licenses.
