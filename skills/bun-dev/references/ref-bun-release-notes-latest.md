# Bun v1.3.13 Release Notes Snapshot

Source: <https://bun.com/blog/release-notes/bun-v1.3.13>

GitHub release: <https://github.com/oven-sh/bun/releases/tag/bun-v1.3.13>

GitHub compare:
<https://github.com/oven-sh/bun/compare/bun-v1.3.12...bun-v1.3.13>

Verified locally:

```bash
bun --version
# 1.3.13

bun --revision
# 1.3.13+bf2e2cecf
```

## Test Runner

`bun test` added scale-oriented flags:

- `--isolate`: fresh global object per test file in the same process.
- `--parallel[=N]`: worker-process parallelism; implies isolation.
- `--parallel-delay=<ms>`: delay before spawning more parallel workers.
- `--shard=M/N`: deterministic file sharding across CI jobs.
- `--changed[=<ref>]`: affected test selection from git changes.

Use these for repos already using `bun test`. Do not treat them as a reason to
rewrite mature Vitest, Jest, or Playwright suites without a migration plan.

## Package Manager

- `bun install` streams tarballs to disk, reducing large-install memory use.
- `bun install --linker=isolated` is much faster in peer-heavy monorepos.
- `BUN_FEATURE_FLAG_DISABLE_STREAMING_INSTALL=1` is the fallback switch if a
  verified streaming-install regression appears.
- `bun ci` remains the reproducible CI install spelling for frozen lockfiles.
- `--minimum-release-age=<seconds>` is available for fresh-publish risk
  mitigation.

## Runtime, Server, and Build

- Source-map memory use is substantially lower.
- Runtime memory is lower due allocator and scavenger changes.
- JavaScriptCore and WebCore were upgraded, improving language/runtime
  performance and fixing compatibility bugs.
- `new Response(Bun.file(path))` streams incrementally in more environments.
- `Bun.serve()` supports standard single-range requests for file-backed
  responses.
- `bun build --compile --target browser` inlines JS-imported file-loader assets
  into standalone HTML output.
- CSS bundling preserves top-level `@layer` ordering declarations.

## Crypto and Networking

- WebCrypto and `node:crypto` support SHA3-224, SHA3-256, SHA3-384, and
  SHA3-512.
- `SubtleCrypto.deriveBits()` supports X25519.
- WebSocket clients support `ws+unix://` and `wss+unix://`.
- Compatibility fixes landed for HTTP/2 h2c, `node:net` socket timeouts,
  WebSocket proxy/TLS edge cases, fetch aborts, N-API finalizers, filesystem
  watchers, profiling output, and process parent PID behavior.

## Rule Routing

- Test flags: `rules/test-bun-test-runner.md`
- CI installs and reproducibility: `rules/pm-bun-install-ci-frozen-lockfile.md`
- Package manager pin: `rules/pm-package-manager-field.md`
- Runtime APIs and file serving: `rules/runtime-bun-native-apis.md`
- Standalone HTML: `rules/build-bun-compile-browser.md`

## Release-Sync Note

The dev-skills-owned Rust implementation reports `verified_bun_version:
"1.3.13"` through `codex-dev --json bun references status`.
