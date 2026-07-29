# Archived Skills

`archive/skills/` preserves retired skill source trees without making them active.

Runtime and tooling contract:

- `skills/` is active-only. Direct `skills/<name>/SKILL.md` folders are eligible for cataloging, packaging, and installation.
- Flat `archive/skills/<name>/` leaves remain supported. Related leaves may instead live under the optional `archive/skills/{gsap,native,rust}/<name>/` group containers.
- Group containers are organization only and are never skills. Each leaf basename is the hyphen-case skill identity and must match its `archive.json` and retained `SKILL.md` frontmatter.
- Archived leaves are source history only. Do not copy or symlink them into installed skills unless intentionally restoring one.
- Each archived skill must include `archive.json`.
- `codex-dev skills audit` validates archive manifests and flags active duplicates, missing replacements, and active-catalog references.

Required `archive.json` fields:

- `schema`: `skill_archive.v1`
- `name`: archived skill name matching the directory and retained `SKILL.md` frontmatter
- `status`: `archived`
- `archived_at`: RFC3339 timestamp
- `source_path`: original active path, either `skills/<name>` or `plugins/<plugin>/skills/<name>`. Snapshot archives may use `~/.agents/skills/<name>` to record a previously installed global copy.
- `archived_path`: full retained leaf path, either `archive/skills/<name>` or `archive/skills/{gsap,native,rust}/<name>`
- `reason`: why the skill was archived
- `restore`: when and how it may be restored

`replacement` is optional, but when present it must name an active skill. Skill
names must be unique across all flat and grouped archive leaves.

`kind` is optional and defaults to `retirement`. Set `kind: "snapshot"` for a
historical copy of a previously installed global skill when the active
`skills/<name>/` (if any) remains canonical. Snapshots allow:

- `source_path` of `~/.agents/skills/<name>` to record the install that was archived.
- The archive leaf name to match an active skill entrypoint (the audit logs a warning instead of an error in that case).

Snapshot archives must still keep `archive.json`, the leaf `SKILL.md`, and the
retired source tree intact. They are not active and must not be copied into
installed skill farms unless intentionally restoring one.
