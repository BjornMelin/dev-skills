# Rust Architecture Orchestration

## Workspace Strategy

Use a workspace when multiple crates share versioning, CI, domain types, or release ownership. Keep boundaries concrete:

- `core` or domain crate for pure logic and shared types.
- CLI binary crate for command parsing and terminal contracts.
- TUI crate or binary for persistent terminal interaction.
- Tauri `src-tauri` crate for desktop/mobile backend and IPC.
- Service crate for HTTP/runtime integration.
- Test/support crates only when shared fixtures or harnesses justify them.

Avoid creating crates just to mimic directory structure. A crate boundary should buy independent compilation, feature isolation, API clarity, reuse, or release separation.

## Default Crate Portfolio

Strong defaults by domain:

- CLI: `clap`, `anstream`, `assert_cmd`, `trycmd`, `insta`
- TUI: `ratatui`, `crossterm`, `unicode-width`
- Tauri: `tauri`, official plugins, `serde`, typed DTOs
- Services: `axum`, `tokio`, `tower`, `tracing`, `sqlx`
- Errors: `thiserror` for libraries/domains; `miette`, `color-eyre`, or `anyhow` at app boundaries
- Quality: `cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-semver-checks`, `criterion`, `proptest`, `insta`
- Release: `release-plz`, `cargo-dist`

Verify current versions and framework changes through official docs/source before committing to public APIs or security-sensitive integrations.

## Decision Scoring

For major choices, score 0.0 to 10.0:

- Solution leverage, 35 percent: mature crates, platform fit, less custom code.
- Application value, 30 percent: user impact, reliability, completeness.
- Maintenance load, 25 percent: readable, testable, low upkeep.
- Adaptability, 10 percent: evolves cleanly without over-engineering.

Target 9.0 or better when realistic. If no option reaches 9.0, record the constraint and choose the best reversible option.

## Delivery Lanes

Split broad work into reviewable lanes:

- Workspace/crate boundary changes.
- Public API or behavior changes.
- CLI/TUI/Tauri/service implementation.
- Test and fixture hardening.
- CI/security/release automation.
- Docs and migration notes.

Avoid mixing pure moves with behavioral edits unless the repo is too small for the distinction to matter.

## Anti-Patterns

- Async everywhere without I/O or cancellation requirements.
- `Arc<Mutex<_>>` as the first answer for ownership.
- Stringly-typed command modes, error codes, or IPC contracts.
- Duplicated config precedence across binaries.
- Background tasks without shutdown.
- Broad Tauri capabilities because command validation is weak.
- Public crate changes without semver checks.
- CI that runs only happy-path unit tests for a product with CLI/service/distribution contracts.
