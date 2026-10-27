# Vitest engineering skill: authority and validation

The active skill is [vitest-dev](../../skills/vitest-dev/SKILL.md). The previous source is retained under [vitest-dev-v4](../../archive/skills/vitest-dev-v4/SKILL.md), with a `skill_archive.v1` manifest. The archive preserves the original files and subtrees; only the archived SKILL.md frontmatter name changes to match its directory. Historical sidecar metadata is retained as provenance, not an active installation.

## Design decision

This is an execution skill, not a documentation mirror. Its entry point is 558 whitespace-delimited words including frontmatter. Five independent references are loaded by task: configuration, performance, test patterns, browser testing, and CI. None links to another local reference. Routine test edits need not load any of them. The package contains no research inventory, sample application, bespoke test wrapper, custom CLI, or evaluation harness. The existing MIT license is retained.

Research belongs here rather than in the installed skill. The entry point narrows the work, preserves local conventions, and defines an honest stopping condition. References carry version-specific pitfalls and decision criteria, not mandatory configuration presets. No empirical claim is made that this text is the globally best skill, guarantees faster tests, or reduces a particular model's bill by a measured percentage.

## Verified release authority

Reviewed on 2026-10-27. The GitHub latest-release endpoint and npm package page identify **5.0.5**, published 2026-10-15, as stable. **5.1.0-beta.1**, published 2026-10-19, is a prerelease and is not silently adopted.

The important correction from earlier v5 guidance is that `fsModuleCache` defaults on from **5.0.4**, not from 5.0.0. The patch series also changes browser worker accounting, cache invalidation, hook cleanup, cancellation, and reporting. The **5.0.5 security fix** matters when UI, Browser Mode, or an API server is enabled; plain non-server runs are outside that advisory's affected configuration.

Primary release and source evidence:

