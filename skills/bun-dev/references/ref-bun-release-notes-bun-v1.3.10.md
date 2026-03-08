#### To install Bun

```sh#curl
$ curl -fsSL https://bun.sh/install | bash
```

```sh#npm
$ npm install -g bun
```

```sh#powershell
$ powershell -c "irm bun.sh/install.ps1|iex"
```

```sh#scoop
$ scoop install bun
```

```sh#brew
$ brew tap oven-sh/bun
$ brew install bun
```

```sh#docker
$ docker pull oven/bun
$ docker run --rm --init --ulimit memlock=-1:-1 oven/bun
```

#### To upgrade Bun

```sh
$ bun upgrade
```

## New REPL

<!-- https://github.com/oven-sh/bun/commit/fa3a30f075ee208331c497fde1ea5cdae682e17f -->

Bun's REPL has been completely rewritten in Zig, replacing the previous third-party npm package. The new REPL starts instantly without downloading any packages, and includes a full-featured terminal UI.

<blockquote class="twitter-tweet"><p lang="en" dir="ltr">In the next version of Bun<br><br>Bun gets a native REPL <a href="https://t.co/RLtaUymgWu">pic.twitter.com/RLtaUymgWu</a></p>&mdash; Jarred Sumner (@jarredsumner) <a href="https://twitter.com/jarredsumner/status/2026587131997831604?ref_src=twsrc%5Etfw">February 25, 2026</a></blockquote> <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

Features:

- **Copy to clipboard** - `.copy` command copies the expression to clipboard
- **Top-level await** - you can use it.
- **ESM import & require** - all the ways to load modules just work.
- **Syntax highlighting** — JavaScript code is colorized as you type.
- **Line editing with Emacs keybindings** — `Ctrl+A/E` to jump to start/end of line, `Ctrl+K/U` to kill to end/start, `Ctrl+W` to delete word backward, `Ctrl+L` to clear screen, and arrow key navigation.
- **Persistent history** — Command history is saved to `~/.bun_repl_history` and navigable with Up/Down arrows or `Ctrl+P/N`.
- **Tab completion** — Complete object properties and REPL commands.
- **Multi-line input** — Automatic continuation detection for incomplete expressions.
- **REPL commands** — `.help`, `.exit`, `.clear`, `.load`, `.save`, `.editor`.
- **Special variables** — `_` holds the last expression result, `_error` holds the last error.
- **Proper REPL semantics** — `const` and `let` declarations are hoisted to `var` for persistence across lines, top-level `await` works out of the box, `import` statements are converted to dynamic imports, and object literals like `{ a: 1 }` are detected without needing parentheses:

```js
> const x = 42
> x + 1
43
> await fetch("https://example.com").then(r => r.status)
200
> import { readFile } from "fs/promises"
> { name: "bun", version: Bun.version }
{ name: "bun", version: "1.3.1" }
```

## `--compile --target=browser` for self-contained HTML output

<!-- https://github.com/oven-sh/bun/commit/b817abe55ef8f9dd0d34ee4c8c0b3dc2ddfc5303 -->

You can now use `bun build --compile --target=browser` to produce self-contained HTML files with all JavaScript, CSS, and assets inlined directly into the output. This supports TypeScript, JSX, React, CSS, ESM, CJS, and everything else Bun's bundler already supports.

This is useful for distributing `.html` files that work via `file://` URLs without needing a web server or worrying about CORS restrictions.

- `<script src="...">` tags become inline `<script>` with bundled code
- `<link rel="stylesheet">` tags become inline `<style>` tags
- Asset references (including CSS `url()`) become `data:` URIs

**CLI:**

```sh
bun build --compile --target=browser ./index.html
```

**API:**

```js
await Bun.build({
  entrypoints: ["./index.html"],
  target: "browser",
  compile: true,
});
```

All entrypoints must be `.html` files. Cannot be used with `--splitting`.

## TC39 Standard ES Decorators

<!-- https://github.com/oven-sh/bun/commit/ce715b5a0f023732723ebe1f1e6502acf025646e -->

Bun's transpiler now fully supports [TC39 stage-3 standard ES decorators](https://github.com/tc39/proposal-decorators) — the non-legacy variant used when `experimentalDecorators` is **not** enabled in your `tsconfig.json`.

This has been one of the most requested features since 2023. Previously, Bun only supported legacy/experimental TypeScript decorators, which meant code using the modern decorator spec — including the `accessor` keyword, decorator metadata via `Symbol.metadata`, and the `ClassMethodDecoratorContext`/`ClassFieldDecoratorContext` APIs — would either fail to parse or produce incorrect results.

