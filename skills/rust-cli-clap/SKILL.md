---
name: rust-cli-clap
description: Implicit Rust CLI and Clap skill. Use for command-line apps and tools, clap derive or builder APIs, subcommands, flags, config and env precedence, stdin/stdout/stderr contracts, JSON output, exit codes, completions, manpages, shell UX, CLI tests, and binary distribution.
license: MIT
metadata:
  category: rust
  domains:
    - cli
    - clap
    - command-line-tools
    - packaging
---

# Rust CLI Clap

Build Rust command-line tools that are predictable, scriptable, pleasant in terminals, and easy to ship.

## Operating Model

1. Discover the existing CLI contract first: `Cargo.toml`, `src/bin`, `clap` definitions, integration tests, snapshots, release scripts, and README examples.
2. Preserve user-facing command behavior unless the task explicitly asks for a breaking change. If behavior changes, update tests and docs in the same patch.
3. Prefer `clap` derive for ordinary CLIs. Use the builder API when commands are generated dynamically, hidden/unstable command surfaces need custom construction, or macros make the shape harder to read.
4. Design command output as an API: human output on stdout, diagnostics on stderr, deterministic `--json`/machine output where automation is expected, and stable exit codes.
5. Keep command handlers thin. Parse into typed arguments, normalize config once, then call library code that tests can exercise without spawning a process.

## Reference Map

Open only the section needed for the task:

- `references/clap-parser-playbook.md` for `clap` derive/builder design, argument groups, validators, completions, and migration notes.
- `references/terminal-contracts.md` for stdout/stderr, color, progress, config precedence, error messages, and shell automation rules.
- `references/testing-packaging.md` for `assert_cmd`, `trycmd`, snapshots, binaries, completions, manpages, `cargo-dist`, and release checks.

## Defaults

- Use `clap = { features = ["derive", "env"] }` when environment variables are part of the contract; otherwise avoid unused features.
- Use `camino` or `Utf8PathBuf` only when UTF-8 paths are an explicit invariant. Otherwise keep `PathBuf`.
- Use `anstream`/`anstyle` or `clap` styling for color-aware output; respect `NO_COLOR`, `CLICOLOR`, and non-TTY behavior.
- Use `tracing` for diagnostics when the CLI has subcommands, network calls, daemon/client modes, or hidden debugging flags.
- Use `thiserror` for domain errors and `miette` or `color-eyre` at the presentation boundary only when rich reports add value.

## Verification

For CLI changes, prefer the smallest ladder that proves the contract:

```bash
cargo fmt --all --check
cargo test --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Add focused command tests for new flags, output modes, exit codes, config precedence, and examples shown in docs. Snapshot CLI output only after normalizing volatile paths, timestamps, colors, and ordering.
