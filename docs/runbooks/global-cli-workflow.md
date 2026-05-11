# Global CLI Workflow

Use this runbook when installing or updating the local Rust CLIs so they are
available from any directory:

- `codex-research`
- `codex-dev`
- `codex-dev-tui`

This repo intentionally uses Cargo-native install/update commands. It does not
publish crates, mutate shell startup files, or require secrets for local CLI
handoff.

## Install Or Update

Run from a trusted checkout:

```bash
git checkout main
git pull --ff-only
cargo install --path crates/codex-research --locked --force
cargo install --path crates/codex-dev --locked --force
cargo install --path crates/codex-dev-tui --locked --force
```

`--locked` installs from the committed `Cargo.lock`. `--force` replaces an
older local install for the same binary. Cargo installs to `~/.cargo/bin` by
default; make sure that directory is already on `PATH`.

Smoke the installed binaries from another directory:

```bash
tmp=$(mktemp -d)
cd "$tmp"
codex-research --help >/dev/null
codex-dev --help >/dev/null
codex-dev-tui --help >/dev/null
```

## Completion And Manpage Smokes

The shipped binaries generate local shell completions and roff manpages from
their canonical Clap command definitions. Supported completion shells are the
shells exposed by `clap_complete`; the WSL-first workflow below blesses bash,
zsh, and fish.

Smoke artifact generation from source:

```bash
cargo run -q -p codex-research -- completions zsh >/tmp/codex-research.zsh
cargo run -q -p codex-dev -- completions zsh >/tmp/codex-dev.zsh
cargo run -q -p codex-dev-tui -- completions zsh >/tmp/codex-dev-tui.zsh
cargo run -q -p codex-research -- manpage >/tmp/codex-research.1
cargo run -q -p codex-dev -- manpage >/tmp/codex-dev.1
cargo run -q -p codex-dev-tui -- manpage >/tmp/codex-dev-tui.1
```

Generate completions from installed binaries:

```bash
mkdir -p ~/.local/share/dev-skills/completions/{bash,zsh,fish}
codex-research completions bash > ~/.local/share/dev-skills/completions/bash/codex-research
codex-dev completions bash > ~/.local/share/dev-skills/completions/bash/codex-dev
codex-dev-tui completions bash > ~/.local/share/dev-skills/completions/bash/codex-dev-tui
codex-research completions zsh > ~/.local/share/dev-skills/completions/zsh/_codex-research
codex-dev completions zsh > ~/.local/share/dev-skills/completions/zsh/_codex-dev
codex-dev-tui completions zsh > ~/.local/share/dev-skills/completions/zsh/_codex-dev-tui
codex-research completions fish > ~/.local/share/dev-skills/completions/fish/codex-research.fish
codex-dev completions fish > ~/.local/share/dev-skills/completions/fish/codex-dev.fish
codex-dev-tui completions fish > ~/.local/share/dev-skills/completions/fish/codex-dev-tui.fish
```

Generate local manpages from installed binaries:

```bash
mkdir -p ~/.local/share/man/man1
codex-research manpage > ~/.local/share/man/man1/codex-research.1
codex-dev manpage > ~/.local/share/man/man1/codex-dev.1
codex-dev-tui manpage > ~/.local/share/man/man1/codex-dev-tui.1
```

Shell startup integration is intentionally manual because this repo should not
rewrite user shell configuration. Point your shell completion path at the
generated directory if you want persistent completions.

## Install Smoke Gates

For validation without mutating `~/.cargo/bin`, install into isolated Cargo
roots under `target/` and execute the binaries from `/tmp`. These smokes use
`--offline` because they are part of the local non-network policy profile; run
the normal install/update workflow once first if the Cargo cache is empty.

```bash
repo=$(pwd)
root="$repo/target/codex-dev-install-smoke/codex-research"
rm -rf "$root"
cargo install --path crates/codex-research --locked --offline --force --root "$root"
(cd /tmp && "$root/bin/codex-research" --help >/dev/null && "$root/bin/codex-research" completions zsh >/dev/null && "$root/bin/codex-research" manpage >/dev/null)

root="$repo/target/codex-dev-install-smoke/codex-dev"
rm -rf "$root"
cargo install --path crates/codex-dev --locked --offline --force --root "$root"
(cd /tmp && "$root/bin/codex-dev" --help >/dev/null && "$root/bin/codex-dev" completions zsh >/dev/null && "$root/bin/codex-dev" manpage >/dev/null)

root="$repo/target/codex-dev-install-smoke/codex-dev-tui"
rm -rf "$root"
cargo install --path crates/codex-dev-tui --locked --offline --force --root "$root"
(cd /tmp && "$root/bin/codex-dev-tui" --help >/dev/null && "$root/bin/codex-dev-tui" completions zsh >/dev/null && "$root/bin/codex-dev-tui" manpage >/dev/null)
```

These are included in the `full_local` policy profile because they are heavier
than ordinary source-level smoke checks.
