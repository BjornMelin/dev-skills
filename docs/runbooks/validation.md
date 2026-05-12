# Validation Runbook

Use this after editing docs, skills, templates, Python helpers, or Rust code.

## Validation Matrix Ownership

The Rust policy profiles in `crates/codex-dev/src/lib.rs` are the canonical
validation matrix for `codex-dev policy manifest`. Markdown snippets marked
with `codex-dev:policy-manifest-*` are machine-owned mirrors of that Rust
source and are checked by:

```bash
cargo run -q -p codex-dev -- --json policy docs-check
```

Unmarked prose and workflow notes are human-owned documentation.

## Local Release Supply Chain

Use [Local Release and Supply Chain](local-release-supply-chain.md) before
global CLI install handoff, release assets, or Cargo metadata changes.

```bash
cargo metadata --locked --no-deps --format-version 1
cargo tree -d --target all
cargo deny check bans licenses sources
cargo deny check advisories
cargo audit
cargo package --list -p codex-dev-core
cargo package --list -p codex-dev
cargo package --list -p codex-dev-tui
cargo package --list -p codex-research
```

`cargo deny check advisories` and `cargo audit` are networked release evidence
unless their advisory databases are already cached. The manifest-backed
`release` and `full_local` policy profiles keep only local, non-secret
supply-chain gates by default.

## Rust CLI

Run after any change under `crates/codex-research/` or root Cargo files:

```bash
cargo fmt --all --check
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
cargo run -q -p codex-research -- completions zsh >/tmp/codex-research.zsh
cargo run -q -p codex-research -- manpage >/tmp/codex-research.1
```

CLI smoke:

```bash
codex-research --json doctor
codex-research --json eval
codex-research eval --list
codex-research --json eval --task evidence-claims-cited --strict
codex-research --json eval --task evidence-bundle-closeout-shape --strict
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
routing, privacy, budgets, cited claims, report shape, and bundle closeout
shape.

## codex-dev Operating Layer

Run after changing `crates/codex-dev-core/`, `crates/codex-dev/`, root Cargo
files, or the `codex-dev` architecture/spec docs:

```bash
cargo fmt --all --check
cargo metadata --locked --no-deps --format-version 1
cargo tree -d --target all
cargo deny check bans licenses sources
cargo package --list -p codex-dev-core
cargo package --list -p codex-dev
cargo package --list -p codex-dev-tui
cargo package --list -p codex-research
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev-core
cargo check -p codex-dev
cargo test -p codex-dev-core
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- completions zsh >/tmp/codex-dev.zsh
cargo run -q -p codex-dev -- manpage >/tmp/codex-dev.1
# codex-dev:policy-manifest-smoke:start
cargo run -q -p codex-dev -- --json policy manifest --profile codex_dev
cargo run -q -p codex-dev -- --json policy manifest --profile full_local
# codex-dev:policy-manifest-smoke:end
cargo run -q -p codex-dev -- --json policy docs-check
cargo run -q -p codex-dev -- --json local doctor
cargo run -q -p codex-dev -- --json local status
cargo run -q -p codex-dev -- --json skills inventory
cargo run -q -p codex-dev -- --json task list
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
cargo run -q -p codex-dev -- --json pr agent --help
cargo run -q -p codex-dev -- --json pr agent-action --help
cargo run -q -p codex-dev -- --json pr readiness --help
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init --title "validation smoke" --branch validation/smoke --root "$tmp" --id validation-smoke --created-at 2026-05-09T04:00:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule status "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule render "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json policy run --capsule "$tmp/validation-smoke" --checked-at 2026-05-09T05:00:00Z
cat > "$tmp/subspawn-plan.json" <<'JSON'
{
  "task": "validation smoke review",
  "mode": "read-only",
  "scope": "fixture capsule",
  "wait_policy": "strict",
  "rendezvous_required": true,
  "roles": [
    {"name": "reviewer"}
  ],
  "prompts": [
    {"role": "reviewer", "prompt": "Review the validation smoke capsule and report blockers."}
  ],
  "registry_issues": [],
  "duplicate_roles_ignored": {
    "test_runner": [
      "skills/subagent-creator/templates/agents/test_runner.toml",
      "skills/subspawn/templates/agents/test_runner.toml"
    ]
  }
}
JSON
cargo run -q -p codex-dev -- --json subagents record-plan --capsule "$tmp/validation-smoke" --batch-id validation-review --source "$tmp/subspawn-plan.json" --command "python3 skills/subspawn/scripts/subspawn_plan.py plan --preset review --json" --recorded-at 2026-05-09T05:10:00Z
cargo run -q -p codex-dev -- --json subagents record-outcome --capsule "$tmp/validation-smoke" --batch-id validation-review --role reviewer --status completed --summary "validation smoke reviewed" --disposition accepted --human-verified --source-id reviewer:validation-smoke --artifact "$tmp/subspawn-plan.json" --recorded-at 2026-05-09T05:20:00Z
cargo run -q -p codex-dev -- --json subagents record-synthesis --capsule "$tmp/validation-smoke" --batch-id validation-review --status completed --summary "subagent evidence smoke complete" --human-verified --source-id synthesis:validation-review --artifact "$tmp/subspawn-plan.json" --recorded-at 2026-05-09T05:30:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json evidence append --capsule "$tmp/validation-smoke" --kind decision --summary "fixture decision" --source-id validation:smoke --actor codex --tool codex-dev --confidence 95 --at 2026-05-09T05:40:00Z
cargo run -q -p codex-dev -- --json capsule status "$tmp/validation-smoke"
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

