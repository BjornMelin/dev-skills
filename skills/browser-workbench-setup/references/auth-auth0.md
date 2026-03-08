# Auth0 Setup

Use this file when the repository uses Auth0 in the browser.

## Official Docs

- Next.js quickstart: https://auth0.com/docs/quickstart/webapp
- Application settings: https://auth0.com/docs/get-started/applications/application-settings
- Management API access tokens: https://auth0.com/docs/secure/tokens/access-tokens/management-api-access-tokens
- Manage users with the Management API: https://auth0.com/docs/manage-users/user-accounts/manage-users-using-the-management-api
- Create users: https://auth0.com/docs/manage-users/user-accounts/create-users
- Auth0 Next.js SDK repo and examples: https://github.com/auth0/nextjs-auth0

## Current Official Guidance

Auth0 does not currently provide the same first-party Playwright helper experience that Clerk does.

For browser automation, the reliable default is:

- use a dedicated database-connection test user
- sign in through the real app UI
- save browser state for reuse

If the repo uses `@auth0/nextjs-auth0`, there is also a testing utility for generating session cookies in integration-style tests:

- `@auth0/nextjs-auth0/testing`
- `generateSessionCookie`

Use that only when formal test setup is explicitly in scope and the repo already uses the Next.js SDK.

## Repo Detection

Typical signals:

- `@auth0/nextjs-auth0`
- `AUTH0_DOMAIN`
- `AUTH0_CLIENT_ID`
- `AUTH0_CLIENT_SECRET`
- `AUTH0_SECRET`

## playwright-interactive Pattern

Preferred interactive path:

1. Create or seed a dedicated database-connection user.
2. Sign in through the app UI.
3. Wait for the authenticated landing page.
4. Save `storageState` to:
   - `output/playwright/auth/auth0-user.json`

Preferred account shape:

- dedicated test user
- standard database connection
- MFA disabled unless the repo explicitly supports automated MFA testing

Avoid social-login-only paths for deterministic local browser automation.

## Formal Playwright Setup

Choose one:

- normal UI login once + saved `storageState`
- for Next.js Auth0 repos, optionally use `generateSessionCookie` in lower-level integration-style tests

If you need to create users automatically:

- obtain a Management API access token
- use the Management API with the minimum scopes needed
- create users only in the intended database connection

Do not add management-token setup to the repo unless the user explicitly wants automated test-user provisioning.

## agent-browser Pattern

Use a persistent profile:

```bash
agent-browser --profile ./output/agent-browser/profile open http://127.0.0.1:3000
```

Then complete login through the app UI and reuse the profile.

Prefer profile persistence over session-name persistence because Auth0 app flows often involve redirect chains and cookie state that are easier to preserve in a full profile.

## Automation Preference

When automating setup in an Auth0 repo:

- install Playwright and Chromium
- create artifact directories and ignore rules
- prefer real UI login plus saved browser state
- add Auth0 test-user provisioning only if the user asked for formal repeatable provisioning
