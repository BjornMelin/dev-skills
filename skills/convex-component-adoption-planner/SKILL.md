---
name: convex-component-adoption-planner
description: Researches a Convex component against Signr's live backend graph, asks scored one-at-a-time design questions, and produces a reusable adoption or rejection package with `PLAN.md` and `CODEX_FULL_PROMPT.md`.
---

# Convex Component Adoption Planner

Use this skill when Signr needs a serious answer to any of these:

- Should we adopt a specific Convex component?
- Which Convex component best fits a planned feature?
- How should a component integrate into Signr without creating duplicate ownership?
- Generate a fresh-session execution package for adopting or explicitly rejecting a component.

## Core stance

- Always inspect Signr's current durable owner before recommending a component.
- Prefer one canonical owner. If a component would create a second source of truth, recommend against it unless the plan is a full hard-cut replacement.
- Keep deterministic prep in the helper CLI. Keep architecture, ownership, and recommendation judgment in the model.
- Use live Convex docs first. If the user asks for the latest or current component state, fetch it.
- Ask remaining design questions one at a time with `request_user_input`, and include weighted scores from `0.0` to `10.0` for each option.

## Start here

1. If the component is not already fixed, fetch the live components index:

```bash
curl -s https://www.convex.dev/components/components.md
```

2. Run the helper first when available:

```bash
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- doctor --json
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- component @convex-dev/aggregate --feature bookmarks-intent-signals --json
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --json
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --stdout
```

### Helper command contract

Use the Rust helper in this order:

1. `doctor`
   - checks whether the useful local tools are available
   - reports posture for `curl`, `rg`, `opensrc`, `ctx7`, `gh`, `bunx`, and `cargo`
   - use `--json` when you want machine-readable output
2. `component <package>`
   - normalizes the package into a docs slug
   - emits the live Convex docs URLs
   - emits the default plan output paths
   - emits suggested `curl`, `opensrc`, `ctx7`, and `rg` commands
   - pass `--feature <slug>` whenever the workstream is already known
   - pass `--docs-slug <slug>` if the Convex docs page does not match the inferred slug
   - pass `--date YYYY-MM-DD` or `--plan-root <path>` when you need a non-default output location
3. `scaffold <package>`
   - uses the same normalization rules as `component`
   - creates the target plan folder and writes stub `PLAN.md` plus `CODEX_FULL_PROMPT.md`
   - use `--json` to get created and skipped file paths back as structured output
   - use `--stdout` to preview the exact file contents without writing
   - use `--force` only when you intentionally want to overwrite existing stubs

Do not treat the helper output as the recommendation. It only prepares deterministic inputs and starter files.

3. Read the helper output, then gather exact repo context:

- `packages/backend/package.json`
- `packages/backend/convex/convex.config.ts`
- `packages/backend/convex/schema.ts`
- feature-specific backend modules and specs
- if the feature is already planned, the matching files under `.agents/signr-implementation-prompts-modernization-package/docs/implementation-prompts/`

4. Fetch the live component markdown page and inspect the package source:

```bash
curl -s https://www.convex.dev/components/<slug>/<slug>.md
opensrc path <package-name>
```

5. Optionally use extra tooling when it improves evidence quality:

- `ctx7`: useful for library documentation lookup and secondary cross-checks
- `opensrc`: preferred when internal implementation details matter
- `bunx convex`: optional for local CLI help or version-aligned Convex context
- `gh`: only if a GitHub issue or PR is part of the adoption decision

Do not hard-depend on any of these tools. Degrade cleanly when absent.

## Workflow

1. Normalize the target component, feature scope, and intended output folder.
   - Prefer `doctor` then `component`.
   - Use `scaffold --stdout` when you want to inspect the starter package before writing it.
   - Use `scaffold` without `--stdout` when you are ready to create the package files.
2. Map the current durable owner in Signr before recommending anything.
3. Read only the exact repo files that govern the feature boundary.
4. Fetch live docs for the components index and the specific component page.
5. Inspect dependency source with `opensrc` when package behavior, tables, or APIs matter.
6. Check whether the modernization-package prompts already define the feature. If they do, include prompt-package alignment in the plan.
7. Ask one design question at a time with `request_user_input` only when local context cannot close the branch safely.
8. Produce the package described in `references/package-spec.md`.

## Decision framework

Use this weighted scoring model for major choices:

| Criterion | Weight |
| --- | --- |
| Solution leverage | 35% |
| Application value | 30% |
| Maintenance and cognitive load | 25% |
| Architectural adaptability | 10% |

Rules:

- Recommended options should usually land at `9.0+` only when the tradeoff is genuinely strong.
- If every option is weak, say so and score them honestly.
- If a component conflicts with an existing durable owner, that conflict is usually a decisive penalty.

## Required outputs

Read `references/package-spec.md` before writing final package files.

Default output location:

```text
.agents/plans/YYYY-MM/MM-DD/convex-components/<component-slug>/
```

Required files:

- `PLAN.md`
- `CODEX_FULL_PROMPT.md`

For a rejection decision, still emit the same package, but make `PLAN.md` a definitive non-adoption memo and make `CODEX_FULL_PROMPT.md` an execution prompt for future re-evaluation or confirmed rejection.

## Question style

When a user wants to work interactively or says `grill me`:

- Ask one question at a time.
- Provide exactly 2-3 mutually exclusive options.
- Put the recommended option first.
- Include a short weighted score summary in each option description.
- Resolve packaging, ownership, migration, and prompt-alignment branches before writing the package.

## What not to automate

- Final recommendation
- Duplicate-ownership analysis
- Hard-cut migration judgment
- Whether a planned feature should stay app-owned or move to a component
- Prompt-package alignment decisions

Those stay in the model.

The helper can scaffold stub `PLAN.md` and `CODEX_FULL_PROMPT.md` files, but it must not decide the recommendation or architecture.

## Examples

Evaluate one candidate:

```bash
curl -s https://www.convex.dev/components/components.md
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- component @convex-dev/aggregate --feature bookmarks-intent-signals --json
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --json
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --stdout
```

Overwrite an existing starter package intentionally:

```bash
cargo run --manifest-path .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep/Cargo.toml -- scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --force --json
```

Use optional Context7 support:

```bash
ctx7 library convex "components aggregate"
ctx7 docs /convex-dev/convex "aggregate component patterns"
```

Install the helper locally for repeated use:

```bash
make -C .agents/skills/convex-component-adoption-planner/scripts/convex-component-adoption-prep install-local
convex-component-adoption-prep doctor --json
convex-component-adoption-prep component @convex-dev/aggregate --feature bookmarks-intent-signals --json
convex-component-adoption-prep scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --json
convex-component-adoption-prep scaffold @convex-dev/aggregate --feature bookmarks-intent-signals --stdout
```
