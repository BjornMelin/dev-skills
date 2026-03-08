# Tooling and Verification

## Rules

1. Use the repo-native package manager and task runner detected from the repo, not a guessed default.
2. Read `AGENTS.md` before choosing format, lint, typecheck, test, or build commands.
3. If command usage is unclear, run `--help` first.
4. Do not expand into cross-domain workflows until the platform lane is known.

## Shared Preflight

- run the repo inventory first
- confirm the active lane
- only then load heavier references or specialized skills