Now, all of the following work correctly:

```ts
function logged(originalMethod: any, context: ClassMethodDecoratorContext) {
  const name = String(context.name);
  return function (this: any, ...args: any[]) {
    console.log(`Entering ${name}`);
    const result = originalMethod.call(this, ...args);
    console.log(`Exiting ${name}`);
    return result;
  };
}

class Example {
  @logged
  greet(name: string) {
    console.log(`Hello, ${name}!`);
  }
}

new Example().greet("world");
// Entering greet
// Hello, world!
// Exiting greet
```

Auto-accessors with the `accessor` keyword are now supported, including on private fields:

```ts
import { Signal } from "signal-polyfill";

function signal(target: any) {
  const { get } = target;
  return {
    get() {
      return get.call(this).get();
    },
    set(value: any) {
      get.call(this).set(value);
    },
    init(value: any) {
      return new Signal.State(value);
    },
  };
}

class Counter {
  @signal accessor #value = 0;

  get value() {
    return this.#value;
  }
  increment() {
    this.#value++;
  }
}

const c = new Counter();
c.increment();
console.log(c.value); // 1
```

Field decorators with `addInitializer`, decorator metadata, and correct evaluation ordering all work as specified:

```ts
function wrap<This, T>(value: T, ctx: ClassFieldDecoratorContext<This, T>) {
  ctx.addInitializer(function () {
    console.log("Initialized", this);
  });
  return (initialValue: T) => initialValue;
}

class A {
  @wrap
  public a: number = 1;
}

const a = new A(); // "Initialized" A {}
```

### What's supported

| Feature                            | Details                                                                 |
| ---------------------------------- | ----------------------------------------------------------------------- |
| Method/getter/setter decorators    | Static and instance, public and private                                 |
| Field decorators                   | Initializer replacement + `addInitializer`                              |
| Auto-accessor (`accessor` keyword) | Public and private fields                                               |
| Class decorators                   | Statement and expression positions                                      |
| Decorator metadata                 | `Symbol.metadata` support                                               |
| Evaluation order                   | Decorator expressions and computed keys evaluated in spec-defined order |

Legacy decorators (`experimentalDecorators: true` in `tsconfig.json`) continue to work as before.

## Faster event loop on macOS & Linux

<blockquote class="twitter-tweet"><p lang="en" dir="ltr">Ensuring everyone really understands. <a href="https://t.co/REGFQ2se1G">https://t.co/REGFQ2se1G</a> <a href="https://t.co/jvoXgSVyYe">pic.twitter.com/jvoXgSVyYe</a></p>&mdash; Ben Dicken (@BenjDicken) <a href="https://twitter.com/BenjDicken/status/2021254589945872666?ref_src=twsrc%5Etfw">February 10, 2026</a></blockquote> <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

## Windows ARM64 Support

<!-- https://github.com/oven-sh/bun/commit/30e609e08073cf7114bfb278506962a5b19d0677 -->

Bun now natively supports Windows on ARM64 (Snapdragon, etc.). You can install and run Bun on ARM64 Windows devices, and cross-compile standalone executables targeting `bun-windows-arm64`.

```js
await Bun.build({
  entrypoints: ["./path/to/my/app.ts"],
  compile: {
    target: "bun-windows-arm64",
    outfile: "./myapp", // .exe added automatically
  },
});
```

Or from the CLI:

```bash
$ bun build --compile --target=bun-windows-arm64 ./path/to/my/app.ts --outfile myapp
```

## Barrel Import Optimization

When you `import { Button } from 'antd'`, the bundler normally has to parse every file that `antd/index.js` re-exports — potentially thousands of modules. Bun's bundler now detects pure barrel files (re-export index files) and **only parses the submodules you actually use**.

<blockquote class="twitter-tweet"><p lang="en" dir="ltr">In the next version of Bun<br><br>Bun&#39;s bundler &amp; frontend dev server gets automatic barrel-file optimization. <br><br>This makes libraries like `lucida-react` build up to 2x faster <a href="https://t.co/LxS0Y4VjcI">pic.twitter.com/LxS0Y4VjcI</a></p>&mdash; Jarred Sumner (@jarredsumner) <a href="https://twitter.com/jarredsumner/status/2021778115312464248?ref_src=twsrc%5Etfw">February 12, 2026</a></blockquote> <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

