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

Run after changing `crates/codex-dev-core/`, `crates/codex-dev/`, root Cargo
files, or the `codex-dev` architecture/spec docs:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev-core
cargo check -p codex-dev
cargo test -p codex-dev-core
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- --json policy manifest
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

The task capsule smoke below covers `evidence append` and the follow-up
`capsule status` evidence summary against a real fixture capsule.

Run after changing `crates/codex-dev-tui/` or TUI docs:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-tui --all-targets -- -D warnings
cargo check -p codex-dev-tui
cargo test -p codex-dev-tui
```

Task capsule smoke:

```bash
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init --title "validation smoke" --branch validation/smoke --root "$tmp" --id validation-smoke --created-at 2026-05-09T04:00:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule status "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule render "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json evidence append --capsule "$tmp/validation-smoke" --kind decision --summary "fixture decision" --source-id validation:smoke --actor codex --tool codex-dev --confidence 95 --at 2026-05-09T04:30:00Z
cargo run -q -p codex-dev -- --json capsule status "$tmp/validation-smoke"
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
cargo run -q -p codex-dev-tui -- --capsule "$tmp/validation-smoke" --render-once --width 100 --height 24
```

Policy gate execution is explicit. Use `--execute` only when you intend to run
the repo-native commands from the manifest; the default dry run records the
planned gate snapshot in the capsule without running commands.
Execution discovers the repository root from the current directory or capsule
path. Pass `--repo-root <path>` when running an installed binary from outside
the repository.
The `codex_dev` policy profile covers core `codex-dev` CLI gates only. Use this
runbook for the broader human validation matrix, including TUI render smoke,
bootstrap packs, subagent templates, and research gates.

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
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts subagents/hardened-codex/scripts tools/bootstrap
```

## Bootstrap Packs

Validate pack manifests, render into temp directories, and prove ignored local
subagent boundaries stay ignored:

```bash
python3 tools/bootstrap/render_bootstrap_pack.py --validate
tmp=$(mktemp -d)
python3 tools/bootstrap/render_bootstrap_pack.py --pack codex-agent-repo --out "$tmp/codex" --repo-name codex-smoke --generated-at 2026-05-09T06:00:00Z
python3 tools/bootstrap/render_bootstrap_pack.py --pack rust-cli-agent-repo --out "$tmp/rust" --repo-name rust-smoke --primary-language rust --generated-at 2026-05-09T06:00:00Z
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --validate-release-manifest
PYTHONDONTWRITEBYTECODE=1 python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/hardened-codex/agents
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --dry-run
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --validate-sources
git check-ignore -v subagents/hardened-codex/overlays.local.json subagents/hardened-codex/roles.local.json subagents/hardened-codex/agents/overlays/private-repo/private_repo_reviewer.toml
git diff --check
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
- memory-derived guidance is verified against current tracked authority or
  clearly labeled historical/`UNVERIFIED`;
- stale smoke evidence from prior runs is rerun before it is used as acceptance
  evidence;
- docs do not include secrets, local tokens, or committed run ledgers.

## Full Local Gate

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev-core
cargo check -p codex-dev
cargo test -p codex-dev-core
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- --json policy manifest
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
cargo clippy -p codex-dev-tui --all-targets -- -D warnings
cargo check -p codex-dev-tui
cargo test -p codex-dev-tui
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init --title "validation smoke" --branch validation/smoke --root "$tmp" --id validation-smoke --created-at 2026-05-09T04:00:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule status "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule render "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json evidence append --capsule "$tmp/validation-smoke" --kind decision --summary "fixture decision" --source-id validation:smoke --actor codex --tool codex-dev --confidence 95 --at 2026-05-09T04:30:00Z
cargo run -q -p codex-dev -- --json capsule status "$tmp/validation-smoke"
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
cargo run -q -p codex-dev-tui -- --capsule "$tmp/validation-smoke" --render-once --width 100 --height 24
python3 tools/bootstrap/render_bootstrap_pack.py --validate
tmp_bootstrap=$(mktemp -d)
python3 tools/bootstrap/render_bootstrap_pack.py --pack codex-agent-repo --out "$tmp_bootstrap/codex" --repo-name codex-smoke --generated-at 2026-05-09T06:00:00Z
python3 tools/bootstrap/render_bootstrap_pack.py --pack rust-cli-agent-repo --out "$tmp_bootstrap/rust" --repo-name rust-smoke --primary-language rust --generated-at 2026-05-09T06:00:00Z
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
cargo run -q -p codex-research -- --json doctor
cargo run -q -p codex-research -- --json eval
cargo run -q -p codex-research -- --json eval --task evidence-claims-cited --strict
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts subagents/hardened-codex/scripts tools/bootstrap
python3 tools/docs/check_links.py docs README.md AGENTS.md
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents skills/subspawn/templates/agents subagents/hardened-codex/agents
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --validate-release-manifest
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --dry-run
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --validate-sources
git check-ignore -v subagents/hardened-codex/overlays.local.json subagents/hardened-codex/roles.local.json subagents/hardened-codex/agents/overlays/private-repo/private_repo_reviewer.toml
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan --preset research --task "validation smoke" --scope "docs and template metadata" --json
python3 tools/eval/skill_subagent_eval.py --json
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
git diff --check
```
