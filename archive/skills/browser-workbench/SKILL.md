---
name: browser-workbench
description: Browser/UI router--playwright-interactive, agent-browser, workbench-setup. Triggers--UI debug, QA, screens, auth, web test.
disable-model-invocation: true
---

# Browser Workbench

Use this as the default entry point for browser work.

For broader app/UI routing across web and Expo/mobile, prefer `ui-workbench`.
For final release QA and explicit evidence capture, prefer `ui-release-signoff`.

This skill is a router, not a browser framework. It decides whether the job should use:

- `browser-workbench-setup`
- `playwright-interactive`
- `agent-browser`
- a staged flow: `playwright-interactive` first, then `agent-browser`

Do not load both browser runtime skills by default. Pick one path unless the task clearly benefits from a staged workflow.

## Default Policy

- Prefer `playwright-interactive` for serious local UI work.
- Prefer `agent-browser` for quick CLI-first smoke checks and annotated artifacts.
- Use `browser-workbench-setup` when the repo is not ready yet.
- Use a staged flow only when the second tool adds distinct value.

## First Step

Before choosing a path, do one quick environment pass:

- check whether the repo has Playwright installed locally
- check whether browser artifacts and ignore rules exist
- check whether auth persistence has already been set up
- check whether the task is local-app debugging, quick smoke testing, or setup/bootstrap

## Routing Rules

### Path: Setup

Choose `browser-workbench-setup` when any of these are true:

- Playwright is not installed in the repo
- browser binaries are missing
- artifact directories or ignore rules are missing
- auth persistence is not configured and the task depends on authenticated browser work
- the user asked to bootstrap a repo for browser automation

When you choose this path:

- state `Path: setup`
- use `browser-workbench-setup`
- let that skill decide the auth-provider-specific setup

### Path: playwright-interactive

Choose `playwright-interactive` by default for:

- local web-app development
- UI debugging that may take multiple iterations
- auth-heavy flows
- desktop plus mobile signoff
- console, network, or runtime investigation
- visual polish and final QA evidence

When you choose this path:

- state `Path: playwright-interactive`
- use `playwright-interactive` as the main loop
- keep the session alive across fixes and verification

### Path: agent-browser

Choose `agent-browser` for:

- quick smoke checks on a running app or site
- annotated screenshots
- accessibility-tree snapshots
- simple DOM or screenshot diffs
- lightweight scraping or extraction
- simple page-error and console review

When you choose this path:

- state `Path: agent-browser`
- use `agent-browser` only
- keep the workflow CLI-first and fast

### Path: staged playwright-interactive -> agent-browser

Choose the staged path only when:

- `playwright-interactive` is needed for the real investigation or auth flow
- and `agent-browser` adds a specific secondary benefit such as:
  - annotated screenshots
  - quick snapshot/screenshot diff
  - a fast post-fix smoke pass

When you choose this path:

- state `Path: staged playwright-interactive -> vercel:agent-browser`
- do the real investigation and signoff work in `playwright-interactive`
- use `vercel:agent-browser` only for the secondary evidence or check

## Anti-Patterns

- Do not mention both `$playwright-interactive` and `$vercel:agent-browser` in ordinary prompts.
- Do not run both runtime skills in parallel unless the task explicitly requires that complexity.
- Do not use `vercel:agent-browser` as the primary debugger for a stateful auth-heavy UI task.
- Do not use `playwright-interactive` for a narrow one-shot CLI smoke task if `vercel:agent-browser` is enough.
- Do not duplicate provider-auth setup guidance here. Route setup/bootstrap work to `browser-workbench-setup`.

## Prompting Guidance

Recommend these patterns to users:

- `Use $browser-workbench to debug and polish this UI flow, verify auth, and capture desktop/mobile evidence.`
- `Use $browser-workbench to smoke-test this running app and capture annotated screenshots of any issues.`
- `Use $browser-workbench to set up this repo for browser-based UI testing and auth persistence.`

Treat direct low-level prompts as advanced overrides:

- `$playwright-interactive` means the user already wants the persistent Playwright path
- `$vercel:agent-browser` means the user already wants the CLI-first path

## Output Contract

When using this skill, begin with the chosen route in the first progress update:

- `Path: setup`
- `Path: playwright-interactive`
- `Path: vercel:agent-browser`
- `Path: staged playwright-interactive -> vercel:agent-browser`

If setup is required before runtime work, do not improvise around missing prerequisites. Route to setup first.