This works in two modes:

- **Automatic mode:** Packages with `"sideEffects": false` in their `package.json` get barrel optimization automatically — no configuration needed.
- **Explicit mode:** Use the new `optimizeImports` option in `Bun.build()` for packages that don't have `"sideEffects": false`.

```ts
await Bun.build({
  entrypoints: ["./app.ts"],
  optimizeImports: ["antd", "@mui/material", "lodash-es"],
});
```

A file qualifies as a barrel if every named export is a re-export (`export { X } from './x'`). If a barrel file has any local exports, or if any importer uses `import *`, all submodules are loaded as usual.

`export *` re-exports are always loaded to avoid circular resolution issues — only named re-exports that aren't used by any importer are deferred. Multi-level barrel chains (A re-exports from B re-exports from C) are handled automatically via BFS un-deferral.

## Fewer closures in bundled output

To make ESM & CJS work as people expect, all bundlers must generate additional wrapping code around modules. This wrapper code has some overhead. And in v1.3.10, Bun's bundled output for ESM & CJS projects now has significantly less overhead.

Measured on a ~23 MB single-bundle app with 600+ React imports:

| Metric               | Before  | After   | Delta              |
| -------------------- | ------- | ------- | ------------------ |
| **Total objects**    | 745,985 | 664,001 | **−81,984 (−11%)** |
| **Heap size**        | 115 MB  | 111 MB  | **−4 MB**          |
| GetterSetter         | 34,625  | 13,428  | −21,197 (−61%)     |
| Function             | 221,302 | 197,024 | −24,278 (−11%)     |
| JSLexicalEnvironment | 70,101  | 44,633  | −25,468 (−36%)     |

These improvements apply automatically to all `Bun.build()` and `bun build` output — no code changes required.

## `--retry` flag for `bun test`

<!-- https://github.com/oven-sh/bun/commit/099b5e430c68302baba46627139b5dfd49b6a0bb -->

You can now set a default retry count for all tests using the `--retry` flag. This is useful for handling flaky tests in CI environments without adding `{ retry: N }` to every individual test.

```sh
# Retry all failing tests up to 3 times
bun test --retry 3
```

Per-test `{ retry: N }` options still take precedence over the global flag:

```ts
import { test, expect } from "bun:test";

// Uses the global --retry count
test("flaky network call", async () => {
  const res = await fetch("https://example.com/api");
  expect(res.ok).toBe(true);
});

// Overrides the global --retry count
test("very flaky test", { retry: 5 }, () => {
  // ...
});
```

You can also configure this in `bunfig.toml`:

```toml
[test]
retry = 3
```

When using the JUnit XML reporter, each retry attempt is now emitted as a separate `<testcase>` entry. Failed attempts include a `<failure>` element, followed by the final passing `<testcase>`. This gives CI systems and flaky test detection tools per-attempt timing and result data using standard JUnit XML.

Thanks to @alii for the contribution!

## `ArrayBuffer` output for `Bun.generateHeapSnapshot("v8")`

<!-- https://github.com/oven-sh/bun/commit/77b640641537258727f17804d4b3eba944fefec3 -->

`Bun.generateHeapSnapshot("v8")` now accepts an optional second argument `"arraybuffer"` to return the heap snapshot as an `ArrayBuffer` instead of a string. This avoids the overhead of creating a JavaScript string for large snapshots and prevents potential crashes when heap snapshots approach the max `uint32` string length.

The `ArrayBuffer` contains UTF-8 encoded JSON that can be written directly to a file or decoded with `TextDecoder`:

```js
const snapshot = Bun.generateHeapSnapshot("v8", "arraybuffer");

// Write directly to a file — no string conversion needed
await Bun.write("heap.heapsnapshot", snapshot);

// Or decode and parse if needed
const parsed = JSON.parse(new TextDecoder().decode(snapshot));
```

## TLS keepalive for custom SSL configs (mTLS)

<!-- https://github.com/oven-sh/bun/commit/e735bffaa9543b3a119b3c25b26b05bfae127a26 -->

Previously, all HTTP connections using custom TLS configurations — such as client certificates (mTLS) or custom CA certificates — had keepalive disabled, forcing a new TCP+TLS handshake on every request.

Custom TLS connections now properly participate in keepalive pooling. Identical TLS configurations are deduplicated via a global registry with reference counting, and the SSL context cache uses bounded LRU eviction (max 60 entries, 30-minute TTL).

This is automatically enabled when using `fetch()` or `bun install`.

