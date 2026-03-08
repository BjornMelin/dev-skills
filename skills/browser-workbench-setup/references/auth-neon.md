# Neon Auth Setup

Use this file when the repository uses Neon Auth directly or uses Neon as the JWT-verifying database backend.

## Official Docs

- Neon API reference: https://api-docs.neon.tech/
- Add JWKS URL: https://api-docs.neon.tech/reference/addprojectjwks
- Create Auth Provider SDK keys: https://api-docs.neon.tech/reference/createneonauthprovidersdkkeys
- Create Neon Auth integration: https://api-docs.neon.tech/reference/createneonauthintegration
- Create auth user: https://api-docs.neon.tech/reference/createneonauthnewuser
- Branch auth OAuth provider endpoint: https://api-docs.neon.tech/reference/addbranchneonauthoauthprovider

## Important Model

Neon can appear in two different roles:

1. Neon-managed or provider-owned auth for the application itself
2. Neon database verification of JWTs issued by an upstream provider such as Clerk or Auth0

Do not confuse them.

If the browser login belongs to Clerk/Auth0/Supabase, the browser automation instructions should come from that provider file. Neon then matters for backend JWT verification and project configuration, not for the browser sign-in UI itself.

## Repo Detection

Typical signals:

- Neon auth endpoints or setup scripts
- Neon auth provisioning in project automation
- JWKS configuration for database auth
- external provider JWTs accepted by Neon

## Browser Automation Pattern

If Neon Auth owns the user lifecycle:

- create or provision a test user through the Neon-managed auth path
- sign in through the app UI
- save Playwright `storageState` or an `agent-browser` profile

If Neon only verifies JWTs from another provider:

- follow the upstream provider auth instructions for browser login
- verify that Neon’s JWT configuration matches that provider

## Neon JWT Verification Pattern

For external-provider JWT auth to Neon:

- configure the project JWKS URL
- set `provider_name`
- set `jwt_audience` when required
- limit accepted roles and branches when appropriate

This is not a browser automation step, but it is often a prerequisite for the authenticated app flow to work correctly against Neon-backed data.

## playwright-interactive Pattern

Preferred interactive path:

- if Neon Auth is the browser identity provider, log in normally and save:
  - `output/playwright/auth/neon-user.json`
- if Neon is downstream of Clerk/Auth0/etc., save storage state under the upstream provider name instead and separately validate Neon JWT config

## agent-browser Pattern

Use a persistent profile:

```bash
agent-browser --profile ./output/agent-browser/profile open http://127.0.0.1:3000
```

Then:

- complete the browser login through the app’s actual auth UI
- reuse the profile for smoke checks

## Automation Preference

When automating setup in a Neon repo:

- first determine whether Neon is the browser-facing auth provider or only the backend verifier
- if browser-facing, treat it like the primary provider and persist browser state normally
- if backend-only, configure browser auth from the upstream provider and validate Neon JWKS/JWT settings separately

## Current Caution

Some older Neon auth endpoints are marked deprecated in the current API reference. Prefer current branch-scoped endpoints when available over older project-scoped auth setup flows.
