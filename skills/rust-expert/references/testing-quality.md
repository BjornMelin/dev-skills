# Testing And Quality

Use this reference when adding tests, changing contracts, reviewing quality, or
closing out Rust work.

## Test Choice

| Risk | Test |
| --- | --- |
| Pure behavior | unit test near the module |
| Public crate behavior | integration test under `tests/` |
| Public examples | doctest |
| CLI contract | stdout, stderr, exit status, and JSON schema assertions |
| Error contract | stable variant/category plus user-facing message boundary |
| Serialization | round-trip, golden, or small snapshot |
| Async/concurrency | deterministic synchronization; avoid sleeps |
| Input space invariant | property test |
| UI/TUI rendering | buffer or backend snapshot |
| Performance | benchmark or profiler evidence |
| Public API compatibility | `cargo-semver-checks` |

## Test Quality Rules

- Name tests by behavior.
- Keep fixtures small and deterministic.
- Test failure paths when introducing a new failure class.
- Do not weaken tests to fit the implementation.
- Assert stable contracts, not full debug strings.
- Avoid sleeping in async tests; use barriers, fake clocks, or explicit
  notifications.
- For snapshots, review and redact unstable fields.
- For mocks, test the boundary contract instead of duplicating internals.

## Review Checklist

For Rust reviews, findings lead:

- correctness and data loss;
- public API and serialization contracts;
- async lifecycle, cancellation, and backpressure;
- error classification and user/operator messages;
- dependency and feature changes;
- unsafe/FFI invariants;
- test coverage for changed risk.

Report file/line evidence, impact, and a concrete fix direction.

## Closeout Commands

Prefer repo-native commands. Common Rust closeout:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test --doc --workspace --locked
git diff --check
```

Use narrower commands while iterating; broaden before claiming completion.
