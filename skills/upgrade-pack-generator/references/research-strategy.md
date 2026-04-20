# Research Strategy

The research stage exists to prove that a pack actually reviewed the upstream
surfaces needed for a safe, modern upgrade.

## Goals

- prove that official docs and API references were reviewed
- prove that migration guides and release notes for the relevant range were
  reviewed
- prove that examples or cookbook-style adoption guidance were reviewed
- prove that pinned upstream source was inspected when needed
- prove that repo-local usage was mapped before implementation begins

## Source Priority

1. Official docs and API references
2. Official migration guides and upgrade walkthroughs
3. Official release notes, changelogs, and release posts
4. Upstream source inspection
5. Official examples and cookbook-style guides
6. Repo-local usage mapping

## Enforcement

- `research_plan` in `upgrade-pack.yaml` declares the required categories and
  exact URLs, source specs, and repo-usage commands.
- `scripts/research_upgrade_pack.py` produces `research-snapshot.json`.
- `scripts/qualify_upgrade_pack.py` must read that snapshot before it can mark a
  pack `ready`.
- If required categories are missing or failed, qualification must degrade to
  `ready-with-caveats` or `insufficient-evidence`.

## Family Guidance

- Next.js packs should include official docs, API refs, version upgrade guides,
  release posts or releases, and repo-specific route/config usage mapping.
- Expo/EAS packs should include SDK guides, EAS docs, changelog sources,
  package reference pages, and mobile-workspace config mapping.
- Convex packs should include CLI docs, generated API or client references,
  release sources, and repo-specific schema or codegen mapping.
- Turborepo packs should include root config guidance, query or ls references,
  release sources, and monorepo task-graph mapping.
- Generic packs should still define the full contract, but may remain
  `insufficient-evidence` until a human or later family override supplies the
  missing official sources.
