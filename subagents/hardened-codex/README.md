# Hardened Codex Subagents

This directory is the tracked source pack for Bjorn's hardened Codex custom
subagents.

It intentionally separates:

- `agents/global`: global personal roles installed into `~/.codex/agents`;
- `agents/overlays/<repo>`: project roles installed into trusted checkouts'
  `.codex/agents`;
- `scripts/render_agents.py`: source-of-truth role spec renderer;
- `scripts/sync_agents.py`: timestamp-backup installer for global and project
  targets;
- `ROLE_CATALOG.md`: generated routing matrix and workflow recipes.
- `RELEASE_MANIFEST.json`: public/private boundary, dry-run/apply commands,
  rollback notes, and install smoke matrix.

The runtime policy is:

- all roles use `gpt-5.5`;
- simple inventory roles use `low`;
- most expert lanes use `high`;
- high-risk ambiguous synthesis roles use `xhigh`;
- no nested subagents by default;
- read-only unless the role explicitly runs tests, browser checks, smoke tests,
  or scoped implementation.

## Regenerate

```bash
python3 subagents/hardened-codex/scripts/render_agents.py
python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/hardened-codex/agents
```

## Install

Dry run:

```bash
python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --dry-run
```

Install a local-only private overlay by keeping its TOML files in an ignored
`agents/overlays/<repo>/` directory and passing the target checkout explicitly:

```bash
python3 subagents/hardened-codex/scripts/sync_agents.py \
  --overlay <repo> \
  --project-dir /path/to/private/repo \
  --dry-run
```

For repeated private installs, copy `overlays.local.example.json` to the ignored
`overlays.local.json`, set local overlay names and project paths there, then use:

```bash
python3 subagents/hardened-codex/scripts/sync_agents.py --list
python3 subagents/hardened-codex/scripts/sync_agents.py --all-local-overlays --validate-sources
python3 subagents/hardened-codex/scripts/sync_agents.py --all-local-overlays --dry-run
```

For maintainable private role definitions, keep the source specs in ignored
`roles.local.json` using `roles.local.example.json` as the shape. The renderer
loads that file when present, regenerates the matching ignored overlay TOMLs, and
keeps `ROLE_CATALOG.md` limited to the public catalog.

Apply with timestamp backups:

```bash
python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays
```

## Smoke

Use the release manifest and validation runbook as the source of truth. Minimum
tracked-pack smoke:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --validate-release-manifest
PYTHONDONTWRITEBYTECODE=1 python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/hardened-codex/agents
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --dry-run
git check-ignore -v subagents/hardened-codex/overlays.local.json subagents/hardened-codex/roles.local.json subagents/hardened-codex/agents/overlays/private-repo/private_repo_reviewer.toml
```

After installation, add representative Codex live spawns for the roles most
likely to be used in the target workflow.
