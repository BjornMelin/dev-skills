# Global CLI Workflow

Use this runbook when installing or updating the local Rust CLIs so they are
available from any directory:

- `codex-research`
- `codex-dev`
- `codex-dev-tui`
- `expo-motion-audit` (optional companion CLI for Expo and React Native motion)
- `gsap-audit` (optional companion CLI for the standalone `gsap` skill)
- `motion-token-audit` (optional companion CLI for cross-stack motion tokens)

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
cargo install --path crates/expo-motion-audit --locked --force
cargo install --path crates/gsap-audit --locked --force
cargo install --path crates/motion-token-audit --locked --force
```

`--locked` installs from the committed `Cargo.lock`. `--force` replaces an
older local install for the same binary. Cargo installs to `~/.cargo/bin` by
default; make sure that directory is already on `PATH`.

## Retired Bun compatibility binary

This repository no longer builds or installs the retired `bun-platform`
compatibility binary. Existing workstation copies are unsupported. After the
source migration is merged and local-deletion approval is explicit, retire the
old executable and generated completions:

```bash
cargo uninstall bun-platform
rm -f \
  ~/.local/share/dev-skills/completions/bash/bun-platform \
  ~/.local/share/dev-skills/completions/zsh/_bun-platform \
  ~/.local/share/dev-skills/completions/fish/bun-platform.fish
! command -v bun-platform
```

Issue [#105](https://github.com/BjornMelin/dev-skills/issues/105) owns the
approval-gated workstation and predecessor-repository cleanup evidence.

Smoke the installed binaries from another directory:

```bash
tmp=$(mktemp -d)
cd "$tmp"
codex-research --help >/dev/null
codex-dev --help >/dev/null
codex-dev-tui --help >/dev/null
expo-motion-audit --help >/dev/null
gsap-audit --help >/dev/null
motion-token-audit --help >/dev/null
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
cargo run -q -p expo-motion-audit -- completions zsh >/tmp/expo-motion-audit.zsh
cargo run -q -p gsap-audit -- completions zsh >/tmp/gsap-audit.zsh
cargo run -q -p motion-token-audit -- completions zsh >/tmp/motion-token-audit.zsh
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
expo-motion-audit completions bash > ~/.local/share/dev-skills/completions/bash/expo-motion-audit
gsap-audit completions bash > ~/.local/share/dev-skills/completions/bash/gsap-audit
motion-token-audit completions bash > ~/.local/share/dev-skills/completions/bash/motion-token-audit
codex-research completions zsh > ~/.local/share/dev-skills/completions/zsh/_codex-research
codex-dev completions zsh > ~/.local/share/dev-skills/completions/zsh/_codex-dev
codex-dev-tui completions zsh > ~/.local/share/dev-skills/completions/zsh/_codex-dev-tui
expo-motion-audit completions zsh > ~/.local/share/dev-skills/completions/zsh/_expo-motion-audit
gsap-audit completions zsh > ~/.local/share/dev-skills/completions/zsh/_gsap-audit
motion-token-audit completions zsh > ~/.local/share/dev-skills/completions/zsh/_motion-token-audit
codex-research completions fish > ~/.local/share/dev-skills/completions/fish/codex-research.fish
codex-dev completions fish > ~/.local/share/dev-skills/completions/fish/codex-dev.fish
codex-dev-tui completions fish > ~/.local/share/dev-skills/completions/fish/codex-dev-tui.fish
expo-motion-audit completions fish > ~/.local/share/dev-skills/completions/fish/expo-motion-audit.fish
gsap-audit completions fish > ~/.local/share/dev-skills/completions/fish/gsap-audit.fish
motion-token-audit completions fish > ~/.local/share/dev-skills/completions/fish/motion-token-audit.fish
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

root="$repo/target/codex-dev-install-smoke/expo-motion-audit"
rm -rf "$root"
cargo install --path crates/expo-motion-audit --locked --offline --force --root "$root"
(cd /tmp && "$root/bin/expo-motion-audit" --help >/dev/null && "$root/bin/expo-motion-audit" doctor >/dev/null && "$root/bin/expo-motion-audit" completions zsh >/dev/null)

root="$repo/target/codex-dev-install-smoke/gsap-audit"
rm -rf "$root"
cargo install --path crates/gsap-audit --locked --offline --force --root "$root"
(cd /tmp && "$root/bin/gsap-audit" --help >/dev/null && "$root/bin/gsap-audit" doctor >/dev/null && "$root/bin/gsap-audit" completions zsh >/dev/null)

root="$repo/target/codex-dev-install-smoke/motion-token-audit"
rm -rf "$root"
cargo install --path crates/motion-token-audit --locked --offline --force --root "$root"
(cd /tmp && "$root/bin/motion-token-audit" --help >/dev/null && "$root/bin/motion-token-audit" doctor >/dev/null && "$root/bin/motion-token-audit" completions zsh >/dev/null)
```

These are included in the `full_local` policy profile because they are heavier
than ordinary source-level smoke checks.
