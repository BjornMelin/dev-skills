# Repo Bootstrap Packs

Bootstrap packs provide a dry-run-first way to seed a new repository with
Codex operating guidance, validation notes, and subagent install docs without
copying private workstation state.

## Packs

List tracked packs:

```bash
python3 tools/bootstrap/render_bootstrap_pack.py --list
```

Validate manifests and referenced templates:

```bash
cargo run -q -p codex-dev -- --json bootstrap status
python3 tools/bootstrap/render_bootstrap_pack.py --validate
```

Current packs:

- `codex-agent-repo`: generic Codex repo guidance, docs, `.gitignore`, and
  project-agent README.
- `rust-cli-agent-repo`: generic guidance plus Rust CLI/service/TUI validation
  and review workflow notes.

Pack `advisory_host_checks` are emitted as metadata for real target
repositories. They are not run by the renderer because temp renders are often
not full project checkouts.

## Render Into A Temp Directory

Always preview or render into a temp directory before copying files into a real
project:

```bash
tmp=$(mktemp -d)
cargo run -q -p codex-dev -- --json bootstrap plan \
  --pack codex-agent-repo \
  --out "$tmp/repo" \
  --repo-name example-service \
  --primary-language rust
python3 tools/bootstrap/render_bootstrap_pack.py \
  --pack codex-agent-repo \
  --out "$tmp/repo" \
  --repo-name example-service \
  --primary-language rust \
  --generated-at 2026-05-09T06:00:00Z \
  --dry-run
python3 tools/bootstrap/render_bootstrap_pack.py \
  --pack codex-agent-repo \
  --out "$tmp/repo" \
  --repo-name example-service \
  --primary-language rust \
  --generated-at 2026-05-09T06:00:00Z
find "$tmp/repo" -maxdepth 3 -type f | sort
```

Use `--force` only when intentionally replacing files in a disposable render
directory or after reviewing the existing target contents.

## Codex Subagent Release Manifest

The Codex subagent release boundary is tracked in
`subagents/codex/RELEASE_MANIFEST.json`.

The public surface is:

- global roles under `subagents/codex/agents/global`;
- public overlays under `subagents/codex/agents/overlays/docmind`
  and `subagents/codex/agents/overlays/tooling`;
- renderer and sync scripts;
- example manifests for local overlays and local roles.

The private local surface is ignored:

- `overlays.local.json` and `overlays.local.*.json`;
- `roles.local.json` and `roles.local.*.json`;
- overlay directories under `subagents/codex/agents/overlays/*/`
  except the explicit public allowlist.

## Smoke Matrix

Run these checks after changing bootstrap packs, Codex subagents, or install
docs:

```bash
cargo run -q -p codex-dev -- --json bootstrap status
tmp_plan=$(mktemp -d)
cargo run -q -p codex-dev -- --json bootstrap plan --pack codex-agent-repo --out "$tmp_plan/codex" --repo-name codex-smoke --primary-language rust
python3 tools/bootstrap/render_bootstrap_pack.py --validate
tmp=$(mktemp -d)
python3 tools/bootstrap/render_bootstrap_pack.py --pack codex-agent-repo --out "$tmp/codex" --repo-name codex-smoke --generated-at 2026-05-09T06:00:00Z
python3 tools/bootstrap/render_bootstrap_pack.py --pack rust-cli-agent-repo --out "$tmp/rust" --repo-name rust-smoke --primary-language rust --generated-at 2026-05-09T06:00:00Z
PYTHONDONTWRITEBYTECODE=1 python3 subagents/codex/scripts/sync_agents.py --validate-release-manifest
PYTHONDONTWRITEBYTECODE=1 python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/codex/agents
PYTHONDONTWRITEBYTECODE=1 python3 subagents/codex/scripts/sync_agents.py --global --all-overlays --dry-run
PYTHONDONTWRITEBYTECODE=1 python3 subagents/codex/scripts/sync_agents.py --global --all-overlays --validate-sources
for path in \
  subagents/codex/overlays.local.json \
  subagents/codex/roles.local.json \
  subagents/codex/agents/overlays/private-repo/private_repo_reviewer.toml; do
  git check-ignore -v -- "$path" >/dev/null || { echo "not ignored: $path" >&2; exit 1; }
done
git diff --check
```

## Rollback

`sync_agents.py` writes timestamped backups before overwriting installed roles:

- global roles: `~/.codex/agent-backups/global-<timestamp>`;
- project overlays: `<project>/.codex/agent-backups/<overlay>-<timestamp>`.

Restore the relevant TOML files from the newest backup directory and rerun the
validator before continuing.
