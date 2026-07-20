# pm-bun-audit-security

## Why

Dependency vulnerabilities and malicious packages are a real supply-chain risk. Bun ships
its own auditing, so a Bun-first repo does not need a separate tool.

## Do

- Scan installed dependencies for known advisories with `bun audit`.
- Use `bun pm scan` for Bun's package scanner (malware / known-bad signals) when
  available.
- Run the audit in CI and on dependency updates; triage findings before merging.
- Combine with `--minimum-release-age` (see `pm-linker-and-streaming-install`) to reduce
  fresh-publish risk.

## Don't

- Don't wire `npm audit` / `yarn npm audit` into a Bun-first repo; use `bun audit`.
- Don't auto-apply major-version "fixes" without reviewing breaking changes.

## Examples

```bash
bun audit
bun pm scan
```
