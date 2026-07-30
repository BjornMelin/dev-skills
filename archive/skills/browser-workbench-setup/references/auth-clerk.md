# Clerk Auth Setup

Use this file when the repository uses Clerk in the browser.

## Official Docs

- Testing overview: https://clerk.com/docs/guides/development/testing/overview
- Playwright overview: https://clerk.com/docs/guides/development/testing/playwright/overview
- Playwright test helpers: https://clerk.com/docs/guides/development/testing/playwright/test-helpers
- Authenticated flows with Playwright: https://clerk.com/docs/guides/development/testing/playwright/test-authenticated-flows

## Current Official Guidance

Clerk has first-party Playwright helpers. This is the best-supported provider in this set for formal Playwright auth setup.

Key points from the docs:

- use `clerkSetup()` in global setup
- use `setupClerkTestingToken()` at the start of tests that touch auth pages
- `clerk.signIn()` supports `password`, `phone_code`, and `email_code`
- navigate to an unprotected page that loads Clerk before calling the helper
- MFA is not supported by `clerk.signIn()`

## Repo Detection

Typical signals:

- `@clerk/nextjs`
- `@clerk/testing/playwright`
- `CLERK_SECRET_KEY`
- `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY`

## playwright-interactive Pattern

For interactive work:

1. Start the app.
2. Open an unprotected page that loads Clerk.
3. If the repo already includes Clerk testing helpers, prefer them.
4. Otherwise sign in through the normal UI with a dedicated non-MFA test user.
5. Save `storageState` to:
   - `output/playwright/auth/clerk-user.json`

Preferred account shape:

- dedicated test user
- password auth if available
- MFA disabled

If the product requires MFA, keep interactive browser setup separate from formal test auth and mark any fully automated login path `UNVERIFIED` unless the repo already supports it.

## Formal Playwright Setup

If the repo already has or needs Playwright tests:

- add `@clerk/testing/playwright`
- create one serial global setup file using `clerkSetup()`
- authenticate once and save storage state
- reuse that storage state in authenticated tests

Do not re-click the full sign-in UI in every test.

## agent-browser Pattern

`agent-browser` has no Clerk-specific first-party helper.

Use this pattern:

1. Open the app with a persistent profile:
   - `agent-browser --profile ./output/agent-browser/profile open http://127.0.0.1:3000`
2. Complete sign-in through the normal app UI.
3. Reuse the profile for later smoke checks.

Prefer `--profile` over `--session-name` for Clerk because redirects and local state are often richer than simple cookie-only persistence.

## Automation Preference

When automating setup in a Clerk repo:

- install Playwright
- add artifact directories and ignore rules
- if formal browser tests are in scope, wire in Clerk’s official Playwright helpers
- otherwise keep auth bootstrap light and rely on saved `storageState` for interactive work
