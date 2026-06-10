# Bun v1.3.13 Capability Map

Verified against Bun `v1.3.13` CLI help, official docs, Bun blog release notes,
Context7 `/oven-sh/bun`, and GitHub release metadata.

## New / important in v1.3.13

### `bun test` scale flags

- `bun test --isolate` runs each test file in a fresh global environment in the
  same process.
- `bun test --parallel[=N]` distributes files across workers and implies
  isolation.
- `bun test --shard=M/N` splits test files deterministically across CI jobs.
- `bun test --changed[=<ref>]` runs test files affected by changed files.
- `--parallel-delay=<ms>` controls worker ramp-up.

Route to:

- `test-bun-test-runner`

Adoption note:

- Use these flags for repos that actually run `bun test`. If a repo has an
  established Vitest/Jest/Playwright harness, do not rewrite the runner just to
  use these flags; add a separate migration plan.

### Package manager memory and install improvements

- `bun install` streams tarballs to disk and avoids materializing full archives
  in memory.
- `bun install --linker=isolated` is materially faster in peer-heavy monorepos.
- `bun ci` is the CI spelling for frozen lockfile installs.
- `--minimum-release-age=<seconds>` gates newly published packages.

Route to:

- `pm-bun-install-ci-frozen-lockfile`
- `pm-package-manager-field`

Fallback knob:

- `BUN_FEATURE_FLAG_DISABLE_STREAMING_INSTALL=1` disables streaming extraction
  only if a verified install regression requires it.

### Runtime and build memory improvements

- Source maps use a compact internal representation with much lower memory use.
- Bun runtime baseline memory is lower due allocator and scavenger upgrades.
- `bun build --compile` benefits from lower source-map memory.

Use when:

- long-running Bun processes, source-map-heavy CLIs, and compiled Bun binaries
  show memory pressure.

### Bun.serve and file response improvements

- File-backed `Response(Bun.file(path))` streams incrementally over SSL and on
  Windows.
- `Bun.serve()` supports `Range` requests for file-backed responses in static
  routes and dynamic handlers.

Route to:

- `runtime-bun-native-apis`

### Crypto, WebSocket, and compatibility improvements

- SHA3-224/256/384/512 works in WebCrypto and `node:crypto`.
- `SubtleCrypto.deriveBits()` supports X25519.
- WebSocket clients support `ws+unix://` and `wss+unix://`.
- JSC, WebCore event handling, zlib-ng compression, HTTP/2, net socket timeout,
  N-API finalizer, `fetch`, WebSocket proxy, and CSS parser bugs were fixed.
- CSS bundling preserves top-level `@layer` ordering declarations, important
  for Tailwind CSS cascade layers.

Use when:

- Bun is the runtime for crypto, local socket transports, WebSocket clients,
  Tailwind/CSS bundling, or Node compatibility work.

## Still important from v1.3.12

### Browser automation

- `Bun.WebView` is a built-in headless automation API
- backends:
  - WebKit on macOS
  - Chrome cross-platform
- use `await using`
- selector actions auto-wait for actionability

Route to:

- `runtime-webview-automation`
- `runtime-bun-native-apis`

### Markdown in terminal

- `bun ./file.md` renders Markdown to ANSI in terminal
- `Bun.markdown.ansi()` renders programmatically
- docs still mark markdown APIs unstable

Route to:

- `runtime-markdown-entrypoints`
- `runtime-bun-native-apis`

### Cron split is now first-class

- `Bun.cron(schedule, handler)`:
  - in-process
  - UTC
  - shared state
  - no overlap
- `Bun.cron(path, schedule, title)`:
  - OS-level
  - local time
  - persistent

Route to:

- `runtime-cron-in-process-vs-os`

### JS engine / language surface

- JavaScriptCore upgrade with native `using` / `await using`
- async stack traces improved for native errors and `Error.captureStackTrace`

Use when:

- cleanup / disposal semantics matter
- async debugging quality matters

### Build / packaging

- `bun build --compile` Linux executables now use embedded ELF section data
- standalone HTML remains important: `bun build --compile --target=browser`
- v1.3.13 fixes JS-imported file-loader assets in standalone HTML output
- `bun build` faster on low-core machines
- `--metafile-md` remains an LLM-friendly graph artifact

Route to:

- `build-compile-executables`
- `build-bun-compile-browser`
- `build-bun-build-bundler`

### Runtime / infra improvements

- `URLPattern` faster and cleaner around `RegExp.lastMatch`
- `Bun.stripANSI` / `Bun.stringWidth` much faster
- `Bun.Glob.scan()` faster
- cgroup-aware `availableParallelism()` / `hardwareConcurrency` on Linux
- HTTPS proxy CONNECT tunnels now keep-alive and reuse correctly
- `Bun.serve()` uses `TCP_DEFER_ACCEPT` on Linux

Use when:

- terminal UX, log formatting, or CLI rendering matters
- Docker / k8s CPU limits matter
- proxy-heavy HTTP clients matter
- Bun server perf matters

## CLI surfaces worth keeping in muscle memory

- top level: `run`, `test`, `x`, `repl`, `exec`, `install`, `add`, `remove`, `update`, `audit`, `outdated`, `link`, `unlink`, `publish`, `patch`, `pm`, `info`, `why`, `build`, `init`, `create`, `upgrade`
- workspace flags: `--filter`, `--workspaces`, `--parallel`, `--sequential`, `--no-exit-on-error`, `--elide-lines`
- test flags: `--changed`, `--isolate`, `--parallel`, `--parallel-delay`, `--shard`, `--retry`, `--pass-with-no-tests`, `--only-failures`, `--coverage`, `--bail`, `--randomize`
- build flags: `--compile`, `--target`, `--metafile`, `--metafile-md`, `--bytecode`, `--env`

## Skill impact

Need direct rule coverage:

- Bun runtime vs. Node runtime choice
- Bun-native APIs
- `Bun.WebView`
- `Bun.cron` overload split
- Markdown terminal entrypoints
- standalone HTML
- workspace script orchestration
- `bun test` isolation, parallelism, changed-file selection, and sharding
- isolated linker and streaming install behavior

Docs-only, no dedicated rule needed:

- deep JSC perf wins
- specific bugfix lists
- TLS root store changes
- lower-level UDP / unix socket lifecycle details
