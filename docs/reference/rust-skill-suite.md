# Rust Skill Suite

The Rust skill suite provides layered global skills for Rust development. The
suite is intentionally split so ordinary work routes to the narrowest useful
skill, while broad architecture work stays explicit.

## Skill Map

| Skill | Invocation | Primary scope |
| --- | --- | --- |
| `rust-expert` | Implicit | Core Rust engineering: ownership, lifetimes, traits, async, errors, crate choice, testing, performance, security, and toolchain policy. |
| `rust-cli-clap` | Implicit | CLI apps and `clap`: parsers, subcommands, config/env precedence, stdout/stderr/JSON contracts, tests, completions, and packaging. |
| `rust-tui-ratatui` | Implicit | Terminal UI architecture: Ratatui widgets/layouts, crossterm event loops, async input, state machines, snapshots, UX, and performance. |
| `rust-tauri-apps` | Implicit | Tauri v2 Rust backends: command surfaces, typed IPC, app state, plugins, capabilities, permissions, updater, bundling, and distribution. |
| `rust-web-services` | Implicit | Production Rust HTTP services: Axum, Tokio, Tower, Hyper, SQLx, tracing, config, graceful shutdown, middleware, and integration tests. |
| `rust-mega-eng` | Explicit only | Multi-crate architecture, broad Rust product strategy, release engineering, crate portfolios, and cross-domain execution plans. |

## Routing Rules

Use the narrowest skill that owns the surface:

- CLI or `clap` parser work routes to `rust-cli-clap`.
- TUI or Ratatui work routes to `rust-tui-ratatui`.
- Tauri v2 app/backend work routes to `rust-tauri-apps`.
- Axum/Tokio/Tower service work routes to `rust-web-services`.
- General Rust implementation and review routes to `rust-expert`.
- Broad multi-crate architecture routes to `rust-mega-eng` only when the user explicitly asks for `rust-mega-eng` or `$rust-mega-eng`.

When multiple domains are involved but `rust-mega-eng` is not explicitly
requested, use the focused specialist for the current surface and `rust-expert`
for shared core Rust concerns. During implementation after an explicit
`rust-mega-eng` plan, drop back into the specialist skill for each surface.

## Validation

Each skill should pass the standard skill validator:

```bash
python3 tools/skill/quick_validate.py skills/rust-expert
python3 tools/skill/quick_validate.py skills/rust-cli-clap
python3 tools/skill/quick_validate.py skills/rust-tui-ratatui
python3 tools/skill/quick_validate.py skills/rust-tauri-apps
python3 tools/skill/quick_validate.py skills/rust-web-services
python3 tools/skill/quick_validate.py skills/rust-mega-eng
```

The suite also includes Rust-specific metadata checks:

```bash
node skills/rust-expert/scripts/check-reference-links.mjs \
  skills/rust-expert \
  skills/rust-cli-clap \
  skills/rust-tui-ratatui \
  skills/rust-tauri-apps \
  skills/rust-web-services \
  skills/rust-mega-eng

node skills/rust-expert/scripts/check-trigger-evals.mjs \
  skills/rust-expert \
  skills/rust-cli-clap \
  skills/rust-tui-ratatui \
  skills/rust-tauri-apps \
  skills/rust-web-services \
  skills/rust-mega-eng
```

The trigger-eval files are intentionally lightweight fixtures. They are not a
model benchmark; they guard against obvious routing drift, such as implicitly
using `rust-mega-eng` for a local borrow-checker fix.

## Global Install

The tracked source of truth is `skills/<skill-name>/`. Global installs should
copy the same folder into `~/.agents/skills/<skill-name>`. Existing targets may
be replaced during sync; backups are optional.

After syncing, verify parity:

```bash
diff -qr skills/rust-expert ~/.agents/skills/rust-expert
diff -qr skills/rust-cli-clap ~/.agents/skills/rust-cli-clap
diff -qr skills/rust-tui-ratatui ~/.agents/skills/rust-tui-ratatui
diff -qr skills/rust-tauri-apps ~/.agents/skills/rust-tauri-apps
diff -qr skills/rust-web-services ~/.agents/skills/rust-web-services
diff -qr skills/rust-mega-eng ~/.agents/skills/rust-mega-eng
```

## Maintenance Notes

- Keep `SKILL.md` concise and move long guidance into `references/`.
- Keep `rust-mega-eng` explicit-only in `agents/openai.yaml`.
- Update `assets/trigger-evals.json` when routing rules change.
- Re-run reference-link and trigger-eval checks after adding, renaming, or
  deleting reference files.
- Version-sensitive crate recommendations should be refreshed from official docs
  or source before making public API, security, or release decisions.