The task capsule smoke below covers `evidence append`, subagent plan, outcome,
and synthesis recording, PR evidence capture, and the follow-up `capsule status`
summary against a real fixture capsule.

Run after changing `crates/codex-dev-tui/` or TUI docs:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-tui --all-targets -- -D warnings
cargo check -p codex-dev-tui
cargo test -p codex-dev-tui
cargo run -q -p codex-dev-tui -- completions zsh >/tmp/codex-dev-tui.zsh
cargo run -q -p codex-dev-tui -- manpage >/tmp/codex-dev-tui.1
```

Task capsule smoke:

```bash
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init --title "validation smoke" --branch validation/smoke --root "$tmp" --id validation-smoke --created-at 2026-05-09T04:00:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule status "$tmp/validation-smoke"
cargo run -q -p codex-dev -- capsule render "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json policy run --capsule "$tmp/validation-smoke" --checked-at 2026-05-09T05:00:00Z
cat > "$tmp/subspawn-plan.json" <<'JSON'
{
  "task": "validation smoke review",
  "mode": "read-only",
  "scope": "fixture capsule",
  "wait_policy": "strict",
  "rendezvous_required": true,
  "roles": [
    {"name": "reviewer"}
  ],
  "prompts": [
    {"role": "reviewer", "prompt": "Review the validation smoke capsule and report blockers."}
  ],
  "registry_issues": [],
  "duplicate_roles_ignored": {
    "test_runner": [
      "skills/subagent-creator/templates/agents/test_runner.toml",
      "skills/subspawn/templates/agents/test_runner.toml"
    ]
  }
}
JSON
cargo run -q -p codex-dev -- --json subagents record-plan --capsule "$tmp/validation-smoke" --batch-id validation-review --source "$tmp/subspawn-plan.json" --command "python3 skills/subspawn/scripts/subspawn_plan.py plan --preset review --json" --recorded-at 2026-05-09T05:10:00Z
cargo run -q -p codex-dev -- --json subagents record-outcome --capsule "$tmp/validation-smoke" --batch-id validation-review --role reviewer --status completed --summary "validation smoke reviewed" --disposition accepted --human-verified --source-id reviewer:validation-smoke --artifact "$tmp/subspawn-plan.json" --recorded-at 2026-05-09T05:20:00Z
cargo run -q -p codex-dev -- --json subagents record-synthesis --capsule "$tmp/validation-smoke" --batch-id validation-review --status completed --summary "subagent evidence smoke complete" --human-verified --source-id synthesis:validation-review --artifact "$tmp/subspawn-plan.json" --recorded-at 2026-05-09T05:30:00Z
cargo run -q -p codex-dev -- --json capsule validate "$tmp/validation-smoke"
cargo run -q -p codex-dev -- --json evidence append --capsule "$tmp/validation-smoke" --kind decision --summary "fixture decision" --source-id validation:smoke --actor codex --tool codex-dev --confidence 95 --at 2026-05-09T05:40:00Z
cargo run -q -p codex-dev -- --json capsule status "$tmp/validation-smoke"
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
cargo run -q -p codex-dev-tui -- --root "$tmp" --render-once --width 100 --height 24
cargo run -q -p codex-dev-tui -- --capsule "$tmp/validation-smoke" --render-once --width 100 --height 24
```

## Global CLI Install And Artifact Smokes

Use [Global CLI Workflow](global-cli-workflow.md) after changes to binary
manifests, command shapes, completions, manpages, or local install
documentation.

```bash
cargo run -q -p codex-research -- completions zsh >/tmp/codex-research.zsh
cargo run -q -p codex-dev -- completions zsh >/tmp/codex-dev.zsh
cargo run -q -p codex-dev-tui -- completions zsh >/tmp/codex-dev-tui.zsh
cargo run -q -p codex-research -- manpage >/tmp/codex-research.1
cargo run -q -p codex-dev -- manpage >/tmp/codex-dev.1
cargo run -q -p codex-dev-tui -- manpage >/tmp/codex-dev-tui.1
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

