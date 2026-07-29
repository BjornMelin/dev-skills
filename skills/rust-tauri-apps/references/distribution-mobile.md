# Distribution and Mobile

## Desktop Packaging

For release work, inspect existing scripts before changing Tauri config. Common checks:

- `cargo test` and `cargo clippy` in `src-tauri`
- frontend build
- `tauri build` for target platforms
- signing/notarization configuration where required
- installer metadata and icons
- update artifacts and signatures

## Sidecars

Sidecars need explicit lifecycle ownership:

- versioned binary source
- checksum verification
- platform-specific paths
- startup/shutdown behavior
- stdout/stderr capture policy
- failure reporting

Do not shell out to arbitrary paths supplied by the frontend.

## Mobile

When mobile is in scope:

- Confirm plugin mobile support.
- Audit permissions separately for iOS and Android.
- Keep filesystem assumptions platform-aware.
- Test deep links, notifications, and secure storage on real/simulator lanes where available.

## Release Automation

Prefer reproducible release scripts over hand-run build sequences. Keep secrets in CI secret stores and avoid local-only signing assumptions unless the project intentionally releases manually.
