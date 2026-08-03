---
name: sentry-cli
description: "Sentry CLI atoms for safe issues, traces, releases, projects, sourcemaps, and deliberate mutations."
license: MIT
metadata:
  verified_cli:
    sentry: "0.38.0"
    sentry-cli: "3.4.2"
---

# Sentry CLI

Use this explicit skill for current Sentry command discovery and bounded
operations. Use `$sentry-cli-fix-issues` for an end-to-end production issue
investigation and repository fix.

## Pick the right binary

| Need | Use | Do not substitute |
| --- | --- | --- |
| Issues, events, traces, logs, orgs, projects, releases, API schema | `sentry` | `sentry-cli` subcommand syntax |
| Sourcemaps, debug files, ProGuard, React Native artifacts | repo wrapper or `sentry-cli` | `sentry` subcommand syntax |

The local `sentry` and legacy `sentry-cli` executables are different CLIs.
Confirm the installed command and verb before acting:

```sh
sentry --version
sentry <noun> --help
sentry-cli --version
sentry-cli <noun> --help
```

## Inspect first

1. Check authentication without exposing credentials:
   `sentry auth status --json`. Never use `--show-token` or `sentry auth token`.
2. Confirm the target with a bounded read; specify `org/project` only when
   automatic detection is wrong.
3. Prefer `--json`, `--fields`, `--limit`, and a narrow `--period`; redact
   customer data, headers, cookies, prompts, and payload values from reports.

Useful reads:

```sh
sentry issue list --query 'is:unresolved' --limit 20 --period 24h --json
sentry issue view ISSUE --json
sentry issue events ISSUE --limit 10 --json
sentry trace view TRACE --json
sentry log list --limit 20 --json
sentry release list --limit 20 --json
sentry project list --limit 20 --json
sentry schema issues --json
```

Treat `sentry issue explain` and `sentry issue plan` as advisory: verify their
claims against raw event evidence and the checked-out code.

## Mutation gate

Get an explicit user decision before running commands that change remote state,
credentials, billing, or release history. Show the exact command and target
first. Use dry runs where available.

| Mutating area | Examples |
| --- | --- |
| Issue state | `resolve`, `unresolve`, `archive`, `merge` |
| Releases | `create`, `finalize`, `delete`, `archive`, `restore`, `deploy`, `set-commits` |
| Projects, alerts, monitors | `create`, `delete`, `update`, `start` |
| Raw API | `sentry api` with non-GET method or request body |
| Artifact upload | `sentry-cli sourcemaps`, `debug-files`, `react-native`, `proguard` |
| Authentication | `auth login`, `logout`, `refresh` |

Never pass an auth token on the command line or print it. Use the CLI's normal
credential store or an approved environment variable. Confirm token scope and
org/project access before release or artifact writes.

## Source-map and release evidence

Use the owning repository's package script first; it pins the right
`@sentry/cli` version and release/build inputs. If no wrapper exists, inspect
the exact legacy verb with `sentry-cli <noun> --help` before proposing a write.
Do not replace a repository's upload integration with an ad-hoc global command.

## Closeout

Report the binary and version used, command, read/write status, target,
bounded evidence, and verification result. Mark provider facts `UNVERIFIED`
until a current command or official authority proves them.