Optional live PR-agent smoke, for branches with a GitHub PR and valid `gh`
authentication:

```bash
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json capsule init \
  --title "PR agent live smoke" \
  --branch "$(git branch --show-current)" \
  --root "$tmp" \
  --id pr-agent-live-smoke \
  --created-at "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cargo run -q -p codex-dev -- --json pr agent \
  --capsule "$tmp/pr-agent-live-smoke" \
  --repo BjornMelin/dev-skills \
  --number <pr-number>
cargo run -q -p codex-dev -- --json pr agent-action \
  --capsule "$tmp/pr-agent-live-smoke" \
  --repo BjornMelin/dev-skills \
  --number <pr-number> \
  --plan-id live-smoke-comment-plan \
  --action post-issue-comment \
  --body "dry-run hosted action smoke"
cargo run -q -p codex-dev -- --json pr readiness \
  --capsule "$tmp/pr-agent-live-smoke" \
  --repo BjornMelin/dev-skills \
  --number <pr-number> \
  --poll-attempts 1 \
  --poll-interval-seconds 0
cargo run -q -p codex-dev -- --json pr status --capsule "$tmp/pr-agent-live-smoke"
```

This smoke performs read-only hosted collection plus local capsule writes. The
`pr agent-action` and `pr readiness` commands are intentionally run without
`--apply`; they must not be used as evidence that hosted review comments were
resolved, failed jobs were rerun, or a PR was merged.

Policy gate execution is explicit. Use `--execute` only when you intend to run
the repo-native commands from the manifest; the default dry run records the
planned gate snapshot in the capsule without running commands.
Execution discovers the repository root from the current directory or capsule
path. Pass `--repo-root <path>` when running an installed binary from outside
the repository. If capsule-path and current-directory discovery point at
different repos, execution fails until `--repo-root` makes the target explicit.
Gate working directories are repo-relative and cannot escape the selected root.
Policy profiles are branch-selection helpers, not automatic release gates:

| Profile | Use for |
| --- | --- |
| `codex_dev` | `crates/codex-dev-core/`, `crates/codex-dev/`, and operating-layer docs |
| `codex_dev_tui` | `crates/codex-dev-tui/` changes and TUI docs |
| `codex_research` | `crates/codex-research/` and research CLI docs |
| `skills` | `skills/`, subagent templates, and Python helper changes |
| `bootstrap_install` | bootstrap packs and global subagent install sync changes |
| `docs` | docs-only updates |
| `release` | audited local release readiness before publishing/install handoff |
| `full_local` | broad local pre-release or high-risk cross-surface changes |

Each manifest gate declares its source, command, working directory, required
tools, network/secrets expectation, and failure interpretation. Built-in
profiles are local and do not require provider credentials; live provider checks
stay explicit in their owning runbooks. Executed gates marked `network` require
`--allow-network`; executed gates marked `secrets` require `--allow-secrets`.

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

The packager writes archive entries as `<skill-name>/...`, validates
`SKILL.md`, and skips common generated caches such as `__pycache__`, `*.pyc`,
`*.skill`, `.codex/`, and local tool caches. It rejects output directories
nested inside the source skill folder and skips symlinks so the bundle cannot
package itself or out-of-tree targets. `quick_validate.py` validates only
`SKILL.md` frontmatter. `tools/eval/skill_subagent_eval.py --json` is the
dedicated offline validator for catalog exposure, local skill links, tracked
generated-cache exclusion, helper syntax, `.skill` bundle shape, and
`agents/openai.yaml` metadata. Use `--strict` before publishing bundles when
ignored local dist artifacts must also be clean.

## Python Helpers

