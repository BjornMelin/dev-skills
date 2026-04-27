# Repo Modernize Upgrade Audit Execution Prompt

## Execution Prompt

```text
Upgrade, modernize, and simplify this repo or monorepo end-to-end.

Use these skills and tools throughout this task:
- `$bun-dev`
- `$hard-cut`
- `$clean-code`
- `$reducing-entropy`
- `$github`
- `context7`
- `web.run`

Conditional skill and tool routing:
- If Expo / React Native / Expo Router / EAS is present, also use:
  - `$expo:upgrading-expo`
  - official Expo docs and changelogs
  - Expo CLI, EAS CLI, and `expo-doctor`
- If Convex is present, also use:
  - `$convex-audit`
  - official Convex docs and latest guidance
- If Next.js is present, also use:
  - `$vercel:nextjs`
  - official Next.js upgrade guides, codemods, and migration docs
- If Turborepo is present, use:
  - `turbo` CLI
  - official Turborepo and Vercel docs
- If Vercel is present, use:
  - Vercel plugin/tools
  - Vercel CLI where appropriate
  - official Vercel docs, release notes, and platform guidance
- If a package upgrade is major, migration-heavy, API-ambiguous, or likely to
  delete custom repo code via newer native capabilities, also use:
  - `$opensrc`
  - `opensrc path`
  - `/home/bjorn/.codex/skill-support/bin/deps-workbench upgrade-prep`
- Only invoke framework-specific skills when the framework is actually detected.
- Do not reference or depend on skills that are not available in the current environment.

Mission:
- Find every vulnerable dependency, outdated dependency, deprecated API, obsolete custom abstraction, and dependency-driven cleanup opportunity in this repo or monorepo.
- Deeply research the latest official dependency documentation, changelogs, release notes, migration guides, API references, GitHub releases, advisories, and upstream issues/PRs where relevant.
- Upgrade dependencies to the latest safe and supportable versions.
- Refactor the codebase to adopt modern dependency-native capabilities and APIs.
- Remove legacy, compatibility, adapter, fallback, shim, polyfill, and duplicate code paths that are no longer justified after the upgrades.
- Leave the repo in a verified, production-ready, simplified state with one canonical implementation path per concern.

Operating mode:
- Act as an architect-level modernization and upgrade agent.
- Do not stop at planning. Execute end-to-end unless blocked by a concrete high-risk issue.
- Prefer primary-source evidence over assumptions.
- Prefer fewer, stronger implementations over preserving historical paths.
- Prefer less code after the change than before it.
- Treat the final codebase shape as the goal, not the smallest immediate diff.

Research and evidence requirements:
1. Before editing anything, inspect:
   - repo structure
   - workspace layout
   - manifests
   - lockfiles
   - package-manager setup
   - runtime versions
   - CI/build/test scripts
   - app/framework boundaries
   - architecture and upgrade docs
2. Determine whether this is:
   - Bun
   - npm
   - pnpm
   - yarn
   - mixed monorepo
3. Use the correct repo-native package manager for actual dependency changes. Use Bun CLI aggressively for auditing and dependency analysis where useful, but do not force Bun as the package manager if the repo is not Bun-managed.
4. Use `bun --help` and command-specific help before using Bun package-management and audit commands.
5. Use `web.run`, `context7`, and GitHub research to verify:
   - latest stable versions
   - latest safe versions
   - breaking changes
   - migration requirements
   - deprecations
   - security advisories
   - official upgrade paths
   - whether newer package versions replace custom repo code with native features
6. Prefer official docs, upstream repos, GitHub releases, advisories, and primary-source migration guides over secondary summaries.
7. When uncertain, research before editing. Label anything still uncertain as `UNVERIFIED`.

Exploration requirements:
1. Map the entire repo or monorepo first.
2. Spawn lightweight explorer subagents in parallel to cover:
   - manifests, lockfiles, and workspace graph
   - framework detection by package/app
   - direct and transitive dependency hotspots
   - deprecated API usage
   - package-level upgrade blast radius
   - custom helper/util code that may now be obsolete
   - duplicate ownership and duplicated implementations
   - compatibility branches, fallbacks, shims, coercions, adapters, and dual-shape code
   - dead code and dead tests
3. Use explorer findings to build a precise affected-files and affected-packages map before broad edits.

Dependency audit requirements:
1. Inventory all direct dependencies by workspace.
2. Inventory important transitive dependencies implicated by audits and outdated checks.
3. Run the strongest relevant stale and security checks available, including Bun where useful:
   - `bun audit`
   - `bun outdated`
   - `bun pm ls`
   - `bun pm why`
   - plus package-manager-native equivalents for the repo
4. For major or ambiguous npm/Bun package upgrades, run the fast prep layer before source-level analysis:
   - `deps-workbench upgrade-prep --cwd <repo-root> --package <pkg> --out <tmp.json>`
   - `deps-workbench report --input <tmp.json> --format md`
   - then use `$opensrc` for current-versus-target source comparison
5. Produce a dependency matrix covering:
   - package
   - current version
   - latest version
   - latest safe version
   - vulnerability status
   - maintenance status
   - likely migration complexity
   - affected code areas
   - whether to upgrade, replace, or remove
6. Treat abandoned or stagnant packages as replacement/removal candidates, not automatic keepers.

Framework-specific requirements:
- Expo:
  - detect current Expo SDK and related React/React Native versions
  - review Expo changelogs and SDK migration docs
  - run `expo-doctor`
  - use Expo-native upgrade flows and dependency alignment
  - check for deprecated Expo packages and replace them with current canonical packages
  - verify EAS/build/runtime implications
- Convex:
  - review latest Convex docs before implementation
  - align functions, validators, schema, and runtime usage with current best practices
  - remove outdated or non-canonical Convex patterns
  - prefer index-backed queries, explicit validators, and current official conventions
- Next.js:
  - detect current Next.js version and React alignment
  - use version-specific official migration guides and codemods
  - upgrade incrementally across major versions if required
  - migrate to current canonical APIs and config expectations
- Turborepo:
  - map tasks, pipelines, filters, cache boundaries, and package relationships
  - update scripts/config to current supported patterns if needed
- Vercel:
  - review Vercel release notes and platform/runtime docs
  - modernize Vercel-related config and remove obsolete workarounds where justified

Upgrade strategy:
1. Upgrade the repo to the newest supportable dependency set, prioritizing:
   - security fixes
   - maintained packages
   - supported APIs
   - reduced custom code
2. Do not just churn lockfiles. Make the codebase actually compatible with the upgraded dependencies.
3. When a dependency major version introduces a better canonical API, migrate fully if the repo can reasonably absorb it.
4. Replace custom implementations with dependency-native capabilities where newer package versions now provide them.
5. Remove packages that are no longer needed after refactors.
6. Collapse overlapping packages and utilities where one canonical dependency can replace several.
7. If the work is large, group changes into coherent upgrade waves, but still complete the modernization in this run unless blocked.

Hard-cut policy:
- Keep one canonical implementation path.
- Remove fallback behavior, compatibility branches, adapters, coercions, aliases, translation layers, dual-shape support, and legacy internal formats.
- Update all producers, consumers, fixtures, test builders, snapshots, and docs to the canonical shape.
- Do not add rejection logic or tests solely to memorialize old internal shapes.
- Preserve compatibility only where there is a concrete external boundary:
  - persisted external or user data
  - on-disk or database state that must still load
  - wire formats across process or service boundaries
  - documented public contracts
- If such a boundary exists, isolate it and name the exact file, function, and reason.

Clean-code policy:
- Make naming intention-revealing and searchable.
- Keep functions small and single-purpose.
- Do not mix abstraction levels within the same function.
- Prefer rewriting unclear code over commenting around it.
- Remove needless indirection, fragility, rigidity, and repetition.
- Favor straightforward data flow and explicit ownership.
- If a helper or abstraction no longer earns its keep after dependency upgrades, delete it.

Reducing-entropy policy:
- Bias toward deletion.
- Measure success by final codebase size and conceptual complexity, not by short-term convenience.
- Ask continuously:
  - What becomes obsolete after this upgrade?
  - What custom code can now be deleted?
  - What wrappers, facades, helper layers, and compatibility utilities can be removed?
  - What packages can be eliminated entirely?
- Prefer simpler composition over extra abstractions.
- Prefer fewer concepts, fewer codepaths, and less glue code.

Implementation rules:
1. Make the minimal architecture needed for the final modernized state, not the minimal diff.
2. Do not preserve old internal paths just because they exist.
3. Do not add new shims simply to make upgrades easier unless a real external boundary requires them.
4. Remove dead code, dead tests, dead comments, dead docs, and dead config in the same change set.
5. Update scripts and docs when commands, package-manager behavior, or architecture change.
6. Keep changes coherent and reviewable, but do not leave partial migrations behind.
7. Prefer dependency-native features over custom code whenever that reduces code and maintenance burden.

Verification requirements:
1. Run the full relevant validation suite for this repo:
   - install/sync
   - audit
   - outdated checks
   - lint
   - typecheck
   - unit/integration tests
   - build
   - repo-native CI validation scripts
2. Re-run vulnerability and outdated scans after changes.
3. Verify deprecated APIs are fully removed or intentionally isolated.
4. Verify removed dependencies are no longer referenced anywhere.
5. Verify obsolete compatibility code is gone.
6. If failures occur, fix them rather than stopping at partial progress unless genuinely blocked.

Reporting format:
1. Findings first:
   - vulnerabilities
   - outdated dependencies
   - deprecated APIs
   - abandoned packages
   - replaceable custom code
   - legacy/duplicate codepaths
2. Then the execution plan:
   - upgrade waves
   - affected packages
   - affected files
   - expected deletions
   - framework-specific migration points
   - risk points
3. Then execute the work.
4. Final report must include:
   - dependencies upgraded, replaced, and removed
   - vulnerabilities fixed
   - deprecated APIs removed
   - custom code deleted because dependencies now provide native functionality
   - compatibility code deleted under hard-cut policy
   - files changed
   - exact commands run and outcomes
   - verification results
   - residual risks or blocked items
   - net LOC added vs deleted, emphasizing total reduction

Quality bar:
- Do not stop at “dependencies updated.”
- Do not stop at “tests pass.”
- Finish the migration so the repo actually uses the modern dependency capabilities cleanly.
- Prefer a smaller, cleaner, more canonical codebase after the upgrade than before it.
```

