# Validation Routing

Use this reference to select checks from touched files and risk without
rebuilding the repository's validation policy.

## Authority Order

1. Repository and nested agent/contributor instructions.
2. CI workflow commands and reusable workflows.
3. Validation manifests, task-runner config, and package scripts.
4. Language/tool configuration.
5. Neighboring tests and historical commands in docs.

CI and scripts may drift; inspect both. If they disagree, report the drift and
use the path that actually gates the target branch unless doing so is unsafe.

## Build the Touched Set

Determine:

- dirty tracked and untracked files in scope;
- branch diff from the intended merge base;
- generated consumers affected by source/config changes;
- package/workspace boundaries;
- public API, schema, auth/security, migration, provider, native, and release
  risks even when only one file changed.

Do not include unrelated dirty changes in fixes, but account for them when a
gate runs against the whole workspace.

## Gate Categories

Select only categories owned by repository evidence:

| Change/risk | Typical gate categories |
| --- | --- |
| Source logic | focused unit/integration tests, typecheck, lint |
| Public API or schema | contract tests, generated output, consumer checks, migration validation |
| UI | component/E2E, accessibility, browser proof, build, visual checks |
| Native | platform tests, typecheck, simulator/device smoke, build/config validation |
| Dependency/toolchain | install/lock integrity, audit, typecheck, build, compatibility profile |
| CI/config | syntax/schema validation, local reproduction of changed workflow commands |
| Docs | repository docs lint/link/contract checks |
| Security/auth | denied-path, isolation, secret/policy, integration checks |
| Release/ops | packaging, artifact, deploy dry-run or release verification if safe and authorized |

These are categories, not commands. Discover exact commands from the repo.

## Routing Procedure

1. List touched packages and risk boundaries.
2. Find scripts and CI jobs that mention those paths or packages.
3. Identify generated artifacts and prerequisite commands.
4. Choose the narrowest test that can fail for the changed behavior.
5. Choose the package/workspace gate that owns the touched set.
6. Add build, E2E, native, security, docs, or release gates only when repository
   policy or risk requires them.
7. Record environment/tool prerequisites before execution.
8. Run narrow-to-wide; stop on the first actionable failure unless independent
   gates can safely run in parallel.

## Repository Scripts First

Prefer, in order:

1. documented task or package script;
2. CI's exact command;
3. tool binary with committed configuration;
4. ad hoc runner flags only for diagnosis.

Do not replace a named script with a raw binary merely because the command
looks equivalent. The script may set projects, environment, setup, codegen,
timeouts, worker counts, reporters, or cleanup.

## Generated Output

When source/config owns generated files:

- identify the canonical generator;
- run it rather than hand-editing generated output;
- inspect generated diffs;
- run the repository's drift/check mode when available;
- do not regenerate unrelated artifacts.

## Environment and Providers

- Never expose secrets while matching CI environment.
- Prefer local fakes/emulators and deterministic fixtures.
- A missing required secret or provider prerequisite is a blocked environment,
  not a passing test.
- Do not mutate provider configuration or production data to reproduce a gate.
- Mark live-provider verification optional unless the repository explicitly
  requires it and the user authorizes the mutation.

## Handoff

Report:

- touched set and risk boundaries;
- exact commands selected and repository evidence for each;
- commands run with pass/fail/blocked result;
- first actionable error for failures;
- gates skipped and why;
- prerequisites unavailable;
- all remaining `UNVERIFIED` coverage.