## Updated Root Certificates

<!-- https://github.com/oven-sh/bun/commit/883e43c37100c3dd4351e42cd4d477ee5f195a8d -->

Bun's bundled root certificates have been updated from NSS 3.117 to NSS 3.119 (Firefox 147.0.3). This removes 4 distrusted CommScope root certificates per Mozilla's NSS 3.118 changes:

- CommScope Public Trust ECC Root-01
- CommScope Public Trust ECC Root-02
- CommScope Public Trust RSA Root-01
- CommScope Public Trust RSA Root-02

This update resolves TLS connection failures that some users experienced after Cloudflare rotated `example.com`'s certificate to a chain terminating at the removed `AAA Certificate Services` (Comodo) root CA.

## Upgraded JavaScriptCore Engine

<!-- https://github.com/oven-sh/bun/commit/4d9752a1f09b0a4ce3a1910374b80dcff6006e23 -->

Bun's underlying JavaScript engine (JavaScriptCore) has been upgraded, bringing several performance improvements and bug fixes.

### Deep Rope String Slicing — 168x faster

Repeated string concatenation using `+=` previously created deeply nested rope strings that caused O(n²) behavior when slicing. The engine now limits rope traversal depth and falls back to flattening the string, dramatically improving performance.

```js
let s = "";
for (let i = 0; i < 100_000; i++) {
  s += "A";
}
// Slicing this string is now up to 168x faster
s.slice(0, 100);
```

### `String.prototype.endsWith` — up to 10.5x faster

`String.prototype.endsWith` is now optimized in the DFG/FTL JIT tiers with a dedicated intrinsic. Constant-foldable cases are up to 10.5x faster, and the general case is 1.45x faster.

```js
const str = "hello world";
str.endsWith("world"); // up to 10.5x faster when constant-folded
```

### RegExp Flag Getters — 1.6x faster

RegExp flag property getters (`.global`, `.ignoreCase`, `.multiline`, `.dotAll`, `.sticky`, `.unicode`, `.unicodeSets`, `.hasIndices`) now have inline cache and DFG/FTL support, making them ~1.6x faster.

### `Intl.formatToParts` — up to 1.15x faster

`Intl` `formatToParts` methods now use pre-built structures for returned part objects, reducing allocation overhead.

### Other Engine Improvements

- `BigInt` values now store digits inline, eliminating a separate allocation and pointer indirection
- String iterator creation is now optimized in DFG/FTL, enabling allocation sinking
- Integer modulo operations in DFG/FTL now avoid expensive `fmod` double operations when inputs are integer-like
- The JIT worklist thread count has been increased from 3 to 4
- Register allocator improvements for better spill slot coalescing

### Bug Fixes

- Fixed: `RegExp.prototype.test()` returning incorrect results due to stale captures in FixedCount groups (@pchasco)
- Fixed: Infinite loop in RegExp JIT when using non-greedy backreferences to zero-width captures (@pchasco)
- Fixed: Incorrect RegExp backtracking from nested alternative end branches (@pchasco)
- Fixed: WebAssembly `ref.cast`/`ref.test` producing wrong results due to inverted condition in B3 optimization (@nickaein)

## `structuredClone` is up to 25x faster for arrays

<!-- https://github.com/oven-sh/bun/commit/0f43ea9becb629711c06d48089b7ee8e9eab325e -->

`structuredClone` and `postMessage` now have a fast path when the root value is a dense array of primitives or strings. Instead of going through the full serialization/deserialization machinery, Bun keeps data in native structures and uses `memcpy` where possible.

This optimization applies automatically when cloning arrays of numbers, strings, booleans, `null`, or `undefined` — the most common case for `postMessage` payloads and deep copies.

```js
const numbers = Array.from({ length: 1000 }, (_, i) => i);
structuredClone(numbers); // 25.3x faster

const strings = Array.from({ length: 100 }, (_, i) => `item-${i}`);
structuredClone(strings); // 2.2x faster

const mixed = [1, "hello", true, null, undefined, 3.14];
structuredClone(mixed); // 2.3x faster
```

| Benchmark                         | Before    | After     | Speedup   |
| --------------------------------- | --------- | --------- | --------- |
| `structuredClone([10 numbers])`   | 308.71 ns | 40.38 ns  | **7.6x**  |
| `structuredClone([100 numbers])`  | 1.62 µs   | 86.87 ns  | **18.7x** |
| `structuredClone([1000 numbers])` | 13.79 µs  | 544.56 ns | **25.3x** |
| `structuredClone([10 strings])`   | 642.38 ns | 307.38 ns | **2.1x**  |
| `structuredClone([100 strings])`  | 5.67 µs   | 2.57 µs   | **2.2x**  |
| `structuredClone([10 mixed])`     | 446.32 ns | 198.35 ns | **2.3x**  |

