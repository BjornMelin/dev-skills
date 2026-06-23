# expo-motion-audit Reference

`expo-motion-audit` is the optional companion CLI for the standalone
`expo-motion` skill under `skills/expo-motion`. It statically audits Expo/React
Native motion code (Reanimated 4 and Worklets) in JS/TS/JSX/TSX source, plus the
project's babel and app config, and reports anti-patterns such as deprecated
`runOnJS`/`runOnUI`, shared-value reassignment, JS-thread `.value` access,
missing `'worklet'` directives, layout-property animation, infinite repeats with
no reduced-motion guard, missing `cancelAnimation`, missing reduced-motion
handling, a missing or misordered worklets babel plugin, the deprecated
reanimated babel plugin, and the New Architecture being disabled.

The tool parses each supported source file with [oxc](https://oxc.rs/) and runs
semantic analysis, so source findings are based on real AST and scope
information rather than text matching. Config files are parsed structurally
(babel config via oxc as CommonJS, app config via JSON). It performs no network
calls and does not modify files.

## Crates

- `crates/expo-motion-audit-core`: oxc-based static-analysis library (parser,
  semantic walk, rule catalog, config analysis, scanner, and output
  formatting). All analysis logic and tests live here.
- `crates/expo-motion-audit`: thin Clap binary `expo-motion-audit` over
  `expo-motion-audit-core`.

## Installation

From the repository root:

```bash
cargo build -p expo-motion-audit
cargo install --path crates/expo-motion-audit --locked --force
cargo run -q -p expo-motion-audit -- --help
```

Use [Global CLI Workflow](../runbooks/global-cli-workflow.md) for the
install/update workflow, completions, and isolated install smokes. The binary is
optional: the `expo-motion` skill is fully usable without it, and the CLI is for
auditing an existing Expo/React Native codebase.

## Commands

```text
expo-motion-audit <command>
```

Top-level commands:

- `scan`: walk a directory tree, parse every supported source file plus babel/app
  config, and report findings.
- `doctor`: print the tool version and the full rule catalog.
- `completions`: generate a shell completion script.

## scan

Walk the given root, parse every supported source file plus `babel.config.js`,
`app.json`, and `app.config.json`, and report findings:

```bash
expo-motion-audit scan --root . --format markdown
expo-motion-audit scan --root ./app --format json --categories worklets-threading,config
```

Options:

- `--root <PATH>`: directory to scan. Default `.`.
- `--format <markdown|json>`: output format. Default `markdown`.
- `--categories <CSV>`: comma-separated subset of rule categories to run
  (`reanimated-core`, `worklets-threading`, `gestures`, `layout`,
  `accessibility`, `lifecycle`, `config`). Default runs every category.
- `--output <PATH>`: write the report to this file instead of stdout.
- `--max-files <N>`: maximum number of files to analyze before truncating.
  Default `5000`. When the cap is hit, the report sets `truncated: true`.

The walk includes `.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs`, `.mts`, `.cts`
source files plus the supported config files, and skips `node_modules`, `.git`,
`dist`, `build`, `.expo`, `android`, `ios`, `target`, and `coverage`.

## doctor

Print the tool name and version plus every rule (id, category, severity, and a
short summary) as markdown or JSON:

```bash
expo-motion-audit doctor
expo-motion-audit doctor --format json
```

`doctor` is the authoritative source for the current rule set. Because the rule
catalog evolves, run `expo-motion-audit doctor` to see the exact rule ids and
severities your installed build ships rather than relying on a hardcoded table
in this doc.

Markdown output is a `# expo-motion-audit rule catalog (v<version>)` heading
followed by an `id | category | severity` table. JSON output is a
`{ "rules": [...] }` object where each rule carries `id`, `category`,
`severity`, `confidence`, and `summary`.

## completions

Generate a shell completion script from the canonical Clap command definition:

```bash
expo-motion-audit completions zsh > ~/.local/share/dev-skills/completions/zsh/_expo-motion-audit
expo-motion-audit completions bash
expo-motion-audit completions fish
```

Supported shells are `bash`, `elvish`, `fish`, `powershell`, and `zsh`. The
command writes the completion script to stdout and does not modify shell startup
files.

## Rule Categories

Rules are grouped into stable categories so `--categories` can scope a scan:

- `reanimated-core`: core Reanimated misuse, including animating layout
  properties (width/height/top/left/margin) in an animated style instead of
  transforms, and reassigning a `useSharedValue` binding directly instead of
  writing to `.value`.
- `worklets-threading`: the worklets/threading model — deprecated
  `runOnJS`/`runOnUI` (replaced by `scheduleOnRN`/`scheduleOnUI` from
  `react-native-worklets`), reading or writing a shared value's `.value` on the
  JS thread, bridging back to JS inside a per-frame gesture callback, and
  extracted named functions passed to animated hooks/gestures without a
  `'worklet'` directive.
- `gestures`: gesture-handler integration issues (reserved for future rules).
- `layout`: layout and looping animation issues such as an infinite
  `withRepeat(anim, -1, ...)` with no reduced-motion guard.
- `accessibility`: motion accessibility, such as animating with Reanimated while
  never referencing a reduced-motion API.
- `lifecycle`: animation lifecycle issues such as animating a shared value
  without ever calling `cancelAnimation` for teardown.
- `config`: project configuration issues — a missing or misordered
  `react-native-worklets/plugin` in `babel.config.js`, the deprecated
  `react-native-reanimated/plugin`, and the New Architecture being disabled
  (or omitted before Expo SDK 53) while the project uses Reanimated 4.

This doc describes rules only at the category level on purpose: the per-rule id
set is maintained in the crate and can change between builds. Run
`expo-motion-audit doctor` for the authoritative current rule list.

### Heuristic limitations

Several rules are deliberately file-scoped heuristics rather than full data-flow
analyses, and report at medium confidence:

- `accessibility.missing-reduced-motion` and `lifecycle.missing-cancel-animation`
  detect the *absence* of an expected token anywhere in the file. They cannot
  see handling that lives in a parent component, so they may false-positive when
  the concern is addressed elsewhere.
- `worklets-threading.value-access-on-js` climbs the AST to decide whether a
  `.value` access is on the JS thread; it does not model values read inside a
  plain helper that is only ever called from a worklet.
- `worklets-threading.missing-worklet` targets extracted *named* function
  expressions, because the babel plugin auto-workletizes inline arrows in these
  positions. A project that workletizes via a wrapper the static check cannot
  see may be over-reported.
- Babel config analysis is structural: a `plugins` array assembled dynamically
  (spread, conditional, computed) or a non-object/dynamic `module.exports`
  yields an informational low `config.unable-to-analyze` finding rather than a
  false high-severity one. A dynamic `app.config.js`/`.ts` is likewise reported
  as informational because only the static `app.json`/`app.config.json` forms
  are parsed.

## Output Formats

Both `scan` and `doctor` support `--format markdown` (default) and
`--format json`.

`scan --format markdown` produces a human-readable report grouped by file.
`scan --format json` produces a stable machine-readable object:

```json
{
  "tool": "expo-motion-audit",
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

Each entry in `findings` describes one anti-pattern with its rule `id`,
`category`, `severity`, `confidence`, `file`, `line`, `column`, `message`, and
`suggestion`. The `summary` object reports the total finding count plus counts
grouped by severity and by category.

## Exit Codes

`expo-motion-audit scan` uses exit codes as a CI-friendly contract:

- `0`: clean, or only low-severity findings were reported.
- `2`: at least one medium- or high-severity finding is present.
- `1`: a usage or IO error occurred (for example an unreadable root or invalid
  arguments).

This lets a CI gate fail on actionable motion anti-patterns while still
surfacing low-severity (informational) findings as advisory output.

## Validation

Run after changing `crates/expo-motion-audit-core/` or `crates/expo-motion-audit/`:

```bash
cargo fmt --all --check
cargo clippy -p expo-motion-audit-core -p expo-motion-audit --all-targets -- -D warnings
cargo check -p expo-motion-audit-core -p expo-motion-audit
cargo test -p expo-motion-audit-core -p expo-motion-audit
cargo run -q -p expo-motion-audit -- doctor
```

Use [Validation](../runbooks/validation.md) for the canonical local gate matrix
and [Local Release and Supply Chain](../runbooks/local-release-supply-chain.md)
for the oxc dependency-tree review and package dry-run evidence.
