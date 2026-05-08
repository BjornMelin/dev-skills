# Testing and Packaging

## Test Shape

Use direct library tests for domain behavior and process-spawning tests for CLI contracts.

Useful crates:

- `assert_cmd` for invoking binaries.
- `predicates` for stdout/stderr assertions.
- `trycmd` for command transcript tests.
- `insta` for normalized snapshots.
- `tempfile` for isolated filesystem state.

Test at least:

- Parser success/failure for new flags and subcommands.
- Config/env/CLI precedence.
- stdout/stderr separation.
- JSON schema or shape when machine output exists.
- Exit codes.
- Help examples that users will paste.

## Snapshot Discipline

Normalize:

- absolute paths
- platform path separators when cross-platform
- timestamps
- ordering from maps, filesystems, and concurrent work
- ANSI color unless color itself is under test

Do not snapshot huge help output just because it is easy. Assert stable behavior and snapshot only the text that is intentionally public.

## Distribution

For serious binaries, consider:

- `cargo-dist` for release artifacts and installers.
- `release-plz` for changelog/release automation.
- `cargo-binstall` support where appropriate.
- generated completions/manpages included in packages.

Ship `Cargo.lock` for application binaries. Set `rust-version` deliberately and test the minimum supported Rust version when claiming MSRV support.
