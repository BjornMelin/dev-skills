# Configuration and migration

## Version authority

Verified against stable **5.0.5** on 2026-10-27. This is a baseline, not permission to upgrade automatically. Check installed manifests, lockfile, runtime, addon peers, and the matching release before changing configuration. Keep core `@vitest/*` addons aligned with Vitest; community renderers and framework harnesses have their own version ranges.

5.0.5 declares Node `^22.12.0 || ^24.0.0 || >=26.0.0` and Vite `^6.4.0 || ^7.0.0 || ^8.0.0`. “Node 22+” is insufficient, and Node 23/25 are outside this range. Use the repository's supported runtime rather than introducing a new pin unnecessarily.

| Boundary | Consequence |
| --- | --- |
| 5.0.0 | New mock-history default, inline inheritance/sharing, strict browser matching, benchmark fixture, and removed entry points. |
| 5.0.3 | Cache-key generators scoped per project; cached imports revalidated; retry screenshot references corrected. |
| 5.0.4 | `fsModuleCache` defaults to `true`; browser instances obey worker limits correctly; experimental dependency recording added. |
| 5.0.5 | Server file-deny security fix; hook cleanup, cancellation, mock retention, and browser global-setup fixes. |
| 5.1.0-beta.1 | Prerelease, not the stable baseline. Do not assume `--last-failed`, `--since-last-run`, or its browser/fake-timer changes exist on 5.0.x. |

For UI, Browser Mode, or `api` users below 5.0.5, prioritize the matching patched release: GHSA-p458-7748-mcp3 covers sensitive file/API-token exposure. Plain runs without an HTTP server are not affected by that advisory. Never expose test control servers or publish their tokens as a convenience.

## Migrate only affected surfaces

For pre-v4 repositories, apply intervening migration requirements first. Record discovered tests and coverage files before changing discovery rules.

- Inline `test.projects` inherits the declaring config by default. Arrays merge, so repeated `setupFiles` can execute twice. Referenced config files/directories do not inherit implicitly. Nested referenced projects now work; merging a root config containing `projects` into its children can create recursive discovery.
- Compatible inline projects share one Vite server. `environment`, `env`, and `setupFiles` do not alone prevent sharing. Project `root`, `alias`, `browser`, optimizer, or Vite plugin/resolution changes can. Hoist identical plugins only when behavior is identical. Use `DEBUG=vitest:projects` to inspect decisions; opt out when plugins require separate initialization.
- Config lookup no longer climbs parent directories. From a subdirectory, pass the intended `--config` and verify root/discovery rather than duplicating config files.
- `clearMocks: true` clears history before each test, including history from `beforeAll`. Keep assertions and observed calls in the same test lifecycle. Restoration and implementation reset are separate policies.
- Unawaited asynchronous assertions fail. `expect.poll` enforces its deadline and supplies cancellation. Fix ownership/awaits, not warning suppression.
- Test full-name matching uses ` > ` between suite segments. Inspect `-t` expressions that span boundaries.
- Coverage globs match root-relative paths more precisely. Recheck the actual file set; glob-specific thresholds do not inherit top-level `perFile`.

## Public APIs and artifacts

Import application test APIs from `vitest`, configuration from `vitest/config`, Node/reporting APIs from `vitest/node`, runtime environment APIs from `vitest/runtime`, and browser context from `vitest/browser`. Do not use `vitest/internal/*`. Removed entry points include `vitest/reporters`, `vitest/coverage`, `vitest/environments`, and `vitest/snapshot`. Read installed exports before adapting a custom runner.

Custom matchers augment `vitest.Matchers<R, T>`, with return type first; global `jest.Matchers` declarations no longer supply Vitest types. Avoid custom matchers unless repetition or a domain contract warrants them.

Reports/attachments moved under `.vitest`; JSON/JUnit write files by default, not stdout. Coverage still defaults to `coverage/`. Keep reference screenshots committed separately from failure artifacts. Worker IDs are 1-based and not globally unique across pools, processes, or worktrees.

Sources: [migration](https://vitest.dev/guide/migration), [sharing](https://vitest.dev/config/sharedviteserver), [tagged package](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/package.json), [releases](https://github.com/vitest-dev/vitest/releases), [security advisory](https://github.com/vitest-dev/vitest/security/advisories/GHSA-p458-7748-mcp3).
