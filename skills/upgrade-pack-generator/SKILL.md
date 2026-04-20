---
name: upgrade-pack-generator
description: Generate repo-local dependency upgrade packs under `.agents/plans/upgrade/topic-slug/` with a playbook, trigger prompt, operator mode, and canonical manifest. Use when Codex needs to research an anchor package, expand related dependencies, detect target repo context, and create a reusable upgrade folder before any implementation work starts.
---

# Upgrade Pack Generator

Generate upgrade-pack folders for dependency families in two stages:

1. Author or refine a reusable family override.
2. Instantiate a repo-specific pack from that family plus live repo context.

The generator is monorepo-aware. During enrichment it must lock an explicit
owner surface for the family:

- repo root
- one owning workspace
- repo root plus related workspaces

The generated pack is docs/research only. Do not implement package changes while
generating the pack.

## Core Workflow

1. Read [references/workflow.md](references/workflow.md).
2. Decide the mode:
   - family authoring
   - repo instantiation
3. Detect repo context first:

   ```bash
   python3 scripts/detect_repo_context.py --repo-root /path/to/repo --json
   ```

4. Bootstrap a manifest from the anchor package:

   ```bash
   python3 scripts/bootstrap_manifest.py \
     --repo-root /path/to/repo \
     --anchor-package lucide-react \
     --out /tmp/upgrade-pack.yaml
   ```

5. Enrich the manifest with live repo probes and official upstream guidance:

   ```bash
   python3 scripts/enrich_manifest.py --manifest /tmp/upgrade-pack.yaml
   ```

6. Refine the enriched manifest further with live research when needed:
   - `$repo-modernize-upgrade-audit`
   - `$opensrc`
   - `$opensrc-inspect`
   - `$technical-writing`
   - `$hard-cut`
7. Validate the manifest:

   ```bash
   python3 scripts/validate_upgrade_pack.py /tmp/upgrade-pack.yaml
   ```

8. Run read-only qualification:

   ```bash
   python3 scripts/qualify_upgrade_pack.py --manifest /tmp/upgrade-pack.yaml
   ```

9. Render the pack:

   ```bash
   python3 scripts/render_upgrade_pack.py \
     --manifest /tmp/upgrade-pack.yaml \
     --output-dir /path/to/repo/.agents/plans/upgrade/<topic>
   ```

## Output Contract

Every generated repo-local pack must contain exactly:

- `upgrade-pack.yaml`
- `qualification-snapshot.json`
- `<basename>-playbook.md`
- `<basename>-trigger-prompt.md`
- `<basename>-operator-mode.md`

The manifest is the canonical source. The markdown files are rendered outputs.
`qualification-snapshot.json` is the canonical machine-readable evidence file for
the separate qualify stage.

Rendered file roles:

- `<basename>-playbook.md`
  - authoritative human handoff doc and the only writable pack file during an
    implementation run
- `<basename>-operator-mode.md`
  - execution delta card that points back to the playbook
- `<basename>-trigger-prompt.md`
  - thin launcher for a fresh Codex session with a compact repo summary

Do not hand-maintain `operator-mode.md` or `trigger-prompt.md`; regenerate them
from the manifest.

## Family Overrides

Read [references/manifest-schema.md](references/manifest-schema.md) before
editing any override.

Family overrides live in `references/family-overrides/`. Use them when a
dependency family recurs across repos and benefits from package-specific:

- end-state decisions
- research lanes
- migration questions
- verification proofs
- file naming and slug conventions

Current built-in family lanes:

- `lucide-react`
- `nextjs`
- `expo-eas`
- `convex`
- `turborepo`
- `shadcn-radix-ui`

Keep overrides narrow. Put only family-specific information there. Let the base
generator provide the common structure.

## Package Manager And Framework Detection

Detect the target repo command family in this order:

1. `package.json#packageManager`
2. root lockfiles
3. repo docs and CI hints

The generator should write that decision into `upgrade-pack.yaml` and render
package-manager-aware command variables into the pack. Do not force Bun into a
pnpm/npm/Yarn repo.

## Research Routing

Use the default research lanes in
[references/research-lanes.md](references/research-lanes.md).

Conditional routing:

- use `$bun-dev` only when Bun posture is actually relevant
- use framework/plugin lanes only when the target repo detects them
- use browser lanes only when the package family affects visible UI
- do not route through `$imagegen` unless raster asset generation is truly part
  of the package-family surface

## Scripts

- `scripts/detect_repo_context.py`
  - inspects package manager, lockfiles, docs/CI hints, frameworks, and package
    manifests
- `scripts/bootstrap_manifest.py`
  - creates a starter `upgrade-pack.yaml` from generic defaults plus an optional
    family override
- `scripts/enrich_manifest.py`
  - enriches `upgrade-pack.yaml` with family-specific repo probes, current
    package versions, and live official-doc snapshots
- `scripts/validate_upgrade_pack.py`
  - validates the manifest contract before rendering
- `scripts/qualify_upgrade_pack.py`
  - runs family-native read-only qualification and writes
    `qualification-snapshot.json`
- `scripts/render_upgrade_pack.py`
  - renders the repo-local pack from the canonical manifest and qualification
    snapshot

## Examples

Bootstrap a known family override:

```bash
python3 scripts/bootstrap_manifest.py \
  --repo-root /path/to/repo \
  --anchor-package lucide-react \
  --out /tmp/lucide-upgrade-pack.yaml
```

Render a generic pack for a package without an override:

```bash
python3 scripts/bootstrap_manifest.py \
  --repo-root /path/to/repo \
  --anchor-package commander \
  --out /tmp/commander-upgrade-pack.yaml
python3 scripts/enrich_manifest.py \
  --manifest /tmp/commander-upgrade-pack.yaml
python3 scripts/qualify_upgrade_pack.py \
  --manifest /tmp/commander-upgrade-pack.yaml
python3 scripts/render_upgrade_pack.py \
  --manifest /tmp/commander-upgrade-pack.yaml \
  --output-dir /path/to/repo/.agents/plans/upgrade/commander
```

Refresh an existing repo-local pack after refining the manifest:

```bash
python3 scripts/enrich_manifest.py \
  --manifest /path/to/repo/.agents/plans/upgrade/lucide-react/upgrade-pack.yaml
python3 scripts/validate_upgrade_pack.py \
  /path/to/repo/.agents/plans/upgrade/lucide-react/upgrade-pack.yaml
python3 scripts/qualify_upgrade_pack.py \
  --manifest /path/to/repo/.agents/plans/upgrade/lucide-react/upgrade-pack.yaml
python3 scripts/render_upgrade_pack.py \
  --manifest /path/to/repo/.agents/plans/upgrade/lucide-react/upgrade-pack.yaml \
  --output-dir /path/to/repo/.agents/plans/upgrade/lucide-react
```
