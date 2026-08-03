# Convex Test Audit and Review

Use this when the user invokes `$convex-test-guide /audit`,
`$convex-test-guide /review`, or asks for Convex backend test coverage,
test-quality, or `convex-test` adoption findings.

## Command grammar

- `/help`: print supported command syntax and descriptions.
- `/audit`: domain-bounded scan of backend Convex source, Convex integration
  tests, and Convex test utilities.
- `/audit full`: scan every supported repo source file except generated,
  vendored, build, and binary paths.
- `/audit backend`: scan `packages/backend/convex/**` for functions needing
  `convex-test` coverage.
- `/audit tests`: scan `packages/backend/test/convex/**` and Convex test utils
  for harness quality and anti-patterns.
- `/audit refresh`: run local package/source snapshot first, then audit.
- `/review`: use `/review diff` if changed files exist, otherwise `/review full`.
- `/review diff`: review unstaged and staged local diffs only.
- `/review pr [base]`: review committed and uncommitted changes against `base`;
  default base is `origin/main` when available, else `main`.
- `/review full`: review all Convex backend source, tests, and test utils.

## First command is report-only

Do not edit source on the first `/audit` or `/review` invocation. Produce a
complete report with an implementation queue. If the user replies `continue`,
implement all safe validated fixes from the queue, then report:

- completed findings
- findings still remaining
- reason each remaining finding was not implemented
- next batch of changes that another `continue` would attempt

Stop before destructive, security-sensitive, public API-breaking, schema
contract-breaking, migration-sensitive, or ambiguous changes. Explain the
blocker and the exact approval needed.

## Scanner

Run the bundled scanner first:

```bash
bun .agents/skills/convex-test-guide/scripts/convex-test-audit.mjs /audit
bun .agents/skills/convex-test-guide/scripts/convex-test-audit.mjs /review diff
bun .agents/skills/convex-test-guide/scripts/convex-test-audit.mjs /review pr origin/main
bun .agents/skills/convex-test-guide/scripts/convex-test-audit.mjs /help
```

Use `--format json` when you need machine-readable findings.

The script is an adaptive cascade entrypoint:

1. inventory supported, generated, vendored, unsupported, and missing files
2. use literal/regex candidate generation for high recall
3. report optional tool availability for `rg`, `ast-grep`, `codeql`,
   `semgrep`, `scip`, `zoekt`, `jq`, and `fzf`
4. leave compiler/dataflow/codemod verification to the agent before final
   findings or implementation

Do not treat every scanner hit as a final issue. Promote only validated hits to
the main findings list. Keep lower-confidence hits in a candidate appendix with
their reason and next check.

## What to look for

- Convex functions that need coverage with `convexTest(schema, convexModules)`
- tests that call `convexTest` without the repo schema/modules shape
- tests in forbidden paths or using `.convex-test.ts` suffixes
- file-level Vitest environment pragmas that bypass the `backend-convex` project
- scheduled-function tests missing fake timers or scheduler drain helpers
- sleeps, live network calls, or time-dependent behavior
- exact backend error-string assertions instead of stable product behavior or
  `ConvexError.data`
- repeated one-off seed/setup logic that belongs in
  `packages/backend/test_utils/convex/**`
- HTTP action tests that should use `t.fetch`
- auth branches that should use `t.withIdentity`

## Report format

Start with:

- scope scanned
- source stamp and package version
- tool capability matrix
- inventory coverage counts and excluded path categories
- top risks

Then list findings by severity: critical, high, medium, low, nit. Each finding
must include:

- file and line reference
- confidence: `validated`, `likely`, or `candidate`
- current pattern
- `convex-test` best-practice replacement
- why the replacement is safer or more deterministic
- implementation target files
- verification command(s)
- whether `continue` can safely implement it

End with:

- implementation queue for `continue`
- candidate appendix for unpromoted scanner hits
- explicitly deferred or blocked items