- [5.0.0 announcement](https://vitest.dev/blog/vitest-5) and [migration guide](https://vitest.dev/guide/migration).
- [5.0.3 release](https://github.com/vitest-dev/vitest/releases/tag/v5.0.3), [5.0.4 release](https://github.com/vitest-dev/vitest/releases/tag/v5.0.4), [5.0.5 release](https://github.com/vitest-dev/vitest/releases/tag/v5.0.5), and [5.1.0-beta.1](https://github.com/vitest-dev/vitest/releases/tag/v5.1.0-beta.1).
- [5.0.5 package exports and engine/peer ranges](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/package.json).
- [5.0.5 defaults](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/defaults.ts), [worker calculations](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/utils/workers.ts), and [config resolution](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/config/resolveConfig.ts).
- [Tagged performance guide](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/guide/improving-performance.md), [filesystem cache](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/config/fsmodulecache.md), [shared servers](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/config/sharedviteserver.md), and [experimental options](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/config/experimental.md).
- [GHSA-p458-7748-mcp3](https://github.com/vitest-dev/vitest/security/advisories/GHSA-p458-7748-mcp3).

Context7 was resolved and queried against its indexed **5.0.3** source. Its opt-in cache guidance was superseded by the tagged 5.0.4/5.0.5 evidence. This is why the skill gives installed types and matching releases precedence over retrieved snippets.

## Research coverage and reuse

The official documentation index was surveyed. Substantive review focused on migration, configuration, experimental execution, project inheritance, CLI/doctor, diagnostics, transform caching, worker calculations, coverage, type tests, benchmarks, browser rendering/locators/assertions/traces/visual regression, mocking, resources, async assertions, and test-authoring guidance. This is not a claim that every page in the index or every implementation file was read line by line.

Useful official pages include [CLI](https://vitest.dev/guide/cli), [profiling](https://vitest.dev/guide/profiling-test-performance), [coverage](https://vitest.dev/guide/coverage), [type tests](https://vitest.dev/guide/testing-types), [benchmarks](https://vitest.dev/guide/benchmarking), [Vi API](https://vitest.dev/api/vi), [resource cleanup](https://vitest.dev/guide/recipes/explicit-resources), [type narrowing](https://vitest.dev/guide/recipes/type-narrowing), [schema matching](https://vitest.dev/guide/recipes/schema-matching), [Browser Mode](https://vitest.dev/guide/browser), and [AI test authoring](https://vitest.dev/guide/learn/writing-tests-with-ai).

Existing agent material was inspected rather than treated as interchangeable authority:

| Source | What was reused | What was not imported |
| --- | --- | --- |
| [antfu/skills Vitest](https://github.com/antfu/skills/tree/main/skills/vitest), also prominent on [skills.sh](https://skills.sh/antfu/skills/vitest) | Public API terminology, selective reference organization, concurrency/sharding distinctions. | Its entire reference library or 5.1-beta assumptions. The inspected entry identified 5.1.0-beta.1. |
| [onmax/nuxt-skills Vitest](https://github.com/onmax/nuxt-skills/tree/main/skills/vitest) | Comparison against a popular generated variant. | Duplicate evidence weighting: it derives from the same authoring lineage and identified 5.0.1. |
| [Vitest's contributor agent](https://github.com/vitest-dev/vitest/blob/v5.0.5/.claude/agents/vitest-test-writer.md) | Verify actual test outcomes/counts and follow the repository's established harness. | Vitest-internal helpers, repository layout, or universal snapshot/assertion restrictions. This is a contributor agent, not a consumer skill package. |
| [OpenAI skill creator](https://github.com/openai/skills/blob/main/skills/.system/skill-creator/SKILL.md) and [Agent Skills guidance](https://agentskills.io/skill-creation/best-practices) | Concise triggers, task-gated progressive disclosure, shallow reference structure, useful agent metadata. | An authoring framework, redundant documents, or mandatory context expansion. |

Additional skills.sh results were screened as discovery leads, not copied or certified by their installation counts. No dedicated consumer skill repository was found by the `vitest-dev` organization skill search; that search result is not proof that no official skill exists anywhere. The authored prose is an original synthesis, not a vendored copy of a third-party skill.

Native ecosystem guidance came from [Oxlint rules](https://oxc.rs/docs/guide/usage/linter/rules/vitest/valid-expect), the upstream [Vitest ESLint plugin](https://github.com/vitest-dev/eslint-plugin-vitest), [Turbo task graphs](https://turborepo.dev/docs/crafting-your-repository/configuring-tasks), and [Bun's runtime documentation](https://bun.com/docs/runtime). The result prefers existing runtime, lint, and task-graph capabilities instead of custom enforcement packages.

## Review and verification

The editorial review corrected or explicitly guarded these cases:

| Scenario | Required behavior in the final text |
| --- | --- |
| Ordinary assertion repair | Local scope; no mandatory doctor, upgrade, broad audit, or full documentation crawl. |
| 5.0.3 versus 5.0.5 cache setting | Different defaults acknowledged; no redundant universal preset. |
| Latest community reference mentions 5.1 flags | Verify installed release; do not apply prerelease-only APIs on stable. |
| Plugin reads external inputs | Fix transform-cache invalidation; do not claim a warm stale cache is faster correctness. |
| VM or isolation experiment | Measure and test state/order/realm implications; no universal fast pool. |
| Turbo sees unchanged app but changed shared source | Model dependency invalidation; verify hit and miss behavior. |
| Concurrent tests and globals | Context-local assertions/cleanup and no shared fake-clock/mock races. |
| Unawaited assertions | Await the work, do not suppress the failure. |
| Browser text migration | Distinguish exact matching from substring/regex assertions. |
| Browser Mode versus E2E/native | Preserve application and device boundaries. |
| Coverage improvement | Inspect source inclusion and branch semantics, not percentage inflation. |
| Lint autofix weakens a boolean assertion | Preserve exact semantics; reject style-only weakening. |
| Static listing or Turbo cache hit | Do not present it as fresh execution. |
| Published artifacts | Keep secrets out and preserve complete shard/report provenance. |

These are editorial acceptance scenarios, not executed model evaluations. No subagent panel or measured Astra-specific performance is claimed.

Local validation uses the repository's exact `tools/skill/quick_validate.py`, whose fetched blob hash was verified, for both active and archived frontmatter. Additional checks cover YAML/JSON, local links, reference fanout, text-size budgets, TypeScript syntax of the browser configuration, preserved legacy source hashes, ZIP integrity, and package/source byte parity. Dependency-backed Vitest execution and real browser/provider runs are not claimed: the local environment cannot resolve the package registry. Repository-wide Rust/catalog/archive and documentation gates are evaluated by the PR's native CI when available; consult their actual results rather than treating this prose as a passing status.

The ChatGPT plugin and standalone ZIP are built from the same canonical skill files. The plugin adds only its portable manifest, not another instruction layer or MCP dependency. Research and archive files are excluded from both installed skill payloads.
