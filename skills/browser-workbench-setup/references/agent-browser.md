# Agent Browser Setup

Use this file when configuring the secondary CLI browser workflow.

## Purpose

`agent-browser` is the secondary tool for:

- quick smoke checks
- annotated screenshots
- accessibility-tree snapshots
- fast console and page-error reads
- lightweight DOM or screenshot diffs

Official source:

- https://github.com/vercel-labs/agent-browser

Key upstream capabilities from the README:

- session names for saved state
- persistent profiles
- annotated screenshots
- console and page error inspection
- snapshot and screenshot diff commands

## Install and Baseline

Install once at the user level and install Chromium:

```bash
agent-browser install
```

On Linux, if needed:

```bash
agent-browser install --with-deps
```

## User-Level Config

Preferred user config at `~/.agent-browser/config.json`:

```json
{
  "headed": true,
  "contentBoundaries": true,
  "maxOutput": 50000
}
```

This belongs outside the repo.

## Repo-Level Convention

Preferred repo artifact directory:

- `output/agent-browser/`

Ignore it in git:

```gitignore
output/agent-browser/
```

## Persistence Standard

Choose one of these:

- `--profile <path>` when you want full browser persistence including cookies, local storage, IndexedDB, and service workers
- `--session-name <name>` when you want lighter automatic session restore through agent-browser’s saved state

Preferred defaults for repo work:

- quick repeated local UI checks: `--profile ./output/agent-browser/profile`
- lightweight smoke path reuse: `--session-name <repo-name>`

Use profiles when auth flows are complex or rely on richer browser state.

## Auth Handling

For most real apps, sign in normally once, then persist the authenticated state with a profile or session name.

Do not rely on the credential vault flow unless the app uses a simple username/password form that the CLI can drive cleanly. For OAuth-heavy or redirect-heavy apps, browser-state persistence is more reliable.

## Recommended Commands

Quick smoke:

```bash
agent-browser --profile ./output/agent-browser/profile open http://127.0.0.1:3000
agent-browser snapshot -i
agent-browser console
agent-browser errors
agent-browser screenshot --annotate ./output/agent-browser/home.png
```

Reuse session later:

```bash
agent-browser --profile ./output/agent-browser/profile open http://127.0.0.1:3000/dashboard
```

## Role Split

If the task becomes stateful, complex, auth-heavy, or visually subtle, switch to `playwright-interactive`.

Do not force `agent-browser` to be the primary debugger.
