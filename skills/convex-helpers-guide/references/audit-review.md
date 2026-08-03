# Convex Helpers Audit and Review

Use this when the user invokes `$convex-helpers-guide /audit`,
`$convex-helpers-guide /review`, or asks for package-leverage findings across a
repo.

## Command grammar

- `/help`: print supported command syntax and descriptions.
- `/audit`: domain-bounded scan of backend Convex, backend Convex tests, and
  Convex-consuming app/client code.
- `/audit full`: scan every supported repo source file except generated,
  vendored, build, and binary paths.
- `/audit backend`: scan `packages/backend/convex/**`.
- `/audit tests`: scan `packages/backend/test/convex/**` and Convex test utils.
- `/audit client`: scan Convex-consuming app code.
- `/audit refresh`: run local package/source snapshot first, then audit.
- `/review`: use `/review diff` if changed files exist, otherwise `/review full`.
- `/review diff`: review unstaged and staged local diffs only.
- `/review pr [base]`: review committed and uncommitted changes against `base`;
  default base is `origin/main` when available, else `main`.
- `/review full`: review all Convex backend, test, and client files.

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
bun .agents/skills/convex-helpers-guide/scripts/convex-helpers-audit.mjs /audit
bun .agents/skills/convex-helpers-guide/scripts/convex-helpers-audit.mjs /review diff
bun .agents/skills/convex-helpers-guide/scripts/convex-helpers-audit.mjs /review pr origin/main
bun .agents/skills/convex-helpers-guide/scripts/convex-helpers-audit.mjs /help
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

- repeated auth/org/ctx boilerplate that fits `customQuery`,
  `customMutation`, `customAction`, or `customCtx`
- repeated relationship point reads that fit `getOrThrow`, `getAll`,
  `getOneFrom`, `getManyFrom`, or `getManyVia`
- unbounded `.collect()` or post-filtering that should be indexed, paginated,
  streamed, or explicitly justified
- multi-source pagination that fits `stream`, `mergedStream`, or
  `MergedStream`
- manual page slicing that fits `getPage` or `paginator`
- invariant-maintenance mutation boilerplate that fits `Triggers` and
  `writerWithTriggers`
- manual CORS or Hono-like HTTP routing that fits `corsRouter` or Hono helpers
- manual retry/jitter/backoff action code that fits `makeActionRetrier` or
  `withJitter`
- manual local rate limiting that fits `defineRateLimits`, `rateLimit`, or
  `checkRateLimit`
- validator duplication that fits `typedV`, `doc`, `partial`, `literals`,
  `nullable`, `validate`, `parse`, or Zod conversion helpers
- client-side arg binding/session/cache patterns that fit `withArgs`,
  session helpers, or React cache helpers

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
- package-native or Convex-native replacement
- why the replacement is safer or simpler
- implementation target files
- verification command(s)
- whether `continue` can safely implement it

End with:

- implementation queue for `continue`
- candidate appendix for unpromoted scanner hits
- explicitly deferred or blocked items
