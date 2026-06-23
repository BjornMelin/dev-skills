# gsap-audit Reference

`gsap-audit` is the optional companion CLI for the standalone `gsap` skill under
`skills/gsap`. It statically audits GSAP usage in JS/TS/JSX/TSX source and
reports anti-patterns such as trial imports, dev-only plugins shipped to
production, ScrollTrigger debug config, GSAP-2 call signatures, layout-property
animation, missing plugin registration, and React `useGSAP`/`gsap.context`
misuse.

The tool parses each supported source file with [oxc](https://oxc.rs/) and runs
semantic analysis, so findings are based on real AST and scope information rather
than text matching. It performs no network calls, reads no configuration, and
does not modify files.

## Crates

- `crates/gsap-audit-core`: oxc-based static-analysis library (parser, semantic
  walk, rule catalog, scanner, and output formatting). All analysis logic and
  tests live here.
- `crates/gsap-audit`: thin Clap binary `gsap-audit` over `gsap-audit-core`.

## Installation

From the repository root:

```bash
cargo build -p gsap-audit
cargo install --path crates/gsap-audit --locked --force
cargo run -q -p gsap-audit -- --help
```

Use [Global CLI Workflow](../runbooks/global-cli-workflow.md) for the
install/update workflow, completions, and isolated install smokes. The binary is
optional: the `gsap` skill is fully usable without it, and the CLI is for
auditing an existing GSAP codebase.

## Commands

```text
gsap-audit <command>
```

Top-level commands:

- `scan`: walk a directory tree, parse every supported source file, and report
  findings.
- `doctor`: print the tool version and the full rule catalog.
- `completions`: generate a shell completion script.

## scan

Walk the given root, parse every supported source file, and report GSAP
anti-patterns:

```bash
gsap-audit scan --root . --format markdown
gsap-audit scan --root ./src --format json --categories react,scrolltrigger
```

Options:

- `--root <PATH>`: directory to scan. Default `.`.
- `--format <markdown|json>`: output format. Default `markdown`.
- `--categories <CSV>`: comma-separated subset of rule categories to run
  (`core`, `react`, `scrolltrigger`, `timeline`, `plugins`, `performance`,
  `utils`). Default runs every category.
- `--output <PATH>`: write the report to this file instead of stdout.
- `--max-files <N>`: maximum number of files to analyze before truncating.
  Default `5000`. When the cap is hit, the report sets `truncated: true`.

## doctor

Print the tool name and version plus every rule (id, category, severity, and a
short summary) as markdown or JSON:

```bash
gsap-audit doctor
gsap-audit doctor --format json
```

`doctor` is the authoritative source for the current rule set. Because the rule
catalog evolves, run `gsap-audit doctor` to see the exact rule ids and
severities your installed build ships rather than relying on a hardcoded table in
this doc.

Markdown output is a `# gsap-audit rule catalog (v<version>)` heading followed by
an `id | category | severity` table. JSON output is a `{ "rules": [...] }` object
where each rule carries `id`, `category`, `severity`, `confidence`, and
`summary`.

## completions

Generate a shell completion script from the canonical Clap command definition:

```bash
gsap-audit completions zsh > ~/.local/share/dev-skills/completions/zsh/_gsap-audit
gsap-audit completions bash
gsap-audit completions fish
```

Supported shells are `bash`, `elvish`, `fish`, `powershell`, and `zsh`. The
command writes the completion script to stdout and does not modify shell startup
files.

## Rule Categories

Rules are grouped into stable categories so `--categories` can scope a scan:

- `core`: core GSAP API misuse, including obsolete `gsap-trial` imports and
  animating layout-affecting properties instead of transforms.
- `react`: React/Next integration issues such as unregistered `useGSAP`,
  running GSAP during SSR, unscoped selectors, and contexts that never revert.
- `scrolltrigger`: ScrollTrigger configuration problems such as debug markers
  left in production and conflicting `scrub`/`toggleActions` settings.
- `timeline`: timeline and sequencing issues such as GSAP-2 duration-as-second
  argument call signatures.
- `plugins`: plugin lifecycle issues such as dev-only plugins (for example
  GSDevTools) shipped in source and plugins used without registration.
- `performance`: performance hazards such as disabled lag smoothing.
- `utils`: `gsap.utils` helper issues.

This doc describes rules only at the category level on purpose: the per-rule id
set is maintained in the crate and can change between builds. Run
`gsap-audit doctor` for the authoritative current rule list.

## Output Formats

Both `scan` and `doctor` support `--format markdown` (default) and
`--format json`.

`scan --format markdown` produces a human-readable report grouped for review.
`scan --format json` produces a stable machine-readable object:

```json
{
  "tool": "gsap-audit",
  "version": "0.1.0",
  "files_scanned": 0,
  "truncated": false,
  "findings": [],
  "summary": {
    "total": 0,
    "by_severity": {},
    "by_category": {}
  }
}
```

Each entry in `findings` describes one anti-pattern with its rule id, category,
severity, confidence, location, and message. The `summary` object reports the
total finding count plus counts grouped by severity and by category.

## Exit Codes

`gsap-audit scan` uses exit codes as a CI-friendly contract:

- `0`: clean, or only low-severity findings were reported.
- `2`: at least one high-severity finding is present.
- `1`: a usage or IO error occurred (for example an unreadable root or invalid
  arguments).

This lets a CI gate fail on high-severity GSAP anti-patterns while still
surfacing lower-severity findings as advisory output.

## Validation

Run after changing `crates/gsap-audit-core/` or `crates/gsap-audit/`:

```bash
cargo fmt --all --check
cargo clippy -p gsap-audit-core -p gsap-audit --all-targets -- -D warnings
cargo check -p gsap-audit-core -p gsap-audit
cargo test -p gsap-audit-core -p gsap-audit
cargo run -q -p gsap-audit -- doctor
```

Use [Validation](../runbooks/validation.md) for the canonical local gate matrix
and [Local Release and Supply Chain](../runbooks/local-release-supply-chain.md)
for the oxc dependency-tree review and package dry-run evidence.
