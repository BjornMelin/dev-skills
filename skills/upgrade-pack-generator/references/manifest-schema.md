# Manifest Schema

The canonical rendered source is `upgrade-pack.yaml`.

## Required Keys

- `schema_version`
- `family_display_name`
- `family_type`
- `mode`
- `family_slug`
- `plan_basename`
- `playbook_title`
- `operator_title`
- `trigger_title`
- `anchor_package`
- `related_packages`
- `repo_context`
- `target_surface`
- `qualification_plan`
- `research_plan`
- `current_version`
- `validated_upstream_version`
- `validated_doc_date`
- `repo_probes`
- `upstream_validation`
- `purpose`
- `use_when`
- `primary_goal`
- `non_goals`
- `primary_persona`
- `secondary_audience`
- `operating_goals`
- `source_hierarchy`
- `default_final_decisions`
- `intake_checklist`
- `required_research`
- `questions_to_resolve`
- `canonical_end_state`
- `what_to_adopt`
- `what_to_avoid`
- `framework_constraints`
- `supported_features`
- `unsupported_features`
- `codemod_recommendations`
- `execution_plan`
- `verification_commands`
- `report_heading`
- `report_requirements`
- `deliverables`
- `skill_routing_playbook`
- `operator_defaults`
- `operator_fast_intake`
- `operator_research`
- `operator_execute`
- `operator_exit_criteria`
- `skill_routing_operator`
- `trigger_mission`
- `trigger_goals`
- `trigger_required_research`
- `trigger_required_decisions`
- `trigger_required_outcomes`
- `trigger_required_deliverables`
- `trigger_verification_expectation`

## Notes

- `repo_context` is generated from live repo inspection.
- `target_surface` is the family-specific owner surface chosen during
  enrichment. It should explicitly identify whether the pack is root-owned,
  workspace-owned, or root-plus-workspaces, plus the verification strategy used
  for that shape.
- `qualification_plan` drives the separate qualification stage. It should
  declare the snapshot filename, current doc URLs, pinned source specs, and
  family-native read-only CLI checks.
- `research_plan` drives the separate research stage. It should declare the
  snapshot filename, raw bundle filename, web-findings filename,
  source-map policy, identity-confidence threshold, required evidence
  categories, required web-confirmation categories, target-version policy,
  release-range reasoning, source priorities, upstream URL buckets, pinned
  source specs, and repo-usage mapping commands.
- `family_type` should distinguish broad families such as `package` vs
  `framework`.
- `mode` should express the mission shape such as `upgrade`, `optimize`, or
  `upgrade+optimize`.
- `current_version`, `validated_upstream_version`, and `validated_doc_date`
  should be recorded after enrichment.
- `repo_probes` and `upstream_validation` are dictionaries whose values are
  ordered string lists.
- `required_research` and `execution_plan` are dictionaries whose values are
  ordered string lists.
- `verification_commands` is an ordered list of shell lines rendered into one
  fenced block.
- `plan_basename` controls file names:
  - `<plan_basename>-playbook.md`
  - `<plan_basename>-trigger-prompt.md`
  - `<plan_basename>-operator-mode.md`
- `research_plan.snapshot_filename` controls the machine-readable research file
  name, usually `research-snapshot.json`.
- `research_plan.bundle_filename` controls the raw machine-readable research
  bundle file name, usually `research-bundle.json`.
- `research_plan.web_findings_filename` controls the machine-readable `web.run`
  confirmation file name, usually `web-research-findings.json`.
- `research_plan.source_map_policy` describes how the bundled source map should
  be used, usually `bundled-seed-then-verify`.
- `research_plan.identity_confidence_threshold` controls the minimum confidence
  required for the generic package-identity resolver to count the pack as
  fully researched.
- `research_plan.required_web_confirmation_categories` declares which
  categories must have explicit `web.run` confirmations before the research
  stage can be `complete`.
