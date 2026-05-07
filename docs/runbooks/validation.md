# Validation Runbook

Use this after editing docs, skills, templates, Python helpers, or Rust code.

## Rust CLI

Run after any change under `crates/codex-research/` or root Cargo files:

```bash
cargo fmt --all --check
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
```

CLI smoke:

```bash
codex-research --json doctor
codex-research --json eval
codex-research eval --list
codex-research --json eval --task evidence-claims-cited --strict
codex-research --json plan "validation smoke" --profile quick
tmp=$(mktemp -d)
codex-research --json run init validation-smoke --profile quick --topic github --out "$tmp/run.json"
codex-research --json run debit --run "$tmp/run.json" --provider github --count 1 --note validation
codex-research --json run status --run "$tmp/run.json"
```

Optional live readiness:

```bash
codex-research --json eval --live
```

Use `cargo run -q -p codex-research -- ...` for these commands before the new
binary is installed locally. The embedded default eval suite is sourced from
`crates/codex-research/evals/research/core.json` at build time and covers
routing, privacy, budgets, cited claims, and report shape.

## Skills

Validate one skill:

```bash
python3 tools/skill/quick_validate.py skills/<skill-name>
```

Validate all skills:

```bash
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
```

Package changed skills:

```bash
python3 tools/skill/package_skill.py skills/deep-researcher skills/dist
python3 tools/skill/package_skill.py skills/subagent-creator skills/dist
python3 tools/skill/package_skill.py skills/subspawn skills/dist
```

## Python Helpers

```bash
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts
```

## Subagent Templates

Validate bundled templates:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate \
  skills/deep-researcher/templates/agents \
  skills/subagent-creator/templates/agents
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset research \
  --task "validation smoke" \
  --scope "docs and template metadata" \
  --json
```

The validator also enforces evidence-return headings for research-oriented
custom agents such as `deep_researcher`, `github_researcher`,
`context7_researcher`, `openai_docs_researcher`, `source_validator`, and
`citation_auditor`.

Validate global installed templates:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate ~/.codex/agents
```

Install dry-run:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target project --project-dir /tmp/deep-researcher-smoke --dry-run
python3 skills/subagent-creator/scripts/subagent_creator.py smoke --pack docs
```

## Docs

Docs currently use a command checklist instead of a separate docs linter.

Required checks:

```bash
! rg -n "TO[D]O|FIX[M]E" docs README.md AGENTS.md
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

Manual checks:

- intentional `UNVERIFIED` mentions are policy examples, not unresolved docs
  markers;
- generated-output references such as `target/` and `skills/dist/` are policy
  notes, not tracked artifacts;
- docs/index.md links every new major doc section;
- README links docs/index.md and the main guides;
- AGENTS.md lists the validation commands affected by the change;
- command examples match current CLI help;
- docs do not include secrets, local tokens, or committed run ledgers.

## Full Local Gate

```bash
cargo fmt --all --check
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
cargo run -q -p codex-research -- --json doctor
cargo run -q -p codex-research -- --json eval
cargo run -q -p codex-research -- --json eval --task evidence-claims-cited --strict
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts
python3 tools/docs/check_links.py docs README.md AGENTS.md
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents skills/subspawn/templates/agents
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan --preset research --task "validation smoke" --scope "docs and template metadata" --json
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
git diff --check
```
