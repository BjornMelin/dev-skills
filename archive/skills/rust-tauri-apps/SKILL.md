---
name: rust-tauri-apps
description: Implicit Rust Tauri v2 skill. Use for Tauri apps, Rust commands, invoke contracts, app state, plugins, capabilities and permissions, secure IPC, filesystem and shell access, sidecars, updater, bundling, desktop and mobile distribution, and Rust frontend bridge design.
license: MIT
metadata:
  category: rust
  domains:
    - tauri
    - desktop-apps
    - rust-backend
    - secure-ipc
---

# Rust Tauri Apps

Build Tauri v2 applications with small Rust command surfaces, secure capabilities, explicit IPC contracts, and maintainable distribution workflows.

## Operating Model

1. Inspect the Tauri version and generated structure first: `src-tauri`, `tauri.conf.json`, capabilities, permissions, plugins, frontend invocation code, and CI/release scripts.
2. Treat every command as a public bridge contract. Validate input, return typed serializable results, and keep filesystem/shell/network capabilities narrow.
3. Prefer app-owned Rust services behind commands. Do not put large domain logic directly in `#[tauri::command]` functions.
4. Keep the capability model least-privilege by default. Avoid broad filesystem, shell, opener, and dialog permissions unless the user workflow truly needs them.
5. Coordinate with the general `tauri-v2` skill when available for framework-specific current docs; use this skill for Rust architecture, security, testing, and command design.

## Reference Map

- `references/tauri-v2-rust-backend.md` for command/state/plugin architecture and typed IPC.
- `references/capabilities-security.md` for permissions, filesystem/shell risk, secrets, updater, and secure defaults.
- `references/distribution-mobile.md` for bundling, updater, sidecars, signing, desktop/mobile packaging, and release checks.

## Defaults

- Keep commands async only when they await I/O. Do CPU-heavy work off the main runtime.
- Use `serde` DTOs at the IPC boundary and convert to domain types inside services.
- Use `tauri::State` for shared app services with clear ownership; avoid global mutable statics.
- Use plugin APIs instead of hand-rolled platform code when mature official plugins cover the use case.
- Keep frontend-originated paths, URLs, and shell arguments untrusted until validated.

## Verification

Use repo-native checks first, then add Tauri-specific validation:

```bash
cargo fmt --all --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

For command changes, add Rust tests around service behavior and frontend tests or contract fixtures for IPC shapes when the app already has a frontend test lane.
