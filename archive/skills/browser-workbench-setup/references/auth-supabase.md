# Supabase Auth Setup

Use this file when the repository uses Supabase Auth in the browser.

## Official Docs

- Auth with email/password: https://supabase.com/docs/guides/auth/passwords
- Next.js server-side auth: https://supabase.com/docs/guides/auth/server-side/nextjs
- `@supabase/ssr`: https://supabase.com/docs/guides/auth/server-side
- JavaScript client auth reference: https://supabase.com/docs/reference/javascript/auth-signinwithpassword

## Current Official Guidance

Key points:

- browser login typically uses `signInWithPassword()`
- Next.js SSR setups should use `@supabase/ssr`
- SSR session refresh depends on correct cookie synchronization in middleware
- admin user creation is available through `supabase.auth.admin.createUser()`

## Repo Detection

Typical signals:

- `@supabase/supabase-js`
- `@supabase/ssr`
- `NEXT_PUBLIC_SUPABASE_URL`
- `NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY`

## playwright-interactive Pattern

Preferred interactive path:

1. Seed or create a dedicated test user.
2. Sign in through the app UI using email/password.
3. Wait until the authenticated landing page is stable.
4. Save `storageState` to:
   - `output/playwright/auth/supabase-user.json`

Preferred account shape:

- seeded dedicated test user
- password auth enabled
- email already confirmed if the app requires confirmation

Avoid magic-link-only flows for deterministic browser automation unless the repo already has an inbox/test-email harness.

## Formal Playwright Setup

Supabase does not provide a first-party Playwright helper like Clerk.

Preferred formal path:

- create or seed a test user with admin APIs or project seed logic
- authenticate once through the real UI
- save `storageState`
- reuse it for authenticated tests

If the repo already exposes server utilities for seeding users, prefer those over writing one-off scripts.

## SSR Caution

For Next.js repos using `@supabase/ssr`, do not disturb the existing middleware/session-refresh path while adding browser automation.

If you touch auth middleware, preserve the cookie handoff exactly. Session drift between server and browser is a common failure mode.

## agent-browser Pattern

Use a persistent profile:

```bash
agent-browser --profile ./output/agent-browser/profile open http://127.0.0.1:3000/login
```

Then:

- sign in through the UI
- reuse the same profile for follow-up smoke checks

Use `--profile` rather than `--session-name` when the app depends on richer browser state or SSR cookie refresh behavior.

## Automation Preference

When automating setup in a Supabase repo:

- add Playwright and Chromium
- create artifact directories and ignore rules
- preserve the repo’s SSR cookie/session architecture
- prefer seeded test users plus saved browser state
