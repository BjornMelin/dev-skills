# Rust Skill Suite

Two layered global skills for Rust development: a core lane and one narrow
specialist for the surface that actually exists in this tree.

## Skill Map

| Skill | Invocation | Primary scope |
| --- | --- | --- |
| `rust-expert` | Implicit | Core Rust engineering: ownership, lifetimes, traits, async, errors, crate choice, testing, performance, security, and toolchain policy. Also owns TUI, desktop and service work now that those specialists are archived. |
| `rust-cli-clap` | Implicit | CLI apps and `clap`: parsers, subcommands, config/env precedence, stdout/stderr/JSON contracts, tests, completions, and packaging. |

## Routing Rules

- CLI or `clap` parser work routes to `rust-cli-clap`.
- Everything else routes to `rust-expert`.

## Archived specialists

`rust-tui-ratatui`, `rust-tauri-apps`, `rust-web-services` and `rust-mega-eng`
were archived to `archive/skills/`. Each described a surface with no
counterpart in this tree: there is no Tauri app, no Axum service, and the one
TUI (`codex-dev-tui`) is maintained directly. Splitting Rust work five ways
also made the skill descriptions collide, since every one of them matched a
generic Rust question.

They are retained rather than deleted. If a matching surface appears, restore
the skill from `archive/skills/<name>/`, add a routing row above, and re-add it
to the validation commands below and to `bootstrap/packs/rust-cli-agent-repo.json`.

## Validation

Each skill should pass the standard skill validator:

```bash
python3 tools/skill/quick_validate.py skills/rust-expert
python3 tools/skill/quick_validate.py skills/rust-cli-clap
```

The suite also includes Rust-specific metadata checks:

```bash
node skills/rust-expert/scripts/check-reference-links.mjs \
  skills/rust-expert \
  skills/rust-cli-clap

node skills/rust-expert/scripts/check-trigger-evals.mjs \
  skills/rust-expert \
  skills/rust-cli-clap
```

The trigger-eval files are intentionally lightweight fixtures. They are not a
model benchmark; they guard against obvious routing drift, such as failing to
pick up a Rust question that no longer has a dedicated specialist.

## Global Install

The tracked source of truth is `skills/<skill-name>/`. Global installs should
copy the same folder into `~/.agents/skills/<skill-name>`. Existing targets may
be replaced during sync; backups are optional.

After syncing, verify parity:

```bash
diff -qr skills/rust-expert ~/.agents/skills/rust-expert
diff -qr skills/rust-cli-clap ~/.agents/skills/rust-cli-clap
```

`claude-config-audit scan --mirror skills` checks the same thing across every
skill at once, and reports drift in references and scripts, not just `SKILL.md`.

## Maintenance Notes

- Keep `SKILL.md` concise and move long guidance into `references/`.
- Update `assets/trigger-evals.json` when routing rules change.
- Re-run reference-link and trigger-eval checks after adding, renaming, or
  deleting reference files.
- Version-sensitive crate recommendations should be refreshed from official docs
  or source before making public API, security, or release decisions.
