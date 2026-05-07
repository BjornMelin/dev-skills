# dev-skills Documentation

This directory documents the research, subagent, and skill-authoring systems
added to this repository:

- `codex-research`: Rust CLI for evidence-first research helpers.
- `deep-researcher`: skill and Focused Six subagent pack for deep cited
  research.
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
- [CLI Reference](reference/codex-research-cli.md): full command guide for
  `codex-research`.
- [Crate Reference](reference/codex-research-crate.md): Rust crate structure,
  data model, provider behavior, and extension points.
- [Validation](runbooks/validation.md): local gates including the manifest-backed
  research eval suite.
- [codex-research v0.2 Spec](specs/codex-research-v0.2.md): implemented
  follow-up plan covering config, budgets, GitHub hydration, source cache,
  privacy, evals, and research-agent contracts.

## Skill References

- [Deep Researcher Skill](reference/deep-researcher-skill.md)
- [Subagent Creator Skill](reference/subagent-creator.md)
- [Subspawn Skill](reference/subspawn.md)
- [Subagent Templates](reference/subagent-templates.md)

## Cookbooks

- [Deep Research Workflow](cookbooks/deep-research-workflow.md)
- [GitHub Archaeology](cookbooks/github-archaeology.md)
- [Context7 and Source Validation](cookbooks/context7-source-validation.md)
- [Subagent Fanout](cookbooks/subagent-fanout.md)
- [Evidence Ledgers and Reports](cookbooks/evidence-ledgers.md)

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

Search results are leads. Evidence is hydrated source material tied to claims,
freshness, confidence, and a ledger.
