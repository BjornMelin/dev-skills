# motion-token-audit Reference

`motion-token-audit` is a static auditor for cross-stack motion token drift. It
discovers the shared motion-token vocabulary emitted by
`skills/design-motion-system/scripts/scaffold_motion_tokens.py`, then checks CSS,
Reanimated, GSAP, and Motion React usage for hardcoded duration, easing, and
spring literals.

The tool parses JS/TS/JSX/TSX with [oxc](https://oxc.rs/) and scans CSS/SCSS
text for motion declarations. It performs no network calls and does not modify
files. Treat findings as review leads: static analysis can prove duplicated
literals, but only a human can decide whether a new orphan value should become a
token or be removed.

## Crates

- `crates/motion-token-audit-core`: token discovery, oxc analysis, CSS scanner,
  rule catalog, output formatting, and tests.
- `crates/motion-token-audit`: thin Clap binary `motion-token-audit` over
  `motion-token-audit-core`.

## Installation

From the repository root:

```bash
cargo build -p motion-token-audit
cargo install --path crates/motion-token-audit --locked --force
cargo run -q -p motion-token-audit -- --help
```

## Commands

```text
motion-token-audit <command>
```

Top-level commands:

- `scan`: walk a directory tree, discover motion tokens, and report drift or
  orphan hardcoded literals.
- `doctor`: print the tool version and the full rule catalog.
- `completions`: generate a shell completion script.

## scan

```bash
motion-token-audit scan --root . --format markdown
motion-token-audit scan --root ./src --format json --categories tokens-css,tokens-reanimated
```

Options:

- `--root <PATH>`: directory to scan. Default `.`.
- `--format <markdown|json>`: output format. Default `markdown`.
- `--categories <CSV>`: comma-separated subset of rule categories to run
  (`ssot`, `tokens-css`, `tokens-reanimated`, `tokens-gsap`, `tokens-react`,
  `tokens-r3f`). Default runs every category.
- `--output <PATH>`: write the report to this file instead of stdout.
- `--max-files <N>`: maximum number of files to analyze before truncating.
  Default `5000`.

`scan` reports hardcoded motion literals as:

- `drift`: the literal equals a known token value but bypasses the token.
- `orphan`: the literal has no matching token and may be a vocabulary gap.

The report also includes per-stack tokenization coverage: tokenized references,
hardcoded literals, coverage percent, and drift/orphan counts.

## doctor

```bash
motion-token-audit doctor
motion-token-audit doctor --format json
```

`doctor` is the authoritative source for rule ids, categories, and severities in
the installed build.

## completions

```bash
motion-token-audit completions zsh > ~/.local/share/dev-skills/completions/zsh/_motion-token-audit
motion-token-audit completions bash
motion-token-audit completions fish
```

Supported shells are `bash`, `elvish`, `fish`, `powershell`, and `zsh`.

## Rule Categories

- `ssot`: missing token-source-of-truth detection.
- `tokens-css`: CSS/SCSS transition and animation duration/easing literals.
- `tokens-reanimated`: `withTiming`, `withDelay`, `withSpring`, and
  `Easing.bezier` literals.
- `tokens-gsap`: GSAP tween `duration` and `ease` literals.
- `tokens-react`: Motion React `transition` duration and ease literals.
- `tokens-r3f`: reserved R3F category for lower-confidence damp/lerp checks.

Run `motion-token-audit doctor` for the current per-rule catalog.

## Output Formats

Both `scan` and `doctor` support `--format markdown` (default) and
`--format json`.

`scan --format json` returns:

```json
{
  "tool": "motion-token-audit",
  "version": "0.1.0",
  "files_scanned": 0,
  "truncated": false,
  "coverage": [],
  "findings": [],
  "summary": {
    "total": 0,
    "by_severity": {},
    "by_category": {}
  }
}
```

Each finding includes rule id, category, severity, confidence, file, line,
column, message, and suggestion.

## Exit Codes

- `0`: clean, or only low-severity findings were reported.
- `2`: at least one medium- or high-severity finding is present.
- `1`: a usage or IO error occurred.

## Validation

Run after changing `crates/motion-token-audit-core/` or
`crates/motion-token-audit/`:

```bash
cargo fmt --all --check
cargo clippy -p motion-token-audit-core -p motion-token-audit --all-targets -- -D warnings
cargo check -p motion-token-audit-core -p motion-token-audit
cargo test -p motion-token-audit-core -p motion-token-audit
cargo run -q -p motion-token-audit -- doctor
```