Non-eligible inputs (objects, nested arrays) are unchanged with no regression.

Thanks to @sosukesuzuki for the contribution!

## `structuredClone` is faster for arrays of objects

<!-- https://github.com/oven-sh/bun/commit/fa78d2b408a42b54bf7d49ea6a5392412d7a1a3d -->

`structuredClone` and `postMessage` now use a fast path when cloning dense arrays of simple objects, completely bypassing byte-buffer serialization. This is the most common real-world pattern — arrays of flat objects with primitive or string values.

```js
// This is now 1.7x faster than before
const data = [
  { name: "Alice", age: 30 },
  { name: "Bob", age: 25 },
];

const cloned = structuredClone(data);
```

When the array contains objects that share the same shape (same property names in the same order), a structure cache skips repeated property transitions during deserialization — making same-shape object arrays especially fast.

This builds on the existing fast paths for dense arrays of primitives and strings (up to 25x faster for integer arrays), extending the optimization to the object case.

| Benchmark       | Node.js v24.12 | Bun v1.3.8 | Bun v1.3.1                 |
| --------------- | -------------- | ---------- | -------------------------- |
| `[10 objects]`  | 2.83 µs        | 2.72 µs    | **1.56 µs** (1.7x faster)  |
| `[100 objects]` | 24.51 µs       | 25.98 µs   | **14.11 µs** (1.8x faster) |

The fast path falls back to normal serialization for objects with getters/setters, nested objects/arrays, non-enumerable properties, or elements like `Date`, `RegExp`, `Map`, `Set`, and `ArrayBuffer`.

Thanks to @sosukesuzuki for the contribution!

## Faster `structuredClone` for numeric arrays

<!-- https://github.com/oven-sh/bun/commit/3debd0a2d2b47d846b08efa3138e4b0f4c40e393 -->

Eliminated a redundant zero-fill in the `structuredClone` fast path for `Int32` and `Double` arrays. Previously, an internal buffer was zero-initialized and then immediately overwritten with the actual data. Now the buffer is constructed directly from the source data in a single copy.

Thanks to @sosukesuzuki for the contribution!

## `Buffer.slice()` / `Buffer.subarray()` is ~1.8x faster

<!-- https://github.com/oven-sh/bun/commit/9484218ba455ba591ebf75ad4fca98b012f3dd13 -->

`Buffer.slice()` and `Buffer.subarray()` have been moved from a JS builtin to a native C++ implementation, eliminating closure allocations and JS→C++ constructor overhead on every call. An int32 fast path skips `toNumber()` coercion when arguments are already integers — the common case for calls like `buf.slice(0, 10)`.

| Benchmark                        | Before   | After        | Speedup   |
| -------------------------------- | -------- | ------------ | --------- |
| `Buffer(64).slice()`             | 27.19 ns | **14.56 ns** | **1.87×** |
| `Buffer(1024).slice()`           | 27.84 ns | **14.62 ns** | **1.90×** |
| `Buffer(1M).slice()`             | 29.20 ns | **14.89 ns** | **1.96×** |
| `Buffer(64).slice(10)`           | 30.26 ns | **16.01 ns** | **1.89×** |
| `Buffer(1024).slice(10, 100)`    | 30.92 ns | **18.32 ns** | **1.69×** |
| `Buffer(1024).slice(-100, -10)`  | 28.82 ns | **17.37 ns** | **1.66×** |
| `Buffer(1024).subarray(10, 100)` | 28.67 ns | **16.32 ns** | **1.76×** |

Thanks to @sosukesuzuki for the contribution!

## `path.parse()` is 2.2–7x faster

<!-- https://github.com/oven-sh/bun/commit/e29e830a2559083536974e65bfd720b516e81db5 -->

`path.parse()` now uses a pre-built object structure for its return value, avoiding repeated property transitions on every call. This brings **~2.2–2.8x** speedups for typical paths and up to **~7x** for edge cases like empty strings.

```js
import { posix } from "path";

// 2.2x faster (119ns vs 267ns)
posix.parse("/home/user/dir/file.txt");
// => { root: "/", dir: "/home/user/dir", base: "file.txt", ext: ".txt", name: "file" }

// 7x faster (21ns vs 152ns)
posix.parse("");
// => { root: "", dir: "", base: "", ext: "", name: "" }
```

