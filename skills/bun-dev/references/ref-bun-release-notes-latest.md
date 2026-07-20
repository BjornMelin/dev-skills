# Bun v1.3.14 Release Notes Snapshot

Source: <https://bun.com/blog/bun-v1.3.14>

GitHub release: <https://github.com/oven-sh/bun/releases/tag/bun-v1.3.14>

Verified locally:

```bash
bun --version
# 1.3.14
```

> Curated summary. The engine's `references sync` fetches the full upstream post; this
> file is hand-distilled down to the headline surface. For exhaustive detail (including
> the large bugfix list) open the source URL above or `bun.com/docs/llms.txt`.

## Package Manager

- **Global Virtual Store**: `bun install --linker=isolated` gains an opt-in shared store
  via `install.globalStore = true` in `bunfig.toml` (or `BUN_INSTALL_GLOBAL_STORE=1`).
  Packages materialize once into a global `<cache>/links/` dir; each project symlinks in.
  Warm installs drop from ~1 file-clone per file to ~1 `symlink()` per package (isolated
  warm install benchmark: 841 ms -> 115 ms). Experimental, off by default.
- `bun publish` now sends `readme` metadata to the registry.
- Several `bun install` / `bunx` fixes: fail fast on 4xx/5xx tarball errors and integrity
  mismatches instead of hanging; `bunx @scope/name` no longer runs an unrelated `$PATH`
  binary.

## Runtime, Server, and Build

- **`Bun.Image`**: built-in image processing (decode/transform/encode JPEG, PNG, WebP,
  GIF, BMP; HEIC/AVIF/TIFF on macOS/Windows) with zero native installs - a chainable,
  drop-in alternative to sharp. Works directly as a `Response`/request body.
- **HTTP/3 (QUIC) in `Bun.serve`**: `http3: true` alongside `tls` (experimental; do not
  deploy to production). Binds TCP for HTTP/1.1+2 and UDP for HTTP/3 on the same port;
  existing `fetch`/`routes` work unchanged. WebSocket-over-H3 and 0-RTT not yet supported.
- **Rewritten `fs.watch()` backend** on Linux/macOS/FreeBSD (inotify/FSEvents/kqueue
  directly): recursive watching now tracks directories created after `watch()` starts;
  delete-and-recreate emits `change` again; macOS uses one watcher thread instead of two.
- **`--no-orphans`** (`[run] noOrphans = true` / `BUN_FEATURE_FLAG_NO_ORPHANS=1`): Bun
  exits when its parent process dies and SIGKILLs descendants it spawned.
- **`process.execve(execPath, args, env)`** support (Node v24 parity).
- **`Bun.Terminal` on Windows** via ConPTY (`Bun.spawn({ terminal })`).
- **`using` / `await using` no longer lowered** when targeting Bun (`bun run`,
  `--target=bun`, `--compile`, `--bytecode`); JavaScriptCore runs them natively.

## fetch, Crypto, and Networking

- **Experimental HTTP/2 client for `fetch()`**: `fetch(url, { protocol: "http2" })` or
  `--experimental-http2-fetch` / `BUN_FEATURE_FLAG_EXPERIMENTAL_HTTP2_CLIENT=1`.
  Multiplexes concurrent requests over one connection; RFC 9113 conformance + DoS
  hardening.
- **Experimental HTTP/3 client for `fetch()`**: `fetch(url, { protocol: "http3" })`;
  optional Alt-Svc auto-upgrade via `--experimental-http3-fetch`.
- **Shared `SSL_CTX` cache** across all TLS APIs (`Bun.SQL`, Valkey, `node:tls`,
  WebSocket, `Bun.connect`): pools with identical TLS config reuse one context - large
  RSS reduction for MongoDB/Mongoose/mysql2 under connection churn.
- `perMessageDeflate: false` now suppresses the WebSocket upgrade extension header.
- `tls.getCACertificates('system')` works without `--use-system-ca`; Windows loads
  intermediate + TrustedPeople certs.

## Engine / Platform

- **JavaScriptCore upgraded** (565 upstream commits): faster async returns, `Array.shift`,
  short-string `JSON.parse`, `startsWith`/`endsWith`, `Intl`; WASM relaxed SIMD +
  Memory64; many correctness fixes.
- **SQLite updated to 3.53.0** (`bun:sqlite`).
- 1st-party native builds for **FreeBSD and Android**; smaller Linux/Windows binary;
  cross-language LTO (Zig <-> C++) on Linux; ~12% faster ESM module loading; reduced
  incremental-GC overhead. Bun Shell fixed 70+ bugs.

## Rule Routing

- Global Virtual Store / isolated linker / install perf -> `pm-linker-and-streaming-install`
- `Bun.serve` HTTP/3 and Bun-native servers/APIs -> `perf-prefer-bun-native-apis`
- CI installs and reproducibility -> `pm-bun-install-ci-frozen-lockfile`
- `bunfig.toml` config (globalStore, noOrphans) -> `tooling-bunfig`
- `bun test --isolate/--parallel/--changed` stability fixes -> `test-bun-test-runner`
- `Bun.Image`, `Bun.Terminal`, `process.execve`, fetch h2/h3 -> no dedicated rule yet;
  see `bun.com/docs/llms.txt`

## Release-Sync Note

The dev-skills-owned Rust engine reports `verified_bun_version: "1.3.14"` through
`codex-dev --json bun references status`.
