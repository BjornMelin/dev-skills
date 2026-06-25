# Maintenance

Use this reference when updating the skill or checking CLI drift.

## Local Diagnostics

```bash
node scripts/firecrawl-doctor.mjs --json
node scripts/firecrawl-help-snapshot.mjs --output /tmp/firecrawl-help.json --markdown /tmp/firecrawl-help.md
node scripts/firecrawl-cache-index.mjs self-test
```

These scripts are non-interactive. They do not run Firecrawl setup or install
skills. `firecrawl-cache-index.mjs` reads local files only and never calls
Firecrawl.

`firecrawl-doctor.mjs` reports required command availability plus drift
warnings for CLI version mismatch, missing `x download`, missing monitor,
research, feedback, or doctor support, `.firecrawl` gitignore posture, and
accidentally reinstalled split CLI skills.

## Validation

From the `dev-skills` repository:

```bash
python3 tools/skill/quick_validate.py skills/firecrawl
node --check skills/firecrawl/scripts/firecrawl-doctor.mjs
node --check skills/firecrawl/scripts/firecrawl-help-snapshot.mjs
node --check skills/firecrawl/scripts/firecrawl-cache-index.mjs
node skills/firecrawl/scripts/firecrawl-cache-index.mjs self-test
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

Rebuild the local `.skill` ZIP when release artifacts matter:

```bash
python3 tools/skill/package_skill.py skills/firecrawl skills/dist
python3 -m zipfile -t skills/dist/firecrawl.skill
```

`skills/dist/` is ignored in this repo. `codex-dev skills inventory` only marks
`package.present=true` for bundles intentionally tracked by git, so a locally
rebuilt ignored bundle can still show `missing_dist_package`.

After installing to the global runtime:

```bash
python3 tools/skill/quick_validate.py "$HOME/.agents/skills/firecrawl"
node "$HOME/.agents/skills/skill-auditor/scripts/audit-skills-baseline.mjs" "$HOME/.agents/skills" /tmp/firecrawl-skill-audit firecrawl-post-merge
```

## Source Of Truth

Tracked source lives in `<your-dev-skills-clone>/skills/firecrawl`. From the
root of that clone, install to the global runtime with:

```bash
rsync -a --delete skills/firecrawl/ "$HOME/.agents/skills/firecrawl/"
```

Before deleting split skills, back them up. Remove only these split CLI skills:

```text
firecrawl-agent
firecrawl-crawl
firecrawl-download
firecrawl-interact
firecrawl-map
firecrawl-parse
firecrawl-scrape
firecrawl-search
firecrawl-monitor
firecrawl-cli
```

Do not remove Firecrawl build or workflow skills as part of this merge.