| Path                        | Before    | After     | Speedup   |
| --------------------------- | --------- | --------- | --------- |
| `"/home/user/dir/file.txt"` | 266.71 ns | 119.62 ns | **2.23x** |
| `"/home/user/dir/"`         | 239.10 ns | 91.46 ns  | **2.61x** |
| `"file.txt"`                | 232.55 ns | 89.20 ns  | **2.61x** |
| `"/root"`                   | 246.75 ns | 92.68 ns  | **2.66x** |
| `""`                        | 152.19 ns | 20.72 ns  | **7.34x** |

Thanks to @sosukesuzuki for the contribution!

## Fixed: `Bun.spawn()` stdio pipes breaking Python asyncio-based MCP servers

<!-- https://github.com/oven-sh/bun/commit/b2d8504a09a6cf9f3282418570fa7205870bfc94 -->

Bun's subprocess stdio pipes used `shutdown()` calls on their underlying socketpairs to make them unidirectional. On `SOCK_STREAM` sockets, `shutdown(SHUT_WR)` sends a FIN to the peer — which caused programs that poll their stdio file descriptors for readability (like Python's `asyncio.connect_write_pipe()`) to interpret it as "connection closed" and tear down their transport prematurely.

This broke **all Python MCP servers** using the `model_context_protocol` SDK whenever they took more than a few seconds to initialize. The `shutdown()` calls have been removed entirely — the socketpairs are already used unidirectionally by convention, and the calls provided no functional benefit.

```js
// Python MCP servers spawned via Bun.spawn() now work correctly
const proc = Bun.spawn({
  cmd: ["python3", "mcp_server.py"],
  stdin: "pipe",
  stdout: "pipe",
  stderr: "pipe",
});

// Previously, the Python server's asyncio write transport would
// be torn down after a few seconds of initialization delay.
// Now it stays open as expected.
const response = await new Response(proc.stdout).text();
```

## Bugfixes

### Node.js compatibility improvements

- Fixed: `AsyncLocalStorage` context not being preserved in `stream.finished` callbacks, causing `getStore()` to return `undefined` instead of the expected value
- Fixed: `Error.captureStackTrace(e, fn)` with a function not in the call stack now correctly returns the error name and message (e.g. `"Error: test"`) instead of `undefined`, matching Node.js behavior
- Fixed: `fs.watch` and `fs.watchFile` not properly handling `file:` URL strings with percent-encoded characters (e.g. `%20` for spaces)
- Fixed: `node:http` sending duplicate `Transfer-Encoding: chunked` headers when explicitly set via `res.writeHead()`, which caused nginx 1.25+ to return 502 errors (@psmamps)
- Fixed: `http.ClientRequest.write()` called multiple times was stripping the explicitly-set `Content-Length` header and switching to `Transfer-Encoding: chunked`, breaking binary file uploads (e.g. Vercel CLI). Bun now preserves `Content-Length` when explicitly set, matching Node.js behavior.
- Fixed: `OutgoingMessage.setHeaders()` incorrectly throwing `ERR_HTTP_HEADERS_SENT`
- Fixed: HTTP response splitting vulnerability in `node:http`. Thanks to @VenkatKwest for reporting this!
- Fixed: Crash when accessing `X509Certificate.issuerCertificate`
- Fixed: Rare crash in `napi_close_callback_scope`
- Fixed: ref count leak in `setImmediate` when the timer's JS object was garbage collected before the immediate task ran
- Fixed: dynamic `import()` of unknown `node:` modules (like `node:sqlite`) inside CJS files no longer fails at transpile time, allowing try/catch to handle the error gracefully at runtime. This fixes Next.js builds with turbopack + `cacheComponents: true` + Better Auth, where Kysely's dialect detection uses `import("node:sqlite")` inside a try/catch.
- Fixed: three GC safety issues that could cause crashes during garbage collection marking, most notably affecting projects using `module._compile` overrides (`ts-node`, `pirates`, `@swc-node/register`, etc.) where an unvisited write barrier could lead to use-after-free crashes
- Fixed: potential GC-related crashes when constructing objects with string values (e.g., HTTP headers, SQLite column names) by avoiding GC allocations inside `ObjectInitializationScope` (thanks @sosukesuzuki!)
- Fixed: crash on older Linux kernels (< 3.17, e.g. Synology NAS) where the `getrandom()` syscall doesn't exist, causing a panic with `"getrandom() failed to provide entropy"`. Bun now falls back to `/dev/urandom` via BoringSSL on these systems.
- Fixed: Socket `recvfrom` failing with `EINVAL` on gVisor-based environments (e.g. Google Cloud Run) due to invalid `MSG_NOSIGNAL` flag being passed to receive operations
- Fixed: a crash on Windows (`OutOfMemory` panic) in `node:fs` path handling when the system is under memory pressure by removing an unnecessary 64KB buffer allocation for paths with drive letter
- Fixed: memory leak when upgrading TCP sockets to TLS in node:tls (thanks to @alanstott!)