- `qualification_plan.snapshot_filename` controls the machine-readable
  qualification file name, usually `qualification-snapshot.json`.

## Minimal Example

```yaml
schema_version: 3
family_display_name: Lucide React
family_type: package
mode: upgrade
family_slug: lucide-react
plan_basename: lucide-react-v1-upgrade
playbook_title: Lucide React v1 Upgrade Playbook
operator_title: Lucide React v1 Upgrade Operator Mode
trigger_title: Lucide React v1 Upgrade Trigger Prompt
anchor_package: lucide-react
related_packages:
  - lucide-react
repo_context:
  repo_root: /path/to/repo
  package_manager: pnpm
  detected_by: packageManager
target_surface:
  surface_type: workspace
  workspace_path: apps/web
  workspace_name: '@repo/web'
  workspace_package_json: apps/web/package.json
  workspace_slug: apps-web
  owner_reason: apps/web declares the framework package and owns its config.
  related_workspaces:
    - apps/web
  verification_strategy: layered-root-and-workspace
qualification_plan:
  strategy: separate-read-only-qualification
  snapshot_filename: qualification-snapshot.json
  doc_urls:
    upgrade guide: https://nextjs.org/docs/app/guides/upgrading/version-16
  source_specs:
    - next@16.2.4
  cli_checks:
    - label: Next.js codemod help
      cwd: apps/web
      command: pnpm dlx @next/codemod@canary --help
research_plan:
  strategy: separate-read-only-research
  snapshot_filename: research-snapshot.json
  bundle_filename: research-bundle.json
  web_findings_filename: web-research-findings.json
  required_categories:
    - official_docs
    - api_reference
    - migration_guides
    - release_history
    - examples_cookbooks
    - source_evidence
    - repo_usage_mapping
  source_priority:
    - official docs and API references first
    - official migration guides and upgrade walkthroughs second
    - official blog, release notes, and changelog sources third
    - upstream source inspection fourth
    - examples and cookbooks fifth
    - repo-local usage mapping always required
  identity_confidence_threshold: 0.75
  source_map_policy: bundled-seed-then-verify
  required_web_confirmation_categories:
    - official_docs
    - api_reference
  target_version_policy: latest-compatible-stable
  target_version: Next.js 16
  compatibility_rationale: >-
    Stay on the latest compatible stable Next.js 16 surface that fits the
    repo's runtime and deployment constraints.
  release_range: 16.2.4 -> Next.js 16
  official_docs:
    docs home: https://nextjs.org/docs
  api_reference:
    typedRoutes: https://nextjs.org/docs/app/api-reference/config/next-config-js/typedRoutes
  migration_guides:
    upgrade guide: https://nextjs.org/docs/app/guides/upgrading/version-16
  release_history:
    github releases: https://github.com/vercel/next.js/releases
  examples_cookbooks:
    static exports: https://nextjs.org/docs/app/guides/static-exports
  source_specs:
    - next@16.2.4
  repo_usage_queries:
    - label: Next config and route surfaces
      cwd: .
      command: rg --files apps/web | rg '(next\\.config\\.|(^|/)proxy\\.|(^|/)app/)'
current_version: 0.577.0
validated_upstream_version: v1
validated_doc_date: 2026-03-31
repo_probes:
  Repo posture:
    - record family-specific probe results here
upstream_validation:
  Official guidance:
    - record the live official guidance snapshot here
purpose: >-
  Use this playbook to fully explore, research, plan, implement, verify, and
  document a `lucide-react` upgrade in this repository.
use_when:
  - the repo already uses `lucide-react`
primary_goal:
  - standardize on named imports from `lucide-react`
non_goals:
  - unrelated design-system rewrites
framework_constraints:
  - record any family-specific constraints here
supported_features:
  - record supported features here
unsupported_features:
  - record unsupported or intentionally out-of-scope features here
codemod_recommendations:
  - record codemods or automated migration helpers here
```
