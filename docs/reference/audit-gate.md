# audit-gate Reference

`audit-gate` is the shared CI-gate layer behind [`gsap-audit`](gsap-audit.md),
[`expo-motion-audit`](expo-motion-audit.md) and
[`motion-token-audit`](motion-token-audit.md). It is a library, not a binary.

Each auditor keeps its own parser, rule catalog and severity enum, and converts
into this crate's normalized `GateFinding` at the CLI boundary. The conversion
is deliberate: the cores stay independent of each other, and this crate stays
free of any parser dependency.

## What it provides

| Capability | Flag it backs | Notes |
| --- | --- | --- |
| Path exclusion | `--exclude <GLOB>` | `*`, `**`, `?`. A bare name matches any path component. |
| Finding baselines | `--baseline`, `--write-baseline` | Fingerprinted `rule-id::file`. |
| SARIF 2.1.0 output | `--format sarif` | Includes `partialFingerprints`; paths are re-anchored to the repository root so annotations land correctly when `--root` is nested. |

## Fingerprints

A baseline entry is `rule-id::file::ordinal`. Line numbers and messages are
excluded on purpose: findings move when unrelated lines above them change, and
messages get reworded. A fingerprint sensitive to either produces a baseline
that expires on contact with ordinary editing, which is the failure that makes
teams delete baselines rather than maintain them.

The ordinal preserves counts. Several rules fire once per literal or per call,
so one file can hold many occurrences of the same rule; collapsing them to a
single entry would let a newly added occurrence pass unnoticed.

The cost is that moving a finding to a different file un-baselines it. That is
the right default, because it usually is new code.

## Glob semantics

Implemented in-crate rather than pulled from a glob dependency: the patterns a
path gate needs are ordinary, and the auditors have no other use for a glob
engine.

- `*` matches within one path component and stops at `/`.
- `**` crosses separators.
- `?` matches a single non-separator character.
- A pattern with no separator and no wildcard matches any whole component, so
  `--exclude node_modules` behaves as expected while `node_modules_helper.ts`
  does not match.
- Backslashes are normalized to `/` before matching.

## Baseline file

```json
{
  "schema": "audit-gate.baseline.v1",
  "findings": ["core.gsap-trial-import::src/a.ts"]
}
```

`schema` is checked on load. A file written by a different schema is an error
rather than a baseline that silently matches nothing.

## Validation

```bash
cargo test -p audit-gate
cargo clippy -p audit-gate --all-targets -- -D warnings
```