### Bun APIs

- Fixed: Fuzzer-detected crash when using `Bun.spawn` with `stdin: new Response(data)` concurrently with `Bun.file().exists()` calls and other spawned process stdout reads
- Fixed: Fuzzer-detected crash in `Bun.spawn`/`Bun.spawnSync` caused by integer overflow when the command array has a spoofed `.length` near `u32` max
- Fixed: Fuzzer-detected crash caused by a double-free in `Bun.plugin.clearAll()` that could corrupt the heap allocator during Worker termination or VM destruction
- Fixed: Fuzzer-detected crash when calling `Listener.getsockname()` without an object argument or with a non-object argument (e.g. `undefined`, `123`, `"foo"`) due to a null pointer dereference
- Fixed: `Bun.stripANSI()` hanging indefinitely in certain cases
- Fixed: crash when resolving `bun:main` before the entry point is generated, such as in HTML entry points or the test runner
- Fixed: `db.close(true)` throwing "database is locked" after using `db.transaction()` due to transaction controller prepared statements not being finalized on close
- Fixed: `bun:sql` PostgreSQL client now uses constant-time comparison for SCRAM-SHA-256 server signature verification, preventing potential timing side-channel attacks
- Fixed: HTTP header injection vulnerability in S3 client where CRLF characters in `contentDisposition`, `contentEncoding`, or `type` options could be used to inject arbitrary HTTP headers
- Fixed: Memory leak (~260KB per request) when cancelling streaming HTTP response bodies via `reader.cancel()` or `body.cancel()`. A strong GC root on the `ReadableStream` was never released on cancellation, causing `ReadableStream` objects, associated `Promise`s, and `Uint8Array` buffers to be retained indefinitely. (thanks @sosukesuzuki!)
- Fixed: Memory leak when cancelling S3 download streams mid-download — `ReadableStream` objects were retained indefinitely because the strong GC reference wasn't released on cancel
- Fixed: `Bun.build()` failing with `NotOpenForReading` when called multiple times after using `FileSystemRouter` routes as entrypoints. The `FileSystemRouter` was caching file descriptors that `Bun.build()` would later close, causing subsequent builds to fail with stale file descriptors. (@ecd4e680)
- Fixed: Crash when constructing objects from entries (e.g. `FileSystemRouter.routes`) caused by GC triggering during partially-initialized object slots
- Fixed: "Unknown HMR script" error that occurred during rapid consecutive file edits when using Bun's dev server with HMR (@prekucki)
- Fixed: Bun.sql now rejects null bytes in connection parameters to prevent protocol injection

### Web APIs

- Fixed: WebSocket connections over `wss://` through an HTTP proxy crashing or receiving spurious 1006 close codes instead of clean 1000 closes when the server sent a ping frame
- Fixed: WebSocket client frame desync when pong payloads were split across TCP segments, which could cause subsequent messages to be misinterpreted as invalid frame headers
- Fixed: Missing RFC 6455 validation for WebSocket pong control frames — payloads exceeding 125 bytes are now correctly rejected, matching the existing behavior for ping and close frames

### bun install

- Fixed: `bun install` producing incomplete `node_modules` on NFS, FUSE, and bind mount filesystems where directory entries were silently skipped due to unknown file types
- Fixed: Path traversal vulnerability in tarball directory extraction.
- Fixed: Scanner-detected undefined behavior in the .npmrc parser when processing truncated or invalid UTF-8 sequences in `.npmrc` files
- Improved: Bun now generates & verifies integrity hashes for GitHub & HTTPS tarball dependencies. Thanks to @dsherret and @orenyomtov for reporting this issue!

### JavaScript bundler

