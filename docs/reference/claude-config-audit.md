# claude-config-audit Reference

`claude-config-audit` is a static auditor for a Claude Code configuration
estate. It reports drift that accumulates silently: skill symlinks that no
longer resolve, `skillOverrides` entries pointing at deleted skills, duplicate
agent names the loader discards without warning, oversized guides and skill
bodies, descriptions past the frontmatter cap, and skills whose text would
trigger a model's reasoning-extraction refusal.

Every rule corresponds to a failure mode observed in a real estate that no
existing tool detected. The tool performs no network calls and never modifies
files.

## Install

```bash
cargo install --path crates/claude-config-audit --locked --force
```

## Commands

```bash
# Audit a Claude home, optionally including a project's .claude directory
claude-config-audit scan --home ~/.claude --project /path/to/repo

# Compare the live install against its authoring source (see "Mirror drift")
claude-config-audit scan --mirror ~/repos/agents/dev-skills/skills

# Also check another agent's skill farm; repeatable
claude-config-audit scan --farm ~/.cursor/skills --farm ~/.factory/skills

# Machine-readable output
claude-config-audit scan --format json

# Ratchet the exit code (default: medium)
claude-config-audit scan --min-severity high

# Print the full rule catalog
claude-config-audit doctor

# Shell completions
claude-config-audit completions zsh
```

The full sweep, as run against this estate:

```bash
claude-config-audit scan \
  --home ~/.claude --project ~/repos/signr \
  --mirror ~/repos/agents/dev-skills/skills \
  --farm ~/.cursor/skills
```

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | No finding at or above `--min-severity` |
| `1` | Usage or IO error |
| `2` | At least one finding at or above `--min-severity` |

## Rules

| Rule | Severity | What it catches |
| --- | --- | --- |
| `links.broken-skill-symlink` | high | A symlink in a skills root does not resolve. The loader skips it in silence, so the skill simply vanishes. |
| `skill.oversized-skipped` | high | `SKILL.md` above 128KB. The loader skips the file entirely rather than truncating. |
| `skill.description-over-cap` | high | Description past the 1024-character frontmatter cap. Applies even when `disable-model-invocation` is set, since that flag removes a skill from the listing but does not exempt its frontmatter from validation. |
| `agent.duplicate-name` | high | Two agent files in the **same scope** declare the same frontmatter name; the loader keeps one and discards the rest. Project agents intentionally shadowing user agents are not reported. |
| `overrides.stale` | medium | A `skillOverrides` entry targets a skill that no longer exists, so it silently does nothing while implying intent. |
| `overrides.targets-disabled-plugin` | medium | An override names a skill supplied **only** by a plugin set to `false` in `enabledPlugins`. The skill exists on disk, so "stale" is the wrong diagnosis, but the override is equally inert. |
| `mirror.drift` | medium | A skill differs between `--mirror` (the authoring source) and the installed copy the loader actually reads. See below. |
| `farm.broken-symlink` | medium | A dangling link in another agent's farm passed via `--farm`. Same failure as `links.broken-skill-symlink`, in a tree this tool would otherwise never look at. |
| `farm.duplicate-target` | low | Two names in one farm resolve to the same skill, usually a pre-rename alias left beside the canonical name. The alias shadows the skill under a stale identity. |
| `skill.name-mismatch` | medium | Frontmatter `name` differs from the directory name. Identity comes from frontmatter. |
| `model.reasoning-extraction-risk` | medium | Body asks a model to expose its reasoning. Negated guidance ("do not show your reasoning") is not reported. |
| `skill.body-too-long` | low | `SKILL.md` body past 500 lines; move detail into `references/`. |
| `skill.description-listing-hog` | low | Description far past the ~280-character (~50 token) target. Paid on every request. |
| `agent.description-bloat` | low | Agent description past the cap; it sits in the system prompt on every request. |
| `skill.missing-frontmatter` | high | No parseable YAML frontmatter. The block is parsed as real YAML, so malformed keys elsewhere in the block are caught. |
| `guide.over-line-budget` | low | A `CLAUDE.md` past the 200-line guidance. The project walk is unbounded in depth (pruning `node_modules`, `.git`, `target`, `dist`, `build`, `.next`, `.turbo`, `worktrees`), so nested guides are not missed. |

## Mirror drift

A skill can live in two places at once: the authoring repo, where it is
reviewed and versioned, and the install root the loader actually reads. Nothing
keeps those in step. An edit committed and merged in the repo can sit there
indefinitely while every session keeps loading the old copy, and the reverse
happens just as easily when a skill is edited in place.

`--mirror` compares the two and reports each skill whose contents differ. The
comparison covers the **whole skill directory**, not just the entrypoint:
`references/`, `scripts/`, `assets/` and any other file change a skill's
behaviour as much as its `SKILL.md` does, and an entrypoint-only check missed
a real 51-line reference drift in the estate this tool audits. Expect a finding
whose message names a reference or script even when the two `SKILL.md` files
are identical.

Generated output is ignored: `__pycache__`, `node_modules`, `.DS_Store`,
`.pytest_cache` and `.ruff_cache`. Symlinks are followed, so a skill whose
`references/` is a shared symlinked directory is not reported as missing files.

It deliberately says nothing about direction: which side is right is a judgment
about what that skill should say, not something a byte comparison can decide.

A skill authored in the mirror but absent from the install is **not** reported.
Installing is a curation decision, and the same shared library is curated
differently for each agent that consumes it.

## Notes

- Plugin marketplaces under `<home>/plugins/marketplaces` are walked for skill
  **names only**, so overrides targeting a plugin-provided skill are not
  misreported as stale. They are not size-audited: they are upstream.
- Character thresholds count Unicode code points, matching
  `tools/skill/quick_validate.py`, not UTF-8 bytes.
- Findings are review leads. A description slightly past target on a skill that
  genuinely needs disambiguation from a sibling is a deliberate trade, not a
  defect.

## Related

- [`gsap-audit`](gsap-audit.md), [`expo-motion-audit`](expo-motion-audit.md),
  [`motion-token-audit`](motion-token-audit.md) share the same output shape
  (`--format markdown|json`, a `doctor` catalog, severity-gated exit codes).
