# Playwright Interactive Setup

Use this file when the repository needs the primary interactive browser workflow.

## Purpose

`playwright-interactive` is the primary tool for:

- iterative UI/UX development
- auth-heavy browser flows
- desktop and mobile passes
- deeper debugging with direct Playwright control
- screenshots and traces that support final signoff

Official skill source:

- https://github.com/openai/skills/blob/main/skills/.curated/playwright-interactive/SKILL.md

Official Playwright docs:

- Auth and storage state: https://playwright.dev/docs/auth
- Emulation and devices: https://playwright.dev/docs/emulation
- Screenshots: https://playwright.dev/docs/screenshots
- Trace viewer: https://playwright.dev/docs/trace-viewer
- Videos: https://playwright.dev/docs/videos

## Repo Setup

Match the repository package manager. For Bun-first repos:

```bash
bun add -d playwright
bunx playwright install chromium
```

Optional:

```bash
bunx playwright install firefox webkit
```

Default artifact layout:

- `output/playwright/screenshots/`
- `output/playwright/traces/`
- `output/playwright/auth/`

Ignore them in git:

```gitignore
output/playwright/
playwright/.auth/
```

## Persistence Standard

For interactive browser work, the default persistence primitive is Playwright `storageState`.

Preferred file location:

- `output/playwright/auth/<provider>-user.json`

Use `storageState` when:

- the repo has a stable login flow
- the session should be reused across repeated debugging runs
- you want deterministic auth setup for later formal tests

Do not commit storage state files.

## First Session Pattern

1. Start the local app.
2. Open the unprotected landing or sign-in page in Playwright.
3. Complete sign-in using the provider-specific path.
4. Save `storageState` after the authenticated landing page is stable.
5. Reuse that file for follow-up sessions instead of logging in again.

## When to Add Formal Test Bootstrap

Only add repo-local Playwright auth bootstrap files when at least one is true:

- the repo already has Playwright tests
- the user asked for formal browser tests
- auth login is slow enough that repeated UI login is wasteful

If you add formal bootstrap, keep it minimal:

- one auth setup file
- one storage-state file path convention
- no helper framework unless the provider officially supplies one

## Desktop and Mobile Standard

For signoff work:

- run one deterministic desktop pass
- run one mobile-emulated pass
- capture screenshots for both when the UI changed

Use explicit viewport mode first. Use native window mode only as a second pass when the bug depends on real browser chrome or OS window sizing.