- Fixed: `bun build --compile` on Linux could produce a corrupted binary when a partial write occurred during executable generation
- Fixed: `bun build --compile` producing an all-zeros binary when the output directory is on a different filesystem than the temp directory, common in Docker containers, Gitea runners, and other environments using overlayfs
- Fixed: `bun build --compile --sourcemap=external` not writing `.map` files to disk — they were embedded in the executable but never actually written. With `--splitting`, each chunk now correctly gets its own `.map` file instead of all overwriting a single file. (@AidanGoldworthy)
- Fixed: `bun build` producing syntactically invalid JavaScript (`Promise.resolve().then(() => )`) for unused dynamic imports like `void import("./dep.ts")` or bare `import("./dep.ts")` expression statements
- Fixed: `Bun.build` with HTML entrypoints returning 404s for non-JS/CSS URL assets like `<link rel="manifest" href="./manifest.json" />` — these files are now correctly copied to the output directory instead of being parsed by their extension-based loader
- Fixed: CSS `<link>` tags missing from second (and subsequent) HTML entrypoints when multiple HTML entrypoints shared the same CSS file with `--production` mode bundling

### CSS Parser

- Fixed: CSS bundler leaving duplicate `@layer` declarations and `@import` statements in output when using `@layer` declarations (e.g. `@layer one;`) followed by `@import` rules with `layer()`
- Fixed: CSS bundler incorrectly removing `:root` rules when they appeared before `@property` at-rules due to style rule deduplication merging across at-rule boundaries (thanks @dylan-conway!)

### bun test

- Fixed: `bun test --bail` not writing JUnit reporter output file (`--reporter-outfile`) when early exit was triggered by a test failure

### Bun Shell

- Fixed: `seq inf`, `seq nan`, and `seq -inf` hanging indefinitely in Bun's shell instead of returning an error (thanks @dylan-conway!)
- Fixed: `[[ -d "" ]]` and `[[ -f "" ]]` crashing with an out-of-bounds panic in Bun's shell instead of returning exit code 1 (thanks @dylan-conway!)
- Fixed: Scanner-detected crash when shell builtins (`ls`, `touch`, `mkdir`, `cp`) run inside command substitution `$(...)` and encounter errors (e.g., permission denied) (thanks @dylan-conway!)
- Fixed: Bun's built-in `echo` in the shell treated `-e` and `-E` flags as literal text instead of parsing them, causing commands like `echo -e $password | sudo -S ...` to fail. Now supports `-e` (enable backslash escapes), `-E` (disable backslash escapes), and combined flags like `-ne`/`-en`, matching bash behavior. Supported escape sequences include `\\`, `\a`, `\b`, `\c`, `\e`, `\f`, `\n`, `\r`, `\t`, `\v`, `\0nnn` (octal), and `\xHH` (hex)
- Fixed: `Bun.$` shell template literals leaking internal `__bunstr_N` references in output when an interpolated value contained a space and a subsequent value contained multi-byte UTF-8 characters (e.g., `Í`, `€`)
- Fixed: Scanner-detected crash in the `seq` shell builtin when called with only flags and no numeric arguments (e.g. `await Bun.$\`seq -w\``)
- Fixed: crash in the shell interpreter when `setupIOBeforeRun` fails (e.g., stdout handle unavailable on Windows), which caused a segfault during GC sweep

### TypeScript types

- Fixed: TypeScript types for `Bun.build()` now correctly allow `splitting` to be used together with `compile` (@alii)

### Windows

- Fixed: Crash that could occur when spawning processes or writing to pipes in long-lived applications
- Fixed: Crash on Windows when a standalone executable with `compile.autoloadDotenv = false` spawned a `Worker` in a directory containing a `.env` file. The dotenv loader was mutating environment state owned by another thread, causing a `ThreadLock` assertion panic. (Thanks to @Hona!)
- Fixed: `"switch on corrupt value"` panic on Windows impacting Claude Code & Opencode users
- Fixed: Hypothetical crash on Windows when `GetFinalPathNameByHandleW` returned paths exceeding buffer capacity

### Thanks to 11 contributors!

- [@alanstott](https://github.com/alanstott)
- [@alii](https://github.com/alii)
- [@cirospaciari](https://github.com/cirospaciari)
- [@dylan-conway](https://github.com/dylan-conway)
- [@hk-shao](https://github.com/hk-shao)
- [@hona](https://github.com/hona)
- [@jarred-sumner](https://github.com/jarred-sumner)
- [@martinamps](https://github.com/martinamps)
- [@prekucki](https://github.com/prekucki)
- [@robobun](https://github.com/robobun)
