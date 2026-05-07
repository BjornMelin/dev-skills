# Maintenance Runbook

Use this when changing skills, custom-agent templates, docs, or the research
CLI.

## Update a Skill

1. Edit `skills/<skill-name>/SKILL.md`.
2. Keep `SKILL.md` concise.
3. Put long supporting material in `references/`.
4. Keep helper scripts deterministic and secret-free.
5. Validate:

   ```bash
   python3 tools/skill/quick_validate.py skills/<skill-name>
   ```

6. Package if the skill is published:

   ```bash
   python3 tools/skill/package_skill.py skills/<skill-name> skills/dist
   ```

## Update Deep Research Agents

1. Edit templates under:

   ```text
   skills/deep-researcher/templates/agents/
   ```

2. Validate:

   ```bash
   python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents
   ```

3. Dry-run install:

   ```bash
   python3 skills/deep-researcher/scripts/install_agents.py --target global --dry-run
   ```

4. Install only when intentional:

   ```bash
   python3 skills/deep-researcher/scripts/install_agents.py --target global --overwrite
   ```

Use `--overwrite` carefully. Global roles may be hand-edited.

## Update Subagent Creator Packs

1. Add or edit templates under `skills/subagent-creator/templates/agents/`.
2. Update pack membership in `scripts/subagent_creator.py`.
3. Validate templates.
4. Run `list --packs`.
5. Run `status` to inspect both global and project installs, then run
   `plan-sync` against the intended target before writing.
6. Smoke test relevant packs.

Commands:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py list --packs
python3 skills/subagent-creator/scripts/subagent_creator.py status --pack docs --project-dir . --include-extra
python3 skills/subagent-creator/scripts/subagent_creator.py plan-sync --pack docs --target project --project-dir . --include-extra
python3 skills/subagent-creator/scripts/subagent_creator.py smoke --pack docs
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/subagent-creator/templates/agents
```

Use `prune --confirm` only after reviewing the status or plan output. The
command backs up deleted TOML files by default.

## Update codex-research

1. Edit `crates/codex-research`.
2. Update [CLI Reference](../reference/codex-research-cli.md) when command
   behavior changes.
3. Update [Crate Reference](../reference/codex-research-crate.md) when data
   structures, provider behavior, cache schema, or extension points change.
4. Run Rust validation.
5. Run CLI smoke commands.

## Update Docs

When adding a new major doc:

- link it from [docs/index.md](../index.md);
- add README links if it is a top-level user entrypoint;
- update AGENTS.md if the doc changes repo workflow expectations;
- keep examples aligned with actual CLI help.

## Mirror Installed Skills

When the tracked skill copy should replace the installed copy:

```bash
rsync -a --delete skills/deep-researcher/ /home/bjorn/.agents/skills/deep-researcher/
rsync -a --delete skills/subagent-creator/ /home/bjorn/.agents/skills/subagent-creator/
rsync -a --delete skills/subspawn/ /home/bjorn/.agents/skills/subspawn/
```

Then validate installed copies:

```bash
python3 tools/skill/quick_validate.py /home/bjorn/.agents/skills/deep-researcher
python3 tools/skill/quick_validate.py /home/bjorn/.agents/skills/subagent-creator
python3 tools/skill/quick_validate.py /home/bjorn/.agents/skills/subspawn
```

## Version-Control Policy

Track:

- handwritten docs;
- skills;
- templates;
- scripts;
- root Cargo files;
- `Cargo.lock`.

Do not track:

- `target/`;
- `skills/dist/`;
- `.codex/research/` run artifacts unless explicitly requested;
- credentials, tokens, provider response dumps with private data.
