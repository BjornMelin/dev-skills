# Performance, Unsafe, And Security

Use this reference when performance, unsafe, FFI, supply chain, secrets,
subprocesses, paths, parsing, or untrusted input are in scope.

## Performance Order

1. Measure in release mode.
2. Check algorithm and data structure first.
3. Inspect allocation and clone patterns.
4. Check I/O batching and lock contention.
5. Add parallelism/SIMD/unsafe only after identifying a real hotspot.

Useful tools:

```bash
cargo bench
cargo flamegraph --bin <binary>
heaptrack ./target/release/<binary>
cargo bloat --release --bin <binary>
cargo tree -e features
```

Avoid optimizing debug builds, hiding algorithmic problems with parallelism, or
adding `unsafe` for micro-optimizations without proof.

## Unsafe Rules

Every `unsafe` block or item needs:

- a `SAFETY:` comment explaining caller obligations and why they hold;
- the smallest possible unsafe scope;
- a safe abstraction around the boundary when possible;
- checks for aliasing, lifetimes, initialization, alignment, thread-safety, and
  panic behavior;
- Miri, sanitizer, fuzzing, or targeted tests when risk warrants it.

## FFI Rules

- Confirm ABI and `repr(C)` layout.
- Document ownership transfer and allocation/freeing side.
- Validate nullability before dereference.
- Do not unwind across FFI.
- Convert raw pointers at the boundary.
- Make cleanup explicit when it can fail.

## Supply Chain

Prefer maintained crates with clear license, docs, source, CI, and recent
activity. Run configured policy tools:

```bash
cargo deny check
cargo audit
cargo tree -d
cargo machete
```

Use `cargo-udeps` only when nightly is acceptable. Treat build scripts and proc
macros as higher-risk code in sensitive environments.

## Application Security

Pay extra attention to:

- auth and authorization checks;
- crypto;
- parsers and deserialization of untrusted data;
- filesystem paths and archive extraction;
- subprocess arguments and shell execution;
- network timeouts and TLS;
- secrets in logs/errors;
- temporary files and permissions.