```bash
python3 -m compileall -q skills tools subagents/hardened-codex/scripts
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
for path in \
  subagents/hardened-codex/overlays.local.json \
  subagents/hardened-codex/roles.local.json \
  subagents/hardened-codex/agents/overlays/private-repo/private_repo_reviewer.toml; do
  git check-ignore -v -- "$path" >/dev/null || { echo "not ignored: $path" >&2; exit 1; }
done
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

`validate-roles` may report duplicate templates ignored when subspawn fallback
copies mirror canonical roles. Use
[Subagent Templates](../reference/subagent-templates.md) to distinguish expected
packaging duplicates from drift. When fallback copies change, manually compare
model, sandbox, edit permissions, safety posture, and required return sections
against the canonical owner; the current validators report duplicate paths but
do not fail on fallback parity drift.

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
cargo run -q -p codex-dev -- completions zsh >/tmp/codex-dev.zsh
cargo run -q -p codex-dev -- manpage >/tmp/codex-dev.1
# codex-dev:policy-manifest-all:start
cargo run -q -p codex-dev -- --json policy manifest --profile codex_dev
cargo run -q -p codex-dev -- --json policy manifest --profile codex_dev_tui
cargo run -q -p codex-dev -- --json policy manifest --profile codex_research
cargo run -q -p codex-dev -- --json policy manifest --profile skills
cargo run -q -p codex-dev -- --json policy manifest --profile bootstrap_install
cargo run -q -p codex-dev -- --json policy manifest --profile docs
cargo run -q -p codex-dev -- --json policy manifest --profile release
cargo run -q -p codex-dev -- --json policy manifest --profile full_local
# codex-dev:policy-manifest-all:end
cargo run -q -p codex-dev -- --json policy docs-check
cargo run -q -p codex-dev -- --json local doctor
cargo run -q -p codex-dev -- --json local status
cargo run -q -p codex-dev -- --json skills inventory
cargo run -q -p codex-dev -- --json task list
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
cargo run -q -p codex-dev -- --json pr agent --help
cargo run -q -p codex-dev -- --json pr agent-action --help
cargo run -q -p codex-dev -- --json pr readiness --help
cargo clippy -p codex-dev-tui --all-targets -- -D warnings
cargo check -p codex-dev-tui
cargo test -p codex-dev-tui
cargo run -q -p codex-dev-tui -- completions zsh >/tmp/codex-dev-tui.zsh
cargo run -q -p codex-dev-tui -- manpage >/tmp/codex-dev-tui.1
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
cargo run -q -p codex-dev-tui -- --root "$tmp" --render-once --width 100 --height 24
cargo run -q -p codex-dev-tui -- --capsule "$tmp/validation-smoke" --render-once --width 100 --height 24
python3 tools/bootstrap/render_bootstrap_pack.py --validate
tmp_bootstrap=$(mktemp -d)
python3 tools/bootstrap/render_bootstrap_pack.py --pack codex-agent-repo --out "$tmp_bootstrap/codex" --repo-name codex-smoke --generated-at 2026-05-09T06:00:00Z
python3 tools/bootstrap/render_bootstrap_pack.py --pack rust-cli-agent-repo --out "$tmp_bootstrap/rust" --repo-name rust-smoke --primary-language rust --generated-at 2026-05-09T06:00:00Z
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
cargo run -q -p codex-research -- completions zsh >/tmp/codex-research.zsh
cargo run -q -p codex-research -- manpage >/tmp/codex-research.1
cargo run -q -p codex-research -- --json doctor
cargo run -q -p codex-research -- --json eval
cargo run -q -p codex-research -- --json eval --task evidence-claims-cited --strict
cargo run -q -p codex-research -- --json eval --task evidence-bundle-closeout-shape --strict
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
python3 -m compileall -q skills tools subagents/hardened-codex/scripts
python3 tools/docs/check_links.py docs README.md AGENTS.md
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents skills/subspawn/templates/agents subagents/hardened-codex/agents
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --validate-release-manifest
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --dry-run
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --validate-sources
for path in \
  subagents/hardened-codex/overlays.local.json \
  subagents/hardened-codex/roles.local.json \
  subagents/hardened-codex/agents/overlays/private-repo/private_repo_reviewer.toml; do
  git check-ignore -v -- "$path" >/dev/null || { echo "not ignored: $path" >&2; exit 1; }
done
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan --preset research --task "validation smoke" --scope "docs and template metadata" --json
python3 tools/eval/skill_subagent_eval.py --json
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
git diff --check
```
