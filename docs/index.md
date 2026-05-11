# dev-skills Documentation

This directory documents the research, subagent, operating-layer, and
skill-authoring systems added to this repository:

- `codex-research`: Rust CLI for evidence-first research helpers.
- `codex-dev`: current CLI for local task capsule lifecycle, structured
  evidence, subspawn plan/outcome/synthesis capture, repo-native policy gates,
  profile-aware validation manifests, PR evidence capture, and apply-gated PR
  agent actions.
- `codex-dev-tui`: optional Ratatui workbench that consumes `codex-dev`
  capsule JSON contracts without owning policy logic.
- `deep-researcher`: skill and Focused Six subagent pack for deep cited
  research.
- Rust skill suite: layered Rust skills for core Rust, CLI/Clap, Ratatui TUI,
  Tauri v2 apps, Axum/Tokio services, and explicit broad architecture planning.
- `subagent-creator`: skill and Python helper for authoring, installing,
  validating, diffing, syncing, backing up, and smoke-testing Codex custom
  agents.
- `subspawn`: hardened subagent delegation policy with planner-generated
  prompts, strict wait, and evidence-first synthesis.

The docs are handwritten and tracked. Generated Rust docs and build output stay
out of version control.

## Start Here

- [Onboarding](guides/onboarding.md): install, inspect, validate, and run the
  first research workflow.
- [System Overview](architecture/overview.md): how skills, subagents, and the
  CLI work together.
- [Research Architecture](architecture/research-system.md): provider routing,
  evidence ledgers, cache policy, and Firecrawl/GitHub/Context7 lanes.
- [codex-dev Operating Layer](specs/codex-dev-operating-layer.md): task capsule
  schema, ownership map, branch graph, and validation expectations for the
  development control plane.
- [codex-dev PR-Agent Safety Model](specs/codex-dev-pr-agent-safety-model.md):
  token, trust-boundary, dry-run, `--apply`, idempotency, and review-comment
  verification policy for hosted PR automation.
- [dev-skills v0.3/v1 Roadmap](specs/dev-skills-v0.3-roadmap.md): canonical
  issue ledger and execution order for the next local CLI/TUI, PR-agent,
  release, and future-surface wave.
- [codex-dev CLI Reference](reference/codex-dev-cli.md): command guide for
  local task capsule lifecycle, evidence appenders, subspawn
  plan/outcome/synthesis capture, policy gates, PR evidence capture, and
  apply-gated PR actions and closeout readiness reports.
- [codex-dev Core Reference](reference/codex-dev-core.md): shared Rust
  contract/read-model crate for capsule files, schema validation, and PR/policy
  evidence shapes.
- [codex-dev TUI Reference](reference/codex-dev-tui.md): optional Ratatui
  workbench for local capsule scanning.
- [codex-research CLI Reference](reference/codex-research-cli.md): full command guide for
  `codex-research`.
- [Crate Reference](reference/codex-research-crate.md): Rust crate structure,
  data model, provider behavior, and extension points.
- [Validation](runbooks/validation.md): local gates including research evals,
  bootstrap pack rendering, and hardened subagent smoke checks.
- [codex-research v0.2 Spec](specs/codex-research-v0.2.md): implemented
  follow-up plan covering config, budgets, GitHub hydration, source cache,
  privacy, evals, and research-agent contracts.

## Skill References

- [Deep Researcher Skill](reference/deep-researcher-skill.md)
- [Rust Skill Suite](reference/rust-skill-suite.md)
- [Subagent Creator Skill](reference/subagent-creator.md)
- [Subspawn Skill](reference/subspawn.md)
- [Subagent Templates](reference/subagent-templates.md)
- [Skill and Subagent Eval Lab](reference/skill-subagent-eval-lab.md)

## Cookbooks

- [Deep Research Workflow](cookbooks/deep-research-workflow.md)
- [GitHub Archaeology](cookbooks/github-archaeology.md)
- [Context7 and Source Validation](cookbooks/context7-source-validation.md)
- [Subagent Fanout](cookbooks/subagent-fanout.md)
- [Evidence Ledgers and Reports](cookbooks/evidence-ledgers.md)
- [Repo Bootstrap Packs](cookbooks/repo-bootstrap-packs.md)
- [Memory Guidance Proposals](cookbooks/memory-guidance-proposals.md)

## Prompt Library

- [Codex Scenario Prompts](prompts/codex-scenario-prompts.md): copy-paste
  prompts for using the new tools and skills in real Codex sessions.

## Runbooks

- [Validation](runbooks/validation.md): required checks for docs, skills, and
  Rust changes.
- [Troubleshooting](runbooks/troubleshooting.md): common failures and recovery
  steps.
- [Maintenance](runbooks/maintenance.md): how to update templates, rebuild
  bundles, install agents, and keep docs aligned.

## Core Rule

Search results and prior memory are leads. Evidence is hydrated source material
tied to claims, freshness, confidence, and a ledger.
