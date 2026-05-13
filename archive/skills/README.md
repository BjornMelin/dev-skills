# Archived Skills

`archive/skills/` preserves retired skill source trees without making them active.

Runtime and tooling contract:

- `skills/` is active-only. Direct `skills/<name>/SKILL.md` folders are eligible for cataloging, packaging, and installation.
- `archive/skills/<name>/` is source history only. Do not copy or symlink this tree into installed skills unless intentionally restoring it.
- Each archived skill must include `archive.json`.
- `codex-dev skills audit` validates archive manifests and flags active duplicates, missing replacements, and active-catalog references.

Required `archive.json` fields:

- `schema`: `skill_archive.v1`
- `name`: archived skill name matching the directory and retained `SKILL.md` frontmatter
- `status`: `archived`
- `archived_at`: RFC3339 timestamp
- `source_path`: original active path, `skills/<name>`
- `archived_path`: retained path, `archive/skills/<name>`
- `reason`: why the skill was archived
- `restore`: when and how it may be restored

`replacement` is optional, but when present it must name an active skill.
