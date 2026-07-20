# pm-bun-audit-security

## Why

Dependency vulnerabilities and malicious packages are a real supply-chain risk. Bun ships
its own auditing, so a Bun-first repo does not need a separate tool.

## Do

- Scan installed dependencies for known advisories with `bun audit` (it checks the
  packages in `bun.lock`).
- Enable a security scanner that runs during `bun install` / `bun add` by setting it in
  `bunfig.toml`:

  ```toml
  [install.security]
  scanner = "<scanner-package>"   # a scanner published by a security vendor, e.g. Socket
  ```

  Install the scanner as a dev dependency first (`bun add -d <scanner-package>`).
- Run `bun audit` in CI and on dependency updates; triage findings before merging.
- Combine with `--minimum-release-age` (see `pm-linker-and-streaming-install`) to reduce
  fresh-publish risk.

## Don't

- Don't wire `npm audit` / `yarn npm audit` into a Bun-first repo; use `bun audit`.
- Don't rely on a `bun pm scan` command - package scanning is configured via
  `[install.security]` and runs at install time, not as a standalone subcommand.
- Don't auto-apply major-version "fixes" without reviewing breaking changes.

## Examples

```bash
bun audit
bun add -d <scanner-package>   # a security-vendor scanner, then set [install.security].scanner
```
