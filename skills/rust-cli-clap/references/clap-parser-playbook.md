# Clap Parser Playbook

## API Choice

Prefer derive for stable, statically known command trees:

- Keep the root parser small and route work through subcommand handler functions.
- Put repeated option groups in flattened structs.
- Use typed enums with `ValueEnum` instead of stringly-typed mode flags.
- Use `ArgGroup` for true mutual exclusion or required-one-of contracts.
- Use `value_parser!` and domain parsers for validated numbers, durations, URLs, and IDs.

Use the builder API when:

- Commands are plugin-discovered or generated from runtime metadata.
- Experimental commands need hidden aliases or feature-gated registration.
- The command tree is easier to audit as data than as nested derive structs.

Avoid mixing derive and builder without a clear boundary. If a derive parser needs heavy post-processing, move that logic into a typed config normalization layer instead of hiding it in parser attributes.

## Argument Design

- Prefer explicit long flags for scripts and stable short flags only for high-frequency terminal use.
- Keep positional arguments rare and obvious. Once there are multiple optional positionals, switch to named flags.
- Use `default_value_t` only when the default is part of the public contract. Otherwise compute defaults after merging config/env/CLI sources.
- Use `Option<T>` for absence and a separate enum for semantic modes. Avoid sentinel strings such as `"auto"` unless `auto` is a real mode.
- For path inputs, accept `PathBuf`; canonicalize only when required, and report the original path in user-facing errors.

## Precedence Model

State precedence explicitly in docs and tests. A solid default:

1. CLI flags
2. Environment variables
3. Config files
4. Project defaults
5. Built-in defaults

Do not let `clap` environment values silently bypass config validation. Parse and normalize into one domain config before execution.

## Help and Discovery

- Use command names as verbs: `sync`, `check`, `init`, `serve`.
- Keep help examples current by testing them.
- Generate shell completions and manpages for shipped CLIs when the distribution channel supports them.
- Hide truly internal flags with `hide = true`, but avoid undocumented "support-only" behavior for public workflows.

## Migration Notes

When upgrading `clap`, check:

- Derive attribute renames or semantic changes.
- Color/styling behavior.
- Error message snapshots.
- Shell completion output.
- `env` feature availability and variable naming.

Use official `clap` docs and changelogs for version-specific behavior before changing public parser contracts.
