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

## codex-dev Operating Layer

Run after changing `crates/codex-dev/`, root Cargo files, or the `codex-dev`
architecture/spec docs:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- --json policy manifest
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

Task capsule smoke:

```bash
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init --title "validation smoke" --branch validation/smoke --root "$tmp" --id validation-smoke --created-at 2026-05-09T04:00:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule status "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule render "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json policy run --capsule "$tmp/validation-smoke" --checked-at 2026-05-09T05:00:00Z
cat > "$tmp/pr-snapshot.json" <<'JSON'
{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "url": "https://github.com/BjornMelin/dev-skills/pull/25",
  "state": "OPEN",
  "checks": [
    {"name": "fixture", "status": "COMPLETED", "conclusion": "SUCCESS"}
  ],
  "review_threads": {"unresolved": 0}
}
JSON
cargo run -q -p codex-dev -- --json pr record --capsule "$tmp/validation-smoke" --source "$tmp/pr-snapshot.json" --checked-at 2026-05-09T05:00:00Z
cargo run -q -p codex-dev -- pr status --capsule "$tmp/validation-smoke"
```

Policy gate execution is explicit. Use `--execute` only when you intend to run
the repo-native commands from the manifest; the default dry run records the
planned gate snapshot in the capsule without running commands.
Execution discovers the repository root from the current directory or capsule
path. Pass `--repo-root <path>` when running an installed binary from outside
the repository.

Keep `codex-research` gates scoped to research changes.

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
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts subagents/hardened-codex/scripts
```

## Subagent Templates

Validate bundled templates:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate \
  skills/deep-researcher/templates/agents \
  skills/subagent-creator/templates/agents \
  skills/subspawn/templates/agents \
  subagents/hardened-codex/agents
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset research \
  --task "validation smoke" \
  --scope "docs and template metadata" \
  --json
python3 tools/eval/skill_subagent_eval.py --json
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
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- --json policy manifest
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init --title "validation smoke" --branch validation/smoke --root "$tmp" --id validation-smoke --created-at 2026-05-09T04:00:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule status "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule render "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json policy run --capsule "$tmp/validation-smoke" --checked-at 2026-05-09T05:00:00Z
cat > "$tmp/pr-snapshot.json" <<'JSON'
{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "url": "https://github.com/BjornMelin/dev-skills/pull/25",
  "state": "OPEN",
  "checks": [
    {"name": "fixture", "status": "COMPLETED", "conclusion": "SUCCESS"}
  ],
  "review_threads": {"unresolved": 0}
}
JSON
cargo run -q -p codex-dev -- --json pr record --capsule "$tmp/validation-smoke" --source "$tmp/pr-snapshot.json" --checked-at 2026-05-09T05:00:00Z
cargo run -q -p codex-dev -- pr status --capsule "$tmp/validation-smoke"
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
cargo run -q -p codex-research -- --json doctor
cargo run -q -p codex-research -- --json eval
cargo run -q -p codex-research -- --json eval --task evidence-claims-cited --strict
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts subagents/hardened-codex/scripts
python3 tools/docs/check_links.py docs README.md AGENTS.md
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents skills/subspawn/templates/agents subagents/hardened-codex/agents
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan --preset research --task "validation smoke" --scope "docs and template metadata" --json
python3 tools/eval/skill_subagent_eval.py --json
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
git diff --check
```
